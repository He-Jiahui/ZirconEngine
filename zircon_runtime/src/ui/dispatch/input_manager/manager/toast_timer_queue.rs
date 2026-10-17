//! 从通知队列值中借用通知标识并解析自动隐藏时长。
//!
//! # 生命周期与边界
//! 返回标识依附输入值存储；管理器在写入计时状态时才取得自己的所有权。
//! 数组递归选择首个有效项；映射和文本各自按下述别名优先级解析。

use zircon_runtime_interface::ui::component::UiValue;

/// 解析组件队列值，供通知计时器更新时使用。
/// # 参数与返回
/// 输入只借用；返回借用标识和正的毫秒时长，不支持的形状返回空值。
/// # 约束
/// 映射按固定顺序选标识及首个可转换时长；零时长会使该项无效，不再尝试较后别名。
pub(super) fn toast_timer_from_queue_value(value: &UiValue) -> Option<(&str, u64)> {
    match value {
        UiValue::Array(values) => values.iter().find_map(toast_timer_from_queue_value),
        UiValue::Map(values) => {
            let toast_id =
                first_string_value(values, &["id", "toast_id", "toastId", "value", "key"])?;
            let timeout_ms = first_u64_value(
                values,
                &[
                    "duration",
                    "duration_ms",
                    "auto_hide_duration_ms",
                    "autoHideDuration",
                ],
            )?;
            (timeout_ms > 0).then_some((toast_id, timeout_ms))
        }
        UiValue::String(value) | UiValue::Enum(value) => toast_timer_from_queue_string(value),
        _ => None,
    }
}

/// 文本首段为去空白标识；首个已知时长字段决定结果，格式错误或零值不会跳到后续字段。
fn toast_timer_from_queue_string(value: &str) -> Option<(&str, u64)> {
    let mut parts = value.split('|');
    let toast_id = parts.next()?.trim();
    if toast_id.is_empty() {
        return None;
    }

    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        if matches!(
            key.trim(),
            "duration" | "duration_ms" | "auto_hide_duration_ms" | "autoHideDuration"
        ) {
            let timeout_ms = value.trim().parse::<u64>().ok()?;
            return (timeout_ms > 0).then_some((toast_id, timeout_ms));
        }
    }
    None
}

/// 按别名优先级借用首个非空文本；不复制映射内存。
fn first_string_value<'a>(
    values: &'a std::collections::BTreeMap<String, UiValue>,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

/// 先选择首个可转换的别名，再检查正值；不是从所有别名中择任意正值。
fn first_u64_value(
    values: &std::collections::BTreeMap<String, UiValue>,
    keys: &[&str],
) -> Option<u64> {
    keys.iter()
        .find_map(|key| values.get(*key).and_then(u64_value))
        .filter(|value| *value > 0)
}

/// 仅借用文本与枚举载荷，保留输入值的存储期限。
fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.as_str()),
        _ => None,
    }
}

/// 数字负值按零处理，浮点先四舍五入；文本沿用无符号整数解析规则。
fn u64_value(value: &UiValue) -> Option<u64> {
    match value {
        UiValue::Int(value) => Some((*value).max(0) as u64),
        UiValue::Float(value) => Some((*value).round().max(0.0) as u64),
        UiValue::String(value) | UiValue::Enum(value) => value.parse::<u64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/toast_timer_queue.rs"]
mod tests;
