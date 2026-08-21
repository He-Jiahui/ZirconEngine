use std::time::Duration;

use super::output::OutputSnapshot;
use super::rss::RssSnapshot;
use crate::diagnostic_log::DiagnosticLogSinkSnapshot;

const MAX_RSS_GROWTH_BYTES: u64 = 128 * 1024 * 1024;
const MAX_CALLER_P95: Duration = Duration::from_millis(50);

pub(super) struct CaseReport {
    logs_per_second: usize,
    caller_count: usize,
    scoped_rule_count: usize,
    sink_delay: Duration,
    caller_p95: Duration,
    load_elapsed: Duration,
    rss: RssSnapshot,
    sink: DiagnosticLogSinkSnapshot,
    output: OutputSnapshot,
}

impl CaseReport {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        logs_per_second: usize,
        caller_count: usize,
        scoped_rule_count: usize,
        sink_delay: Duration,
        caller_p95: Duration,
        load_elapsed: Duration,
        rss: RssSnapshot,
        sink: DiagnosticLogSinkSnapshot,
        output: OutputSnapshot,
    ) -> Self {
        Self {
            logs_per_second,
            caller_count,
            scoped_rule_count,
            sink_delay,
            caller_p95,
            load_elapsed,
            rss,
            sink,
            output,
        }
    }

    pub(super) fn print(&self) {
        println!(
            "PERF-MVP-434 requested_rate={} achieved_rate={:.2} attempted={} callers={} scoped_rules={} sink_delay_ms={} load_elapsed_ms={} caller_p95_us={} rss_baseline={} rss_peak={} rss_after={} rss_peak_growth={} rss_samples={} dequeued={} final_depth={} queue_peak={} queue_age_us={} dropped_debug={} critical_backpressure={} written={} batches={} output_writes={} output_flushes={} output_syncs={} wrong_thread_calls={} validated_records={} duplicate_records={} malformed_records={} closed={} output_errors={}",
            self.logs_per_second,
            self.logs_per_second as f64 / self.load_elapsed.as_secs_f64(),
            self.logs_per_second,
            self.caller_count,
            self.scoped_rule_count,
            self.sink_delay.as_millis(),
            self.load_elapsed.as_millis(),
            self.caller_p95.as_micros(),
            self.rss.baseline.unwrap_or(0),
            self.rss.peak.unwrap_or(0),
            self.rss.after.unwrap_or(0),
            self.rss.peak_growth().unwrap_or(0),
            self.rss.sample_count,
            self.sink.dequeued_records,
            self.sink.queue_depth,
            self.sink.max_queue_depth,
            self.sink.max_queue_age.as_micros(),
            self.sink.dropped_debug,
            self.sink.critical_backpressure_count,
            self.sink.written_records,
            self.sink.flush_batches,
            self.output.write_calls,
            self.output.flush_calls,
            self.output.sync_calls,
            self.output.wrong_thread_calls,
            self.output.validated_records,
            self.output.duplicate_records,
            self.output.malformed_records,
            self.sink.closed,
            self.sink.output_errors,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn assert_case(
    attempted: usize,
    formatted: usize,
    queue_capacity: usize,
    sink_delay: Duration,
    caller_p95: Duration,
    load_elapsed: Duration,
    sink: &DiagnosticLogSinkSnapshot,
    output: &OutputSnapshot,
    rss: RssSnapshot,
) {
    assert_eq!(formatted, attempted);
    assert_caller_latency(caller_p95);
    assert!(load_elapsed >= Duration::from_secs(1));
    assert!(load_elapsed <= Duration::from_millis(1_250));
    assert_eq!(sink.dequeued_records + sink.dropped_debug, attempted as u64);
    assert_eq!(sink.written_records, sink.dequeued_records);
    assert!(sink.max_queue_depth <= queue_capacity);
    assert_eq!(sink.queue_depth, 0);
    assert!(sink.closed);
    assert_eq!(sink.output_errors, 0);
    assert_eq!(output.wrong_thread_calls, 0);
    assert_eq!(output.write_calls, sink.flush_batches);
    assert_eq!(output.written_bytes, sink.written_bytes);
    assert_eq!(output.validated_records, sink.written_records);
    assert_eq!(output.duplicate_records, 0);
    assert_eq!(output.malformed_records, 0);
    assert!(output.flush_calls >= output.write_calls);
    assert_eq!(output.sync_calls, 1);
    if attempted <= 1_000 {
        assert_eq!(sink.dropped_debug, 0);
        assert_eq!(sink.written_records, attempted as u64);
    }
    if attempted == 100_000 && sink_delay == Duration::from_millis(100) {
        assert_eq!(sink.max_queue_depth, queue_capacity);
        assert!(sink.dropped_debug > 0);
    }
    if let Some(growth) = rss.peak_growth() {
        assert!(growth <= MAX_RSS_GROWTH_BYTES);
    }
    #[cfg(windows)]
    {
        assert!(rss.baseline.is_some() && rss.peak.is_some() && rss.after.is_some());
        assert!(rss.sample_count > 0);
    }
}

fn assert_caller_latency(caller_p95: Duration) {
    assert!(
        caller_p95 <= MAX_CALLER_P95,
        "caller P95 {:?} exceeded {:?} release budget",
        caller_p95,
        MAX_CALLER_P95
    );
}

pub(super) fn percentile_95(latencies: &[Duration]) -> Duration {
    let index = latencies
        .len()
        .saturating_mul(95)
        .div_ceil(100)
        .saturating_sub(1);
    latencies.get(index).copied().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{assert_caller_latency, MAX_CALLER_P95};
    use std::time::Duration;

    #[test]
    fn caller_latency_gate_accepts_the_budget_boundary() {
        assert_caller_latency(MAX_CALLER_P95);
    }

    #[test]
    #[should_panic(expected = "exceeded")]
    fn caller_latency_gate_rejects_an_over_budget_sample() {
        assert_caller_latency(MAX_CALLER_P95 + Duration::from_micros(1));
    }
}
