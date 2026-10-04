//! 提供视图模型共用的路径、大小和相对时间展示，避免页面另行解释底层数据。
//! 这些返回值是显示文本；执行路径继续使用原始路径、参数和时间戳。

use std::path::Path;

use crate::settings::HubLanguage;

use super::HubTextBundle;

pub(crate) const MILLIS_PER_MINUTE: u64 = 60_000;
pub(crate) const MILLIS_PER_HOUR: u64 = 60 * MILLIS_PER_MINUTE;
pub(crate) const MILLIS_PER_DAY: u64 = 24 * MILLIS_PER_HOUR;
pub(crate) const MILLIS_PER_WEEK: u64 = 7 * MILLIS_PER_DAY;

/// 用于只读展示；空路径呈现当前语言占位，不能将这个结果当作配置或执行路径。
pub(crate) fn path_text(path: &Path, language: HubLanguage) -> String {
    if path.as_os_str().is_empty() {
        return HubTextBundle::new(language)
            .pair("Not configured", "未配置")
            .to_string();
    }
    path.to_string_lossy().into_owned()
}

/// 供诊断路径字段沿用英文空值占位；正常路径的原始显示不随语言改变。
pub(crate) fn path_text_en(path: &Path) -> String {
    path_text(path, HubLanguage::English)
}

/// 将目录扫描的字节数变为紧凑显示，分档按二进制倍数；不能用于反推精确长度。
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

/// 把同一毫秒时间轴的历史完成时间投影成紧凑文案；时钟回拨/未来记录按刚刚显示。
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
#[path = "tests/display.rs"]
mod tests;
