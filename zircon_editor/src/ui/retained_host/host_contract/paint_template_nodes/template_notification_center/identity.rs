use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_notification_center(
    node: &TemplatePaneNodeData,
) -> bool {
    node.role.as_str() == "NotificationCenter"
        || node.component_role.as_str() == "notification-center"
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn header_text(
    node: &TemplatePaneNodeData,
) -> String {
    let title = non_empty(node.text.as_str()).unwrap_or("Notifications");
    notification_header_text(
        title,
        node.notification_unread_count,
        node.notification_overflow_count,
    )
}

fn notification_header_text(title: &str, unread_count: usize, overflow_count: usize) -> String {
    if unread_count == 0 && overflow_count == 0 {
        return title.to_string();
    }
    let (unread_digits, unread_start) = if unread_count == 0 {
        ([0; 20], 20)
    } else {
        decimal_digits(unread_count)
    };
    let (overflow_digits, overflow_start) = if overflow_count == 0 {
        ([0; 20], 20)
    } else {
        decimal_digits(overflow_count)
    };
    let unread_digits = &unread_digits[unread_start..];
    let overflow_digits = &overflow_digits[overflow_start..];
    let capacity = title.len()
        + usize::from(unread_count != 0) * (" (".len() + unread_digits.len() + ")".len())
        + usize::from(overflow_count != 0)
            * (" +".len() + overflow_digits.len() + " omitted".len());
    let mut header = String::with_capacity(capacity);
    header.push_str(title);
    if unread_count != 0 {
        header.push_str(" (");
        push_ascii_digits(&mut header, unread_digits);
        header.push(')');
    }
    if overflow_count != 0 {
        header.push_str(" +");
        push_ascii_digits(&mut header, overflow_digits);
        header.push_str(" omitted");
    }
    header
}

fn decimal_digits(mut value: usize) -> ([u8; 20], usize) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    (digits, start)
}

fn push_ascii_digits(output: &mut String, digits: &[u8]) {
    for digit in digits {
        output.push(char::from(*digit));
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn empty_text(
    node: &TemplatePaneNodeData,
) -> String {
    non_empty(node.value_text.as_str())
        .unwrap_or("No notifications")
        .to_string()
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const HEADERS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn header_uses_generation_metadata_without_scanning_rows() {
        let node = TemplatePaneNodeData {
            text: "Notifications".into(),
            notification_unread_count: 3,
            notification_overflow_count: 12,
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(header_text(&node), "Notifications (3) +12 omitted");

        let source = include_str!("identity.rs");
        let row_collection = ["structured_", "options"].concat();
        let cloning_access = ["row_", "data"].concat();
        assert!(!source.contains(&row_collection));
        assert!(!source.contains(&cloning_access));
    }

    #[test]
    fn header_pixels_keep_the_existing_label_when_no_rows_were_dropped() {
        let unread = TemplatePaneNodeData {
            text: "Notifications".into(),
            notification_unread_count: 2,
            ..TemplatePaneNodeData::default()
        };
        let empty = TemplatePaneNodeData {
            text: "Notifications".into(),
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(header_text(&unread), "Notifications (2)");
        assert_eq!(header_text(&empty), "Notifications");
    }

    #[test]
    fn optimization_batch_ey_editor387_preserves_notification_header_bytes() {
        for (title, unread_count, overflow_count) in [
            ("Notifications", 0, 0),
            ("Notifications", 3, 0),
            ("Notifications", 0, 12),
            ("Build alerts", 37, 4_096),
            ("N", usize::MAX, usize::MAX),
        ] {
            assert_eq!(
                notification_header_text(title, unread_count, overflow_count),
                legacy_notification_header_text(title, unread_count, overflow_count)
            );
        }

        let production = include_str!("identity.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(!production.contains("format!("));
        assert!(production.contains("String::with_capacity(capacity)"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ey_editor387_direct_notification_header_benchmark() {
        for _ in 0..4 {
            black_box(measure_headers(legacy_notification_header_text));
            black_box(measure_headers(notification_header_text));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_headers(legacy_notification_header_text));
                optimized_samples.push(measure_headers(notification_header_text));
            } else {
                optimized_samples.push(measure_headers(notification_header_text));
                legacy_samples.push(measure_headers(legacy_notification_header_text));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn legacy_notification_header_text(
        title: &str,
        unread_count: usize,
        overflow_count: usize,
    ) -> String {
        match (unread_count, overflow_count) {
            (0, 0) => title.to_string(),
            (unread_count, 0) => format!("{title} ({unread_count})"),
            (0, overflow_count) => format!("{title} +{overflow_count} omitted"),
            (unread_count, overflow_count) => {
                format!("{title} ({unread_count}) +{overflow_count} omitted")
            }
        }
    }

    fn measure_headers(mut build: impl FnMut(&str, usize, usize) -> String) -> u128 {
        const TITLE: &str = "Notifications";
        let started = Instant::now();
        let mut total_len = 0_usize;
        for index in 0..HEADERS_PER_SAMPLE {
            let unread_count = black_box(1_000 + index % 97);
            let overflow_count = black_box(10_000 + index % 389);
            let header = build(black_box(TITLE), unread_count, overflow_count);
            total_len += black_box(header.len());
            black_box(header);
        }
        black_box(total_len);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR387_DIRECT_NOTIFICATION_HEADER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} headers_per_sample={HEADERS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
            "direct notification header construction must reduce P95 by at least 25%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
