use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    let hours = minutes / 60;
    let minutes = minutes % 60;
    if hours > 0 {
        return format!("{hours}h {minutes}m {seconds}s");
    }
    format!("{minutes}m {seconds}s")
}