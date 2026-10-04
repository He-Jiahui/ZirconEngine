use std::cell::Cell;
use std::time::{Duration, Instant};

// 计数器名称固定且低基数；TLS 聚合只在一次请求结束时发布，避免按段落或文本内容生成动态键。
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

/// 建立请求级 TLS 聚合；Tracy 构建始终启用，profiling 构建仅在采集期间启用。
pub(super) fn begin(input_bytes: usize) {
    begin_enabled(input_bytes, profile_metrics_enabled());
}

/// 入口在后端返回 Result 后统一调用此函数；成功与失败都先取走并清空本次 TLS，再发布快照。
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
#[path = "tests/analysis_profile.rs"]
mod tests;
