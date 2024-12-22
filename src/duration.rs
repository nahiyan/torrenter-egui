pub fn format_duration<T>(secs: T) -> String
where
    T: Into<i64>,
{
    let secs = secs.into();
    if secs < 0 {
        "N/A".into()
    } else if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{:.0}m", secs as f64 / 60_f64)
    } else if secs < 86400 {
        format!("{:.0}h", secs as f64 / 3600_f64)
    } else {
        format!("{:.0}d", secs as f64 / 86400_f64)
    }
}
