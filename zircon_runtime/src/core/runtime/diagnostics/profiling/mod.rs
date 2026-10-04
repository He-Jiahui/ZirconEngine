//! Feature-gated CPU timeline recorder shared by runtime and editor hosts.

mod counter_hotspot;
mod export;
mod hotspot;
mod macros;
mod recorder;
mod scope;
#[cfg(feature = "profiling-tracy")]
mod tracy;
mod ui_hotspot;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
#[cfg(feature = "profiling")]
use std::{env, path::PathBuf};

pub use counter_hotspot::analyze_counter_hotspots;
pub use export::{ProfileExportError, ProfileExportReport, ProfileExportResult};
pub use hotspot::analyze_hotspots;
pub use recorder::{ProfileRecorder, ProfileRecorderStatus};
pub(crate) use scope::ProfileFrameContext;
pub use scope::{ProfileFrameScope, ProfileScope};
#[cfg(feature = "profiling-tracy")]
pub use tracy::{initialize_tracy_sink, TracyFrameScope, TracySinkStatus};
pub use ui_hotspot::analyze_ui_hotspots;
pub use zircon_runtime_interface::{
    CounterHotspotEntry, CounterHotspotReport, HotspotEntry, HotspotReport, ProfileCaptureConfig,
    ProfileControlCommand, ProfileControlRequest, ProfileControlResponse, ProfileCounterSnapshot,
    ProfileFrameSnapshot, ProfileSnapshot, ProfileSpanSnapshot, UiHotspotAlert, UiHotspotReport,
    UiScenarioHotspot, PROFILE_CAPTURE_MAX_COUNTERS, PROFILE_CAPTURE_MAX_FRAMES,
    PROFILE_CAPTURE_MAX_FRAME_BUDGET_MS, PROFILE_CAPTURE_MAX_SPANS, PROFILE_COUNTER_HOTSPOTS_FILE,
    PROFILE_DEFAULT_FRAME_BUDGET_MS, PROFILE_DEFAULT_MAX_COUNTERS, PROFILE_DEFAULT_MAX_FRAMES,
    PROFILE_DEFAULT_MAX_SPANS, PROFILE_DEFAULT_OUTPUT_ROOT, PROFILE_DEFAULT_SESSION_ID,
    PROFILE_HOTSPOTS_FILE, PROFILE_SUMMARY_FILE, PROFILE_TIMELINE_NATIVE_FILE,
    PROFILE_TIMELINE_PERFETTO_FILE, PROFILE_UI_HOTSPOTS_FILE,
};

pub use crate::{profile_counter, profile_dynamic_scope, profile_frame, profile_scope};

static GLOBAL_RECORDER: OnceLock<Mutex<ProfileRecorder>> = OnceLock::new();
const CAPTURE_ACTIVE_BIT: u64 = 1;
static CAPTURE_STATE: AtomicU64 = AtomicU64::new(0);

pub fn feature_enabled() -> bool {
    cfg!(feature = "profiling")
}

/// 开始新的进程级录制；重开会清空旧样本并切换 epoch，隔离前一轮的异步完成。
pub fn start_capture(config: ProfileCaptureConfig) -> ProfileRecorderStatus {
    if !feature_enabled() {
        return ProfileRecorderStatus::disabled();
    }
    let mut recorder = lock_recorder();
    let status = recorder.start_capture(config);
    if status.active {
        advance_capture_epoch(true);
    }
    drop(recorder);
    status
}

/// 停止接收新样本但保留缓冲；当前 epoch 中已开始的异步结果仍可完成。
pub fn stop_capture() -> ProfileRecorderStatus {
    if !feature_enabled() {
        return ProfileRecorderStatus::disabled();
    }
    let mut recorder = lock_recorder();
    let status = recorder.stop_capture();
    CAPTURE_STATE.fetch_and(!CAPTURE_ACTIVE_BIT, Ordering::AcqRel);
    drop(recorder);
    status
}

pub fn reset_capture() -> ProfileRecorderStatus {
    if !feature_enabled() {
        return ProfileRecorderStatus::disabled();
    }
    let mut recorder = lock_recorder();
    let status = recorder.reset();
    // Reset invalidates asynchronous work recorded by the preceding capture.
    advance_capture_epoch(false);
    drop(recorder);
    status
}

/// Cheap macro-facing hint that avoids touching the global recorder while idle.
#[doc(hidden)]
pub fn capture_active() -> bool {
    feature_enabled() && capture_state_is_active(CAPTURE_STATE.load(Ordering::Acquire))
}

/// Returns the current capture epoch for async producers, if a capture is live.
/// A completion from an older epoch must be discarded instead of contaminating
/// a later profiling session.
pub(crate) fn capture_epoch() -> Option<u64> {
    if !feature_enabled() {
        return None;
    }
    let state = CAPTURE_STATE.load(Ordering::Acquire);
    capture_state_is_active(state).then_some(capture_state_epoch(state))
}

/// Returns the epoch that may still accept a previously recorded async result.
/// Stopping seals the current epoch for completion; reset and start advance it.
pub(crate) fn capture_epoch_for_completion() -> Option<u64> {
    feature_enabled().then(|| capture_state_epoch(CAPTURE_STATE.load(Ordering::Acquire)))
}

fn advance_capture_epoch(active: bool) {
    CAPTURE_STATE
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |state| {
            let next_epoch = capture_state_epoch(state).wrapping_add(1);
            Some((next_epoch << 1) | u64::from(active))
        })
        .expect("capture state update is infallible");
}

const fn capture_state_epoch(state: u64) -> u64 {
    state >> 1
}

const fn capture_state_is_active(state: u64) -> bool {
    state & CAPTURE_ACTIVE_BIT != 0
}

pub fn snapshot() -> ProfileSnapshot {
    lock_recorder().snapshot()
}

pub fn export_report() -> ProfileExportResult<ProfileExportReport> {
    if !feature_enabled() {
        return Err(ProfileExportError::FeatureDisabled);
    }
    // 配置与样本在 recorder 锁内一起快照，文件写出在释放该锁后进行。
    let (snapshot, include_perfetto) = with_recorder(|recorder| {
        (
            recorder.snapshot(),
            recorder.config().include_perfetto && cfg!(feature = "profiling-chrome"),
        )
    });
    export::export_snapshot(&snapshot, include_perfetto)
}

pub fn start_capture_from_env(default_session_id: &str) -> Option<ProfileRecorderStatus> {
    if !feature_enabled() || !env_capture_enabled() {
        return None;
    }
    Some(start_capture(env_capture_config(default_session_id)))
}

pub fn stop_and_export_capture_from_env() -> Option<ProfileExportResult<ProfileExportReport>> {
    if !feature_enabled() || !env_capture_enabled() {
        return None;
    }
    stop_capture();
    Some(export_report())
}

pub fn control(request: ProfileControlRequest) -> ProfileControlResponse {
    match request.command {
        ProfileControlCommand::StartCapture => {
            let status = start_capture(request.config.unwrap_or_default());
            let mut response = ProfileControlResponse::ok(status.message);
            response.snapshot = Some(snapshot());
            response
        }
        ProfileControlCommand::StopCapture => {
            let status = stop_capture();
            let mut response = ProfileControlResponse::ok(status.message);
            response.snapshot = Some(snapshot());
            response
        }
        ProfileControlCommand::Snapshot => {
            let mut response = ProfileControlResponse::ok("profile snapshot captured");
            response.snapshot = Some(snapshot());
            response
        }
        ProfileControlCommand::RuntimeDiagnosticsSnapshot => {
            ProfileControlResponse::error("runtime diagnostics snapshot requires dynamic session")
        }
        ProfileControlCommand::RuntimeModuleCompositionReceipt => ProfileControlResponse::error(
            "runtime module composition receipt requires dynamic session",
        ),
        ProfileControlCommand::ExportReport => match export_report() {
            Ok(report) => {
                let mut response = ProfileControlResponse::ok("profile report exported");
                response.snapshot = Some(report.snapshot);
                response.hotspot_report = Some(report.hotspots);
                response.counter_hotspot_report = Some(report.counter_hotspots);
                response.ui_hotspot_report = Some(report.ui_hotspots);
                response.export_dir = Some(report.export_dir);
                response.files = report.files;
                response
            }
            Err(error) => ProfileControlResponse::error(error.to_string()),
        },
        ProfileControlCommand::Reset => {
            let status = reset_capture();
            let mut response = ProfileControlResponse::ok(status.message);
            response.snapshot = Some(snapshot());
            response
        }
    }
}

#[cfg(feature = "profiling")]
fn env_capture_enabled() -> bool {
    env::var("ZIRCON_PROFILE_CAPTURE")
        .map(|value| {
            matches!(
                value.as_str(),
                "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
            )
        })
        .unwrap_or(false)
}

#[cfg(not(feature = "profiling"))]
fn env_capture_enabled() -> bool {
    false
}

#[cfg(feature = "profiling")]
fn env_capture_config(default_session_id: &str) -> ProfileCaptureConfig {
    let mut config = ProfileCaptureConfig::default();
    if let Ok(session_id) = env::var("ZIRCON_PROFILE_SESSION") {
        config.session_id = session_id;
    } else {
        config.session_id = default_session_id.to_string();
    }
    if let Ok(output_root) = env::var("ZIRCON_PROFILE_OUTPUT_ROOT") {
        config.output_root = output_root;
    }
    if let Ok(max_frames) = env::var("ZIRCON_PROFILE_MAX_FRAMES") {
        config.max_frames = max_frames.parse().unwrap_or(config.max_frames);
    }
    if let Ok(max_spans) = env::var("ZIRCON_PROFILE_MAX_SPANS") {
        config.max_spans = max_spans.parse().unwrap_or(config.max_spans);
    }
    if let Ok(max_counters) = env::var("ZIRCON_PROFILE_MAX_COUNTERS") {
        config.max_counters = max_counters.parse().unwrap_or(config.max_counters);
    }
    if let Ok(frame_budget_ms) = env::var("ZIRCON_PROFILE_FRAME_BUDGET_MS") {
        config.frame_budget_ms = frame_budget_ms.parse().unwrap_or(config.frame_budget_ms);
    }
    if let Ok(include_perfetto) = env::var("ZIRCON_PROFILE_INCLUDE_PERFETTO") {
        config.include_perfetto = !matches!(include_perfetto.as_str(), "0" | "false" | "FALSE");
    }
    config.output_root = PathBuf::from(config.output_root)
        .to_string_lossy()
        .into_owned();
    config.normalized()
}

#[cfg(not(feature = "profiling"))]
fn env_capture_config(default_session_id: &str) -> ProfileCaptureConfig {
    let _ = default_session_id;
    ProfileCaptureConfig::default()
}

pub(crate) fn begin_scope(
    stream: &'static str,
    category: &'static str,
    name: &'static str,
) -> Option<scope::ProfileScopeToken> {
    if !capture_active() {
        return None;
    }
    begin_scope_named(stream, category, name.to_string())
}

pub(crate) fn begin_scope_named(
    stream: &'static str,
    category: &'static str,
    name: String,
) -> Option<scope::ProfileScopeToken> {
    let capture_epoch = capture_epoch()?;
    scope::begin_scope_named(capture_epoch, stream, category, name)
}

pub(crate) fn finish_scope(token: scope::ProfileScopeToken) {
    if feature_enabled() {
        scope::finish_scope(token);
    }
}

pub(crate) fn begin_frame(
    stream: &'static str,
    name: &'static str,
) -> Option<scope::ProfileFrameToken> {
    let capture_epoch = capture_epoch()?;
    scope::begin_frame(capture_epoch, stream, name)
}

pub(crate) fn finish_frame(token: scope::ProfileFrameToken) {
    if feature_enabled() {
        scope::finish_frame(token);
    }
}

pub fn record_counter(stream: &'static str, name: &'static str, value: f64) {
    let Some(capture_epoch) = capture_epoch() else {
        return;
    };
    scope::record_counter(capture_epoch, stream, name, value);
}

/// Records counters under one recorder lock and one shared timestamp.
#[doc(hidden)]
pub fn record_counter_batch(stream: &'static str, counters: &[(&'static str, f64)]) {
    #[cfg(feature = "profiling-tracy")]
    for &(name, value) in counters {
        tracing::info!(
            target: "zircon.profile.counter",
            stream = stream,
            name = name,
            value = value,
        );
    }
    let Some(capture_epoch) = capture_epoch() else {
        return;
    };
    if !counters.is_empty() {
        scope::record_counter_batch(capture_epoch, stream, counters);
    }
}

pub(crate) fn with_recorder<R>(action: impl FnOnce(&mut ProfileRecorder) -> R) -> R {
    let mut recorder = lock_recorder();
    action(&mut recorder)
}

fn lock_recorder() -> MutexGuard<'static, ProfileRecorder> {
    recorder()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn recorder() -> &'static Mutex<ProfileRecorder> {
    GLOBAL_RECORDER
        .get_or_init(|| Mutex::new(ProfileRecorder::new(ProfileCaptureConfig::default())))
}

#[cfg(test)]
pub(crate) fn test_capture_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
