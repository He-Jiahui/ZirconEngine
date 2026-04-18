use std::time::Duration;

use super::now_unix_ms::now_unix_ms;

pub(crate) fn format_recent_project_time(last_opened_unix_ms: u64) -> String {
    if last_opened_unix_ms == 0 {
        return "Unknown".to_string();
    }
    let now = now_unix_ms();
    let delta_ms = now.saturating_sub(last_opened_unix_ms);
    let delta = Duration::from_millis(delta_ms);
    if delta < Duration::from_secs(60) {
        "Just now".to_string()
    } else if delta < Duration::from_secs(60 * 60) {
        format!("{}m ago", delta.as_secs() / 60)
    } else if delta < Duration::from_secs(60 * 60 * 24) {
        format!("{}h ago", delta.as_secs() / (60 * 60))
    } else {
        format!("{}d ago", delta.as_secs() / (60 * 60 * 24))
    }
}
