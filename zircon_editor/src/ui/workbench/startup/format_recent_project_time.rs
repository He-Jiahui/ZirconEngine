pub(crate) fn format_recent_project_time(last_opened_unix_ms: u64, now_unix_ms: u64) -> String {
    if last_opened_unix_ms == 0 {
        return "Unknown".to_string();
    }
    let delta_seconds = now_unix_ms.saturating_sub(last_opened_unix_ms) / 1_000;
    if delta_seconds < 60 {
        "Just now".to_string()
    } else if delta_seconds < 60 * 60 {
        elapsed_count_label(delta_seconds / 60, 'm')
    } else if delta_seconds < 60 * 60 * 24 {
        elapsed_count_label(delta_seconds / (60 * 60), 'h')
    } else {
        elapsed_count_label(delta_seconds / (60 * 60 * 24), 'd')
    }
}

fn elapsed_count_label(mut count: u64, unit: char) -> String {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (count % 10) as u8;
        count /= 10;
        if count == 0 {
            break;
        }
    }
    let digit_count = digits.len() - start;
    let mut label = String::with_capacity(digit_count + "m ago".len());
    for digit in &digits[start..] {
        label.push(char::from(*digit));
    }
    label.push(unit);
    label.push_str(" ago");
    label
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::format_recent_project_time;

    const SAMPLE_PAIRS: usize = 17;
    const LABELS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn recent_project_labels_share_the_snapshot_clock() {
        let now = 10 * 60 * 1_000;
        assert_eq!(format_recent_project_time(0, now), "Unknown");
        assert_eq!(format_recent_project_time(now - 30_000, now), "Just now");
        assert_eq!(format_recent_project_time(now - 120_000, now), "2m ago");
    }

    #[test]
    fn optimization_batch_ev_editor384_preserves_recent_time_boundaries() {
        let now = 400 * 24 * 60 * 60 * 1_000_u64;
        for delta_ms in [
            0,
            59_999,
            60_000,
            3_599_999,
            3_600_000,
            86_399_999,
            86_400_000,
            399 * 86_400_000,
        ] {
            let last_opened = now - delta_ms;
            assert_eq!(
                format_recent_project_time(last_opened, now),
                legacy_format_recent_project_time(last_opened, now)
            );
        }
        assert_eq!(
            format_recent_project_time(now + 1_000, now),
            legacy_format_recent_project_time(now + 1_000, now)
        );

        let production = include_str!("format_recent_project_time.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        assert!(!production.contains("format!("));
        assert!(production.contains("String::with_capacity"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ev_editor384_direct_recent_time_label_benchmark() {
        for _ in 0..4 {
            black_box(measure_labels(legacy_format_recent_project_time));
            black_box(measure_labels(format_recent_project_time));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_labels(legacy_format_recent_project_time));
                optimized_samples.push(measure_labels(format_recent_project_time));
            } else {
                optimized_samples.push(measure_labels(format_recent_project_time));
                legacy_samples.push(measure_labels(legacy_format_recent_project_time));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn legacy_format_recent_project_time(last_opened_unix_ms: u64, now_unix_ms: u64) -> String {
        if last_opened_unix_ms == 0 {
            return "Unknown".to_string();
        }
        let delta = Duration::from_millis(now_unix_ms.saturating_sub(last_opened_unix_ms));
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

    fn measure_labels(mut format_label: impl FnMut(u64, u64) -> String) -> u128 {
        const NOW_MS: u64 = 2_000_000_000_000;
        const DELTAS_MS: [u64; 3] = [37 * 60_000, 19 * 3_600_000, 937 * 86_400_000];

        let started = Instant::now();
        let mut total_len = 0_usize;
        for index in 0..LABELS_PER_SAMPLE {
            let delta_ms = black_box(DELTAS_MS[index % DELTAS_MS.len()] + index as u64 * 1_000);
            let label = format_label(NOW_MS - delta_ms, NOW_MS);
            total_len += black_box(label.len());
            black_box(label);
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
            "EDITOR384_DIRECT_RECENT_TIME_LABEL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} labels_per_sample={LABELS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
            "direct recent time label encoding must reduce P95 by at least 25%"
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
