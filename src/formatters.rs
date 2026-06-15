use sysinfo::DiskUsage;

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;

    if b >= TB {
        format!("{:.1} TB", b / TB)
    } else if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_cpu(raw_percentage: f32) -> String {
    format!("{raw_percentage:.2} %")
}

pub fn format_disk_usage(raw_data: DiskUsage) -> String {
    let read = format_bytes(raw_data.total_read_bytes);
    let write = format_bytes(raw_data.total_written_bytes);
    format!("R:{read} W:{write}")
}

pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;

    match (days, hours, minutes) {
        (d, h, _) if d > 0 => format!("{d}d {h}h"),
        (_, h, m) if h > 0 => format!("{h}h {m}m"),
        (_, _, m) if m > 0 => format!("{m}m {secs}s"),
        _ => format!("{secs}s"),
    }
}
