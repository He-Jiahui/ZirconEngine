//! Runtime foreign-output 状态快照的紧凑诊断格式化。
//!
//! 仅在快照存在活动时分配一行文本；字段顺序对应 `RuntimeForeignOutputKind::ALL` 和
//! `RuntimeForeignOutputMetricsSnapshot::for_kind` 的稳定类别顺序。

use super::super::kind::RuntimeForeignOutputKind;
use super::super::metrics::RuntimeForeignOutputMetricsSnapshot;

const DIAGNOSTIC_LINE_INITIAL_CAPACITY: usize = 4 * 1024;

/// 有活动时将会话 fuse 与各输出类别计数序列化为一行 `key=value` 文本。
///
/// 调用者应传入一次 `RuntimeForeignOutputState::metrics` 快照；没有活动的快照返回 `None`，
/// 有活动时按固定字段顺序返回行字符串。
pub(super) fn render_diagnostic_line(
    metrics: RuntimeForeignOutputMetricsSnapshot,
) -> Option<String> {
    if !metrics.has_activity() {
        return None;
    }

    let mut line = String::with_capacity(DIAGNOSTIC_LINE_INITIAL_CAPACITY);
    line.push_str("protocol_failed=");
    line.push_str(if metrics.protocol_failed {
        "true"
    } else {
        "false"
    });
    line.push_str(" protocol_failures=");
    push_u64(&mut line, metrics.protocol_failures);
    line.push_str(" blocked_session_calls=");
    push_u64(&mut line, metrics.blocked_session_calls);
    for kind in RuntimeForeignOutputKind::ALL {
        let counters = metrics.for_kind(kind);
        let label = kind.label();
        push_metric(
            &mut line,
            label,
            "accepted_payloads",
            counters.accepted_payloads,
        );
        push_metric(&mut line, label, "accepted_bytes", counters.accepted_bytes);
        push_metric(
            &mut line,
            label,
            "rejected_payloads",
            counters.rejected_payloads,
        );
        push_metric(&mut line, label, "rejected_bytes", counters.rejected_bytes);
        push_metric(&mut line, label, "call_failures", counters.call_failures);
        push_metric(&mut line, label, "blocked_calls", counters.blocked_calls);
        push_metric(
            &mut line,
            label,
            "total_decode_ns",
            counters.total_decode_nanoseconds,
        );
        push_metric(
            &mut line,
            label,
            "max_decode_ns",
            counters.max_decode_nanoseconds,
        );
    }
    Some(line)
}

/// 追加一个类别指标字段；标签与字段名由调用者提供。
#[inline]
fn push_metric(line: &mut String, label: &str, name: &str, value: u64) {
    line.push(' ');
    line.push_str(label);
    line.push('.');
    line.push_str(name);
    line.push('=');
    push_u64(line, value);
}

/// 把 u64 写成十进制 ASCII 数字，不构造中间格式化字符串。
#[inline]
fn push_u64(line: &mut String, mut value: u64) {
    if value == 0 {
        line.push('0');
        return;
    }

    let mut digits = [0_u8; 20];
    let mut cursor = digits.len();
    while value != 0 {
        cursor -= 1;
        digits[cursor] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    // Every byte above is an ASCII decimal digit.
    // SAFETY: 上面的循环只写入 ASCII 数字；当前切片仅覆盖本次写入的十进制数字，因此是有效 UTF-8。
    line.push_str(unsafe { std::str::from_utf8_unchecked(&digits[cursor..]) });
}

#[cfg(test)]
#[path = "tests/diagnostic.rs"]
mod tests;
