//! Opt-in count observations independent of artifact capture and recorder retention.
use super::UiPerfCounter;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn record(counter: UiPerfCounter, value: f64) {
    if !matches!(
        counter,
        UiPerfCounter::WorkbenchLayoutPassCount
            | UiPerfCounter::FrameUpdateCount
            | UiPerfCounter::RuntimeTickCount
            | UiPerfCounter::RedrawFullFrame
            | UiPerfCounter::PresentCount
            | UiPerfCounter::CoalescedPresentCount
            | UiPerfCounter::ChromeSnapshotCount
            | UiPerfCounter::InputToDamageUs
            | UiPerfCounter::DamageToSubmitUs
            | UiPerfCounter::GpuUploadBytes
            | UiPerfCounter::GpuDrawCalls
            | UiPerfCounter::GpuRenderPasses
            | UiPerfCounter::GpuRetainedCacheCopyBytes
    ) {
        return;
    }
    static ENABLED: OnceLock<bool> = OnceLock::new();
    if !*ENABLED
        .get_or_init(|| std::env::var("ZIRCON_EDITOR_TRACE_UI_ACTIVITY").as_deref() == Ok("1"))
    {
        return;
    }
    let unix_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    // No timer, capture request, formatting buffer or retained per-frame allocation.
    eprintln!("[zircon_editor] ui_activity unix_ns={unix_ns} counter={counter:?} value={value}");
}
