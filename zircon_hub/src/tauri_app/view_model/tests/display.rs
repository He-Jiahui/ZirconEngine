use std::path::Path;

use crate::settings::HubLanguage;

use super::*;

#[test]
fn empty_path_text_uses_current_language() {
    assert_eq!(
        path_text(Path::new(""), HubLanguage::English),
        "Not configured"
    );
    assert_eq!(path_text(Path::new(""), HubLanguage::Chinese), "未配置");
}

#[test]
fn relative_time_uses_compact_labels() {
    let now = 10 * MILLIS_PER_DAY;

    assert_eq!(relative_time(now, now, HubLanguage::English), "just now");
    assert_eq!(
        relative_time(now, now - (2 * MILLIS_PER_HOUR), HubLanguage::English),
        "2h ago"
    );
    assert_eq!(
        relative_time(now, now - (3 * MILLIS_PER_DAY), HubLanguage::English),
        "3d ago"
    );
    assert_eq!(relative_time(now, now, HubLanguage::Chinese), "刚刚");
    assert_eq!(
        relative_time(now, now - (2 * MILLIS_PER_HOUR), HubLanguage::Chinese),
        "2 小时前"
    );
    assert_eq!(
        relative_time(now, now - (3 * MILLIS_PER_DAY), HubLanguage::Chinese),
        "3 天前"
    );
}
