pub fn format_duration(secs: i32) -> String {
    let res = if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{:.0}m", secs as f64 / 60 as f64)
    } else if secs < 86400 {
        format!("{:.0}h", secs as f64 / 3600 as f64)
    } else {
        format!("{:.0}d", secs as f64 / 86400 as f64)
    };
    res.to_string()
}
