use std::collections::VecDeque;

pub(super) const MAX_OUTPUT_TAIL_LINES: usize = 512;

const OUTPUT_TRUNCATION_MARKER: &str =
    "[earlier output truncated; full log is available as an artifact]";

pub(super) fn push_bounded_output_line(lines: &mut VecDeque<String>, line: String) -> u64 {
    if lines.len() < MAX_OUTPUT_TAIL_LINES {
        lines.push_back(line);
        return 0;
    }

    let dropped = if lines
        .front()
        .is_some_and(|value| value == OUTPUT_TRUNCATION_MARKER)
    {
        let marker = lines
            .pop_front()
            .expect("a marked output tail must retain its marker");
        let _ = lines.pop_front();
        lines.push_front(marker);
        1
    } else {
        let _ = lines.pop_front();
        let _ = lines.pop_front();
        lines.push_front(OUTPUT_TRUNCATION_MARKER.to_string());
        2
    };
    lines.push_back(line);
    dropped
}

pub(super) fn retain_bounded_output_tail(lines: &mut VecDeque<String>) -> u64 {
    if lines.len() <= MAX_OUTPUT_TAIL_LINES {
        return 0;
    }

    let dropped = lines.len() - (MAX_OUTPUT_TAIL_LINES - 1);
    for _ in 0..dropped {
        let _ = lines.pop_front();
    }
    lines.push_front(OUTPUT_TRUNCATION_MARKER.to_string());
    dropped as u64
}

pub(super) fn retain_bounded_output_lines(lines: &mut Vec<String>) -> u64 {
    if lines.len() <= MAX_OUTPUT_TAIL_LINES {
        return 0;
    }

    let dropped = lines.len() - (MAX_OUTPUT_TAIL_LINES - 1);
    lines.drain(..dropped);
    lines.insert(0, OUTPUT_TRUNCATION_MARKER.to_string());
    dropped as u64
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::hint::black_box;
    use std::time::Instant;

    use super::{
        push_bounded_output_line, retain_bounded_output_lines, retain_bounded_output_tail,
        MAX_OUTPUT_TAIL_LINES, OUTPUT_TRUNCATION_MARKER,
    };

    #[test]
    fn tail_never_exceeds_limit() {
        let mut lines = VecDeque::new();
        for index in 0..(MAX_OUTPUT_TAIL_LINES * 3) {
            push_bounded_output_line(&mut lines, format!("line-{index}"));
        }

        assert_eq!(lines.len(), MAX_OUTPUT_TAIL_LINES);
        assert_eq!(lines.back().map(String::as_str), Some("line-1535"));
    }

    #[test]
    fn truncation_marker_is_retained() {
        let mut lines = (0..(MAX_OUTPUT_TAIL_LINES + 10))
            .map(|index| format!("line-{index}"))
            .collect::<VecDeque<_>>();
        let dropped = retain_bounded_output_tail(&mut lines);
        push_bounded_output_line(&mut lines, "last".to_string());

        assert_eq!(dropped, 11);
        assert_eq!(lines.len(), MAX_OUTPUT_TAIL_LINES);
        assert_eq!(
            lines.front().map(String::as_str),
            Some(OUTPUT_TRUNCATION_MARKER)
        );
        assert_eq!(lines.back().map(String::as_str), Some("last"));
    }

    #[test]
    fn terminal_vec_results_are_bounded_at_the_output_boundary() {
        let mut lines = (0..(MAX_OUTPUT_TAIL_LINES + 10))
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>();

        let dropped = retain_bounded_output_lines(&mut lines);

        assert_eq!(dropped, 11);
        assert_eq!(lines.len(), MAX_OUTPUT_TAIL_LINES);
        assert_eq!(
            lines.first().map(String::as_str),
            Some(OUTPUT_TRUNCATION_MARKER)
        );
        assert_eq!(lines.last().map(String::as_str), Some("line-521"));
    }

    fn legacy_retain_bounded_output_lines(lines: &mut Vec<String>) -> u64 {
        let mut tail = std::mem::take(lines).into_iter().collect::<VecDeque<_>>();
        let dropped = retain_bounded_output_tail(&mut tail);
        *lines = tail.into_iter().collect();
        dropped
    }

    #[test]
    fn optimization_batch_ep_output_line_tail_trims_in_place() {
        let mut optimized = (0..(MAX_OUTPUT_TAIL_LINES * 4))
            .map(|index| format!("line-{index}"))
            .collect::<Vec<_>>();
        let mut legacy = optimized.clone();
        let original_capacity = optimized.capacity();

        let optimized_dropped = retain_bounded_output_lines(&mut optimized);
        let legacy_dropped = legacy_retain_bounded_output_lines(&mut legacy);

        assert_eq!(optimized_dropped, legacy_dropped);
        assert_eq!(optimized, legacy);
        assert_eq!(optimized.capacity(), original_capacity);

        let source = include_str!("output_tail.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("output tail production implementation");
        let vec_boundary = production
            .split("fn retain_bounded_output_lines(")
            .nth(1)
            .expect("Vec output boundary");
        assert!(vec_boundary.contains("lines.drain(..dropped)"));
        assert!(!vec_boundary.contains("std::mem::take(lines)"));
        assert!(!vec_boundary.contains("collect::<VecDeque"));
    }

    #[test]
    #[ignore = "release-only in-place output tail benchmark"]
    fn optimization_batch_ep_in_place_output_tail_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const BATCHES_PER_SAMPLE: usize = 32;
        const SOURCE_LINE_COUNT: usize = 4_096;

        fn measure(seed: &[String], trim: fn(&mut Vec<String>) -> u64) -> u128 {
            let mut batches = (0..BATCHES_PER_SAMPLE)
                .map(|_| seed.to_vec())
                .collect::<Vec<_>>();
            let started = Instant::now();
            let mut checksum = 0_u64;
            for lines in &mut batches {
                checksum = checksum
                    .wrapping_add(trim(black_box(lines)))
                    .wrapping_add(lines.len() as u64);
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let seed = (0..SOURCE_LINE_COUNT)
            .map(|index| format!("captured-output-line-{index:05}"))
            .collect::<Vec<_>>();
        for _ in 0..4 {
            black_box(measure(&seed, legacy_retain_bounded_output_lines));
            black_box(measure(&seed, retain_bounded_output_lines));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure(&seed, legacy_retain_bounded_output_lines));
                optimized_samples.push(measure(&seed, retain_bounded_output_lines));
            } else {
                optimized_samples.push(measure(&seed, retain_bounded_output_lines));
                legacy_samples.push(measure(&seed, legacy_retain_bounded_output_lines));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "EDITOR377_IN_PLACE_OUTPUT_TAIL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             batches_per_sample={BATCHES_PER_SAMPLE} source_line_count={SOURCE_LINE_COUNT} \
             retained_line_count={MAX_OUTPUT_TAIL_LINES} pair_order=alternating_legacy_even \
             legacy_container_allocations_per_trim=2 optimized_container_allocations_per_trim=0 \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "in-place output tail trimming must reduce P95 by at least 30%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
