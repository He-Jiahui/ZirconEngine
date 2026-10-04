use super::super::kind::RuntimeForeignOutputKind;
use super::super::metrics::RuntimeForeignOutputMetricsSnapshot;

const DIAGNOSTIC_LINE_INITIAL_CAPACITY: usize = 4 * 1024;

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

#[inline]
fn push_metric(line: &mut String, label: &str, name: &str, value: u64) {
    line.push(' ');
    line.push_str(label);
    line.push('.');
    line.push_str(name);
    line.push('=');
    push_u64(line, value);
}

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
    line.push_str(unsafe { std::str::from_utf8_unchecked(&digits[cursor..]) });
}

#[cfg(test)]
#[path = "tests/diagnostic.rs"]
mod tests;
