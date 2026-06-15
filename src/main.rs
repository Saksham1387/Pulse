use std::{cell::RefCell, io, time::Duration};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    widgets::{Block, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
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
        sys,
        process_table_state: RefCell::new(table_state),
    };
    let app_result = app.run(&mut terminal);

    ratatui::restore();
    app_result
}

pub struct App {
    exit: bool,
    sys: System,
    process_table_state: RefCell<TableState>,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if crossterm::event::poll(Duration::from_millis(200))? {
                if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                    self.handle_key_event(key_event)?;
                }
            }

            self.sys.refresh_all();
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
        let mut rows: Vec<Row> = Vec::new();

        let vertical_layout =
            Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
        let [title_area, guage_area] = vertical_layout.areas(area);

        let header = Row::new([
            "PID", "User", "Name", "Time", "State", "CPU", "Memory", "Disk",
        ])
        .style(Style::new().bold().black())
        .bg(Color::Green);

        let users = Users::new_with_refreshed_list();
        for (pid, process) in self.sys.processes() {
            let username = process
                .user_id()
                .and_then(|uid| users.get_user_by_id(uid))
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

        let block = Block::bordered()
            .title("Pulse")
            .border_set(border::THICK)
            .style(Style::default().fg(Color::Cyan));
        let block_inner_area = block.inner(title_area);

        let system_info_text = format!(
            "OS: {}\nTotal Memory: {}\nUsed memory: {}\nTotal swap: {}\nNo. of CPUs: {}",
            System::name().unwrap_or_default(),
            format_bytes(self.sys.total_memory()),
            format_bytes(self.sys.used_memory()),
            format_bytes(self.sys.total_swap()),
            self.sys.cpus().len()
        );

        Paragraph::new(system_info_text)
            .style(Style::default().fg(Color::White))
            .render(block_inner_area, buf);

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
