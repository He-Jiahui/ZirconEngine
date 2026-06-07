use std::path::Path;

use crate::settings::HubLanguage;

use super::HubTextBundle;

pub(crate) const MILLIS_PER_MINUTE: u64 = 60_000;
pub(crate) const MILLIS_PER_HOUR: u64 = 60 * MILLIS_PER_MINUTE;
pub(crate) const MILLIS_PER_DAY: u64 = 24 * MILLIS_PER_HOUR;
pub(crate) const MILLIS_PER_WEEK: u64 = 7 * MILLIS_PER_DAY;

pub(crate) fn path_text(path: &Path, language: HubLanguage) -> String {
    if path.as_os_str().is_empty() {
        return HubTextBundle::new(language)
            .pair("Not configured", "未配置")
            .to_string();
    }
    path.to_string_lossy().into_owned()
}

pub(crate) fn path_text_en(path: &Path) -> String {
    path_text(path, HubLanguage::English)
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let bytes = bytes as f64;
    if bytes >= GIB {
        return format!("{:.1} GB", bytes / GIB);
    }
    if bytes >= MIB {
        return format!("{:.1} MB", bytes / MIB);
    }
    if bytes >= KIB {
        return format!("{:.1} KB", bytes / KIB);
    }
    format!("{} B", bytes as u64)
}

pub(crate) fn relative_time(now_ms: u64, then_ms: u64, language: HubLanguage) -> String {
    let elapsed = now_ms.saturating_sub(then_ms);
    if elapsed < MILLIS_PER_MINUTE {
        return match language {
            HubLanguage::English => "just now".to_string(),
            HubLanguage::Chinese => "刚刚".to_string(),
        };
    }
    if elapsed < MILLIS_PER_HOUR {
        let minutes = elapsed / MILLIS_PER_MINUTE;
        return match language {
            HubLanguage::English => format!("{minutes}m ago"),
            HubLanguage::Chinese => format!("{minutes} 分钟前"),
        };
    }
    if elapsed < MILLIS_PER_DAY {
        let hours = elapsed / MILLIS_PER_HOUR;
        return match language {
            HubLanguage::English => format!("{hours}h ago"),
            HubLanguage::Chinese => format!("{hours} 小时前"),
        };
    }
    if elapsed < MILLIS_PER_WEEK {
        let days = elapsed / MILLIS_PER_DAY;
        return match language {
            HubLanguage::English => format!("{days}d ago"),
            HubLanguage::Chinese => format!("{days} 天前"),
        };
    }
    let weeks = elapsed / MILLIS_PER_WEEK;
    match language {
        HubLanguage::English => format!("{weeks}w ago"),
        HubLanguage::Chinese => format!("{weeks} 周前"),
    }
}

#[cfg(test)]
mod tests {
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
}
