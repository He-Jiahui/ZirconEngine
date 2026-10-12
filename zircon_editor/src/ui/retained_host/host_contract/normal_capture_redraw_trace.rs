//! Opt-in observations of the real normal-window redraw and projection owners.
use super::data::HostPresentationGenerationCursor;
use std::panic::Location;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime::core::framework::channel::ChannelWakeCallback;

const LOG_LIMIT: usize = 4_096;
static ENABLED: OnceLock<bool> = OnceLock::new();
static LOG_COUNT: AtomicUsize = AtomicUsize::new(0);

pub(in crate::ui::retained_host) fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        std::env::var("ZIRCON_EDITOR_TRACE_NORMAL_CAPTURE_REDRAW").as_deref() == Ok("1")
    })
}

#[track_caller]
pub(in crate::ui::retained_host) fn record(stage: &'static str, message: impl FnOnce() -> String) {
    if !enabled() {
        return;
    }
    let Ok(previous_count) =
        LOG_COUNT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
            (count < LOG_LIMIT).then_some(count + 1)
        })
    else {
        return;
    };
    let sequence = previous_count + 1;
    let caller = Location::caller();
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    eprintln!(
        "[zircon_editor] normal_capture_redraw_trace seq={sequence} unix_ms={unix_ms} pid={} stage={stage} caller={}:{} {}",
        std::process::id(), caller.file(), caller.line(), message(),
    );
}

pub(in crate::ui::retained_host) fn cursor_before(
    cursor: impl FnOnce() -> HostPresentationGenerationCursor,
) -> Option<HostPresentationGenerationCursor> {
    enabled().then(cursor)
}

#[track_caller]
pub(in crate::ui::retained_host) fn projection_commit(
    owner: usize,
    operation: &'static str,
    before: Option<HostPresentationGenerationCursor>,
    after: impl FnOnce() -> HostPresentationGenerationCursor,
) {
    let Some(before) = before else {
        return;
    };
    record("projection_commit", || {
        format!(
            "owner={owner:#x} operation={operation} before={before:?} after={:?}",
            after()
        )
    });
}

#[track_caller]
pub(in crate::ui::retained_host) fn wake_callback(
    source: &'static str,
    callback: ChannelWakeCallback,
) -> ChannelWakeCallback {
    if !enabled() {
        return callback;
    }
    let registered_at = Location::caller();
    Arc::new(move || {
        record("background_sender", || {
            format!("source={source} registered_at={registered_at}")
        });
        callback();
    })
}
