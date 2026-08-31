use std::cell::Cell;
use std::time::{Duration, Instant};

const TEXT_ANALYSIS_PROFILE_COUNTER_NAMES: [&str; 11] = [
    "text_analysis_request_count",
    "text_analysis_request_input_bytes",
    "text_analysis_bidi_build_count",
    "text_analysis_bidi_input_bytes",
    "text_analysis_bidi_build_nanos",
    "text_analysis_script_emoji_build_count",
    "text_analysis_script_emoji_input_bytes",
    "text_analysis_script_emoji_build_nanos",
    "text_analysis_line_break_build_count",
    "text_analysis_line_break_input_bytes",
    "text_analysis_line_break_build_nanos",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TextAnalysisProfileMetrics {
    request_count: usize,
    request_input_bytes: usize,
    bidi_build_count: usize,
    bidi_input_bytes: usize,
    bidi_build_nanos: u64,
    script_emoji_build_count: usize,
    script_emoji_input_bytes: usize,
    script_emoji_build_nanos: u64,
    line_break_build_count: usize,
    line_break_input_bytes: usize,
    line_break_build_nanos: u64,
}

thread_local! {
    static TEXT_ANALYSIS_PROFILE_METRICS: Cell<Option<TextAnalysisProfileMetrics>> =
        const { Cell::new(None) };
}

pub(super) fn begin(input_bytes: usize) {
    begin_enabled(input_bytes, profile_metrics_enabled());
}

pub(super) fn finish() {
    let Some(metrics) = take() else {
        return;
    };
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[0],
        metrics.request_count
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[1],
        metrics.request_input_bytes
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[2],
        metrics.bidi_build_count
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[3],
        metrics.bidi_input_bytes
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[4],
        metrics.bidi_build_nanos
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[5],
        metrics.script_emoji_build_count
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[6],
        metrics.script_emoji_input_bytes
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[7],
        metrics.script_emoji_build_nanos
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[8],
        metrics.line_break_build_count
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[9],
        metrics.line_break_input_bytes
    );
    crate::profile_counter!(
        "runtime",
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES[10],
        metrics.line_break_build_nanos
    );
}

pub(super) fn start_build() -> Option<Instant> {
    TEXT_ANALYSIS_PROFILE_METRICS
        .with(|metrics| metrics.get().is_some())
        .then(Instant::now)
}

pub(super) fn record_bidi_build(input_bytes: usize, started: Option<Instant>) {
    let Some(started) = started else {
        return;
    };
    record_bidi_metrics(input_bytes, duration_to_nanos(started.elapsed()));
}

fn record_bidi_metrics(input_bytes: usize, elapsed_nanos: u64) {
    update(|metrics| {
        metrics.bidi_build_count = metrics.bidi_build_count.saturating_add(1);
        metrics.bidi_input_bytes = metrics.bidi_input_bytes.saturating_add(input_bytes);
        metrics.bidi_build_nanos = metrics.bidi_build_nanos.saturating_add(elapsed_nanos);
    });
}

pub(super) fn record_script_emoji_build(input_bytes: usize, started: Option<Instant>) {
    let Some(started) = started else {
        return;
    };
    record_script_emoji_metrics(input_bytes, duration_to_nanos(started.elapsed()));
}

fn record_script_emoji_metrics(input_bytes: usize, elapsed_nanos: u64) {
    update(|metrics| {
        metrics.script_emoji_build_count = metrics.script_emoji_build_count.saturating_add(1);
        metrics.script_emoji_input_bytes =
            metrics.script_emoji_input_bytes.saturating_add(input_bytes);
        metrics.script_emoji_build_nanos = metrics
            .script_emoji_build_nanos
            .saturating_add(elapsed_nanos);
    });
}

pub(super) fn record_line_break_build(input_bytes: usize, started: Option<Instant>) {
    let Some(started) = started else {
        return;
    };
    record_line_break_metrics(input_bytes, duration_to_nanos(started.elapsed()));
}

fn record_line_break_metrics(input_bytes: usize, elapsed_nanos: u64) {
    update(|metrics| {
        metrics.line_break_build_count = metrics.line_break_build_count.saturating_add(1);
        metrics.line_break_input_bytes = metrics.line_break_input_bytes.saturating_add(input_bytes);
        metrics.line_break_build_nanos =
            metrics.line_break_build_nanos.saturating_add(elapsed_nanos);
    });
}

fn begin_enabled(input_bytes: usize, enabled: bool) {
    TEXT_ANALYSIS_PROFILE_METRICS.with(|metrics| {
        metrics.set(enabled.then_some(TextAnalysisProfileMetrics {
            request_count: 1,
            request_input_bytes: input_bytes,
            ..TextAnalysisProfileMetrics::default()
        }));
    });
}

fn update(update: impl FnOnce(&mut TextAnalysisProfileMetrics)) {
    TEXT_ANALYSIS_PROFILE_METRICS.with(|metrics| {
        let Some(mut current) = metrics.get() else {
            return;
        };
        update(&mut current);
        metrics.set(Some(current));
    });
}

fn take() -> Option<TextAnalysisProfileMetrics> {
    TEXT_ANALYSIS_PROFILE_METRICS.with(|metrics| metrics.replace(None))
}

fn duration_to_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn profile_metrics_enabled() -> bool {
    #[cfg(feature = "profiling-tracy")]
    {
        return true;
    }
    #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
    {
        return crate::core::diagnostics::profiling::capture_active();
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TEXT_ANALYSIS_PROFILE_COUNTER_NAMES, begin_enabled, record_bidi_metrics,
        record_line_break_metrics, record_script_emoji_metrics, take,
    };
    use std::collections::HashSet;

    #[test]
    fn request_profile_distinguishes_duplicate_line_break_builds() {
        begin_enabled(12, true);
        record_bidi_metrics(12, 3);
        record_script_emoji_metrics(12, 5);
        record_line_break_metrics(12, 7);
        record_line_break_metrics(12, 11);

        let metrics = take().expect("enabled request profiling must retain one request aggregate");
        assert_eq!(metrics.request_count, 1);
        assert_eq!(metrics.request_input_bytes, 12);
        assert_eq!(metrics.bidi_build_count, 1);
        assert_eq!(metrics.bidi_input_bytes, 12);
        assert_eq!(metrics.bidi_build_nanos, 3);
        assert_eq!(metrics.script_emoji_build_count, 1);
        assert_eq!(metrics.script_emoji_input_bytes, 12);
        assert_eq!(metrics.script_emoji_build_nanos, 5);
        assert_eq!(metrics.line_break_build_count, 2);
        assert_eq!(metrics.line_break_input_bytes, 24);
        assert_eq!(metrics.line_break_build_nanos, 18);
        assert!(take().is_none(), "completion must detach the TLS aggregate");
    }

    #[test]
    fn analysis_profile_uses_only_fixed_request_names() {
        let unique = TEXT_ANALYSIS_PROFILE_COUNTER_NAMES
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(unique.len(), 11);
        assert!(
            unique.iter().all(|name| name.starts_with("text_analysis_")),
            "analysis profiling must use one fixed low-cardinality namespace"
        );
    }
}
