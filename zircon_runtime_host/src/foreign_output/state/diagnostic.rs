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
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::super::super::kind::RUNTIME_FOREIGN_OUTPUT_KIND_COUNT;
    use super::super::super::metrics::RuntimeForeignOutputMetrics;
    use super::*;

    fn metrics_fixture() -> RuntimeForeignOutputMetricsSnapshot {
        RuntimeForeignOutputMetricsSnapshot {
            protocol_failed: true,
            protocol_failures: 3,
            blocked_session_calls: 5,
            by_kind: std::array::from_fn(|index| {
                let base = index as u64 + 1;
                RuntimeForeignOutputMetrics {
                    accepted_payloads: base,
                    accepted_bytes: base * 10,
                    rejected_payloads: base * 2,
                    rejected_bytes: base * 20,
                    call_failures: base * 3,
                    blocked_calls: base * 4,
                    total_decode_nanoseconds: base * 100,
                    max_decode_nanoseconds: base * 50,
                }
            }),
        }
    }

    fn legacy_render_diagnostic_line(
        metrics: RuntimeForeignOutputMetricsSnapshot,
    ) -> Option<String> {
        if !metrics.has_activity() {
            return None;
        }
        let mut fields = vec![
            format!("protocol_failed={}", metrics.protocol_failed),
            format!("protocol_failures={}", metrics.protocol_failures),
            format!("blocked_session_calls={}", metrics.blocked_session_calls),
        ];
        for kind in RuntimeForeignOutputKind::ALL {
            let counters = metrics.for_kind(kind);
            fields.push(format!(
                "{}.accepted_payloads={} {}.accepted_bytes={} {}.rejected_payloads={} {}.rejected_bytes={} {}.call_failures={} {}.blocked_calls={} {}.total_decode_ns={} {}.max_decode_ns={}",
                kind.label(), counters.accepted_payloads,
                kind.label(), counters.accepted_bytes,
                kind.label(), counters.rejected_payloads,
                kind.label(), counters.rejected_bytes,
                kind.label(), counters.call_failures,
                kind.label(), counters.blocked_calls,
                kind.label(), counters.total_decode_nanoseconds,
                kind.label(), counters.max_decode_nanoseconds,
            ));
        }
        Some(fields.join(" "))
    }

    #[test]
    fn single_buffer_diagnostic_matches_legacy_output() {
        let metrics = metrics_fixture();
        assert_eq!(
            render_diagnostic_line(metrics),
            legacy_render_diagnostic_line(metrics)
        );
        assert!(render_diagnostic_line(RuntimeForeignOutputMetricsSnapshot {
            protocol_failed: false,
            protocol_failures: 0,
            blocked_session_calls: 0,
            by_kind: [RuntimeForeignOutputMetrics::default(); RUNTIME_FOREIGN_OUTPUT_KIND_COUNT],
        })
        .is_none());
    }

    #[test]
    #[ignore = "release-only diagnostic render benchmark"]
    fn diagnostic_render_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 21;
        const RENDERS_PER_SAMPLE: usize = 10_000;

        fn measure_legacy(metrics: RuntimeForeignOutputMetricsSnapshot) -> u128 {
            let started = Instant::now();
            for _ in 0..RENDERS_PER_SAMPLE {
                black_box(legacy_render_diagnostic_line(black_box(metrics)).unwrap());
            }
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(metrics: RuntimeForeignOutputMetricsSnapshot) -> u128 {
            let started = Instant::now();
            for _ in 0..RENDERS_PER_SAMPLE {
                black_box(render_diagnostic_line(black_box(metrics)).unwrap());
            }
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

        let metrics = metrics_fixture();
        for _ in 0..4 {
            black_box(measure_legacy(metrics));
            black_box(measure_optimized(metrics));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(metrics));
                optimized_samples.push(measure_optimized(metrics));
            } else {
                optimized_samples.push(measure_optimized(metrics));
                legacy_samples.push(measure_legacy(metrics));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "RUNTIME_INTERFACE09_DIAGNOSTIC_RENDER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
renders_per_sample={RENDERS_PER_SAMPLE} output_kinds={RUNTIME_FOREIGN_OUTPUT_KIND_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_allocated_buffers=12 optimized_allocated_buffers=1 \
allocated_buffer_reduction_pct=91.667 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(40),
            "single-buffer diagnostic rendering must reduce P95 by at least 60%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
