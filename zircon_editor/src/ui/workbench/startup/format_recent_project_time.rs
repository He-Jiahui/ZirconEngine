use std::time::Duration;

pub(crate) fn format_recent_project_time(last_opened_unix_ms: u64, now_unix_ms: u64) -> String {
    if last_opened_unix_ms == 0 {
        return "Unknown".to_string();
    }
    let delta_ms = now_unix_ms.saturating_sub(last_opened_unix_ms);
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

#[cfg(test)]
mod performance_tests {
    use super::format_recent_project_time;

    #[test]
    fn recent_project_labels_share_the_snapshot_clock() {
        let now = 10 * 60 * 1_000;
        assert_eq!(format_recent_project_time(0, now), "Unknown");
        assert_eq!(format_recent_project_time(now - 30_000, now), "Just now");
        assert_eq!(format_recent_project_time(now - 120_000, now), "2m ago");
    }
}
