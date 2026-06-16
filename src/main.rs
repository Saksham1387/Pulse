use std::{cell::RefCell, io, sync::{Arc, Mutex, mpsc}, thread, time::Duration};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};
use sysinfo::{System, Users};

use crate::formatters::{format_bytes, format_cpu, format_disk_usage, format_duration};

pub mod formatters;

fn main() -> io::Result<()> {
    // This initilization is in the raw mode. Meaning the out fo the box feature it gives are disabled by default i.e Ctrl + c to end the task
    let mut terminal = ratatui::init();
    let sys = System::new_all();

    let table_state = TableState::default();
    let mut app = App {
        exit: false,
        sys: Arc::new(Mutex::new(sys)),
        process_table_state: RefCell::new(table_state),
        users: Users::new_with_refreshed_list()
    };
    let app_result = app.run(&mut terminal);

    ratatui::restore();
    app_result
}

enum AppEvent {
    Tick,
    Key(crossterm::event::KeyEvent)
}

pub struct App {
    exit: bool,
    sys: Arc<Mutex<System>>,
    process_table_state: RefCell<TableState>,
    users:Users
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let (tx, rx) = mpsc::channel::<AppEvent>();
        let sys_clone = Arc::clone(&self.sys);
        let tx_tick = tx.clone();

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(1));
            {
                let mut sys = sys_clone.lock().unwrap();
                sys.refresh_all();
            }
            if tx_tick.send(AppEvent::Tick).is_err() {
                break;
            }
        });

        let tx_key = tx.clone();
        thread::spawn(move || loop {
            if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                if tx_key.send(AppEvent::Key(key)).is_err() {
                    break;
                }
            }
        });

        terminal.draw(|frame| self.draw(frame))?;

        while !self.exit {

            match rx.recv() {
                Ok(AppEvent::Tick) => {
                    terminal.draw(|f| self.draw(f))?;
                }

                Ok(AppEvent::Key(key)) => {
                    self.handle_key_event(key)?;
                    terminal.draw(|f| self.draw(f))?;
                }

                Err(_) => break,
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
        }

        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Down {
            self.process_table_state.borrow_mut().scroll_down_by(10);
        }

        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Up {
            self.process_table_state.borrow_mut().scroll_up_by(10);
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let sys = self.sys.lock().unwrap();
        let mut rows: Vec<Row> = Vec::with_capacity(sys.processes().len());

        let vertical_layout =
            Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)]);

        let [title_area, guage_area] = vertical_layout.areas(area);

        let block = Block::bordered()
            .title("Pulse")
            .border_set(border::THICK)
            .style(Style::default().fg(Color::Cyan));

        let inner_title_area = block.inner(title_area);

        let horizontal_layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);

        let [sys_info_area, cpus_area] = horizontal_layout.areas(inner_title_area);

        let header = Row::new([
            "PID", "User", "Name", "Time", "State", "CPU", "Memory", "Disk",
        ])
        .style(Style::new().bold().black())
        .bg(Color::Green);
        
        let bars: Vec<Bar> = sys.cpus().iter()
            .map(|cpu| {
                let usage = cpu.cpu_usage();
                let bar_style = if usage > 80.0 {
                    Style::new().red()
                } else if usage > 50.0 {
                    Style::new().yellow()
                } else {
                    Style::new().blue()
                };
                Bar::default()
                    .value(usage as u64)
                    .label(Line::from(format!("{:.0}%", usage)))
                    .text_value(cpu.name().to_string())
                    .style(bar_style)
                    .value_style(Style::new().white().bold())
            })
            .collect();

        let n_cpus = bars.len() as u16;
        let bar_gap: u16 = 1;
        let available_width = cpus_area.width.saturating_sub(2);
        let bar_width = if n_cpus > 0 {
            ((available_width + bar_gap) / n_cpus).saturating_sub(bar_gap).max(1)
        } else {
            1
        };

        BarChart::default()
            .data(BarGroup::default().bars(&bars))
            .bar_width(bar_width)
            .bar_gap(bar_gap)
            .max(100)
            .label_style(Style::new().cyan())
            .render(cpus_area, buf);
        
        for (pid, process) in sys.processes() {
            let username = process
                .user_id()
                .and_then(|uid| self.users.get_user_by_id(uid))
                .map(|user| user.name().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            rows.push(Row::new([
                pid.to_string(),
                username,
                process.name().to_string_lossy().to_string(),
                format_duration(process.run_time()),
                process.status().to_string(),
                format_cpu(process.cpu_usage()),
                format_bytes(process.memory()),
                format_disk_usage(process.disk_usage()),
            ]));
        }

        let widths = [
            Constraint::Percentage(10), // PID
            Constraint::Percentage(10), // User ID
            Constraint::Percentage(40), // Name
            Constraint::Percentage(10), // Time
            Constraint::Percentage(10), // Status
            Constraint::Percentage(10), // CPU
            Constraint::Percentage(10), // Memory
            Constraint::Percentage(10), // Disk
        ];

        let system_info_text = format!(
            " OS: {}\n Total Memory: {}\n Used memory: {}\n Total swap: {}\n No. of CPUs: {}",
            System::name().unwrap_or_default(),
            format_bytes(sys.total_memory()),
            format_bytes(sys.used_memory()),
            format_bytes(sys.total_swap()),
            sys.cpus().len()
        );

        Paragraph::new(system_info_text)
            .style(Style::default().fg(Color::White))
            .render(sys_info_area, buf);

        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(1)
            .style(Color::White)
            .row_highlight_style(Style::new().on_black().bold())
            .column_highlight_style(Color::Gray)
            .cell_highlight_style(Style::new().reversed().yellow())
            .highlight_symbol("🍴 ");

        block.render(title_area, buf);

        StatefulWidget::render(
            table,
            guage_area,
            buf,
            &mut self.process_table_state.borrow_mut(),
        );
    }
}
