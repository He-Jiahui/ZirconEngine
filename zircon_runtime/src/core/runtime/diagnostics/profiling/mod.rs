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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
#[cfg(feature = "profiling")]
use std::{env, path::PathBuf};

pub use counter_hotspot::analyze_counter_hotspots;
pub use export::{ProfileExportError, ProfileExportReport, ProfileExportResult};
pub use hotspot::analyze_hotspots;
pub use recorder::{ProfileRecorder, ProfileRecorderStatus};
pub use scope::{ProfileFrameScope, ProfileScope};
#[cfg(feature = "profiling-tracy")]
pub use tracy::{initialize_tracy_sink, TracyFrameScope, TracySinkStatus};
pub use ui_hotspot::analyze_ui_hotspots;
pub use zircon_runtime_interface::{
    CounterHotspotEntry, CounterHotspotReport, HotspotEntry, HotspotReport, ProfileCaptureConfig,
    ProfileControlCommand, ProfileControlRequest, ProfileControlResponse, ProfileCounterSnapshot,
    ProfileFrameSnapshot, ProfileSnapshot, ProfileSpanSnapshot, UiHotspotAlert, UiHotspotReport,
    UiScenarioHotspot, PROFILE_COUNTER_HOTSPOTS_FILE, PROFILE_DEFAULT_FRAME_BUDGET_MS,
    PROFILE_DEFAULT_MAX_COUNTERS, PROFILE_DEFAULT_MAX_FRAMES, PROFILE_DEFAULT_MAX_SPANS,
    PROFILE_DEFAULT_OUTPUT_ROOT, PROFILE_DEFAULT_SESSION_ID, PROFILE_HOTSPOTS_FILE,
    PROFILE_SUMMARY_FILE, PROFILE_TIMELINE_NATIVE_FILE, PROFILE_TIMELINE_PERFETTO_FILE,
    PROFILE_UI_HOTSPOTS_FILE,
};

pub use crate::{profile_counter, profile_dynamic_scope, profile_frame, profile_scope};

static GLOBAL_RECORDER: OnceLock<Mutex<ProfileRecorder>> = OnceLock::new();
static CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn feature_enabled() -> bool {
    cfg!(feature = "profiling")
}

pub fn start_capture(config: ProfileCaptureConfig) -> ProfileRecorderStatus {
    if !feature_enabled() {
        return ProfileRecorderStatus::disabled();
    }
    let status = lock_recorder().start_capture(config);
    CAPTURE_ACTIVE.store(status.active, Ordering::Release);
    status
}

pub fn stop_capture() -> ProfileRecorderStatus {
    if !feature_enabled() {
        return ProfileRecorderStatus::disabled();
    }
    let status = lock_recorder().stop_capture();
    CAPTURE_ACTIVE.store(false, Ordering::Release);
    status
}

pub fn reset_capture() -> ProfileRecorderStatus {
    if !feature_enabled() {
        return ProfileRecorderStatus::disabled();
    }
    let status = lock_recorder().reset();
    CAPTURE_ACTIVE.store(false, Ordering::Release);
    status
}

/// Cheap macro-facing hint that avoids touching the global recorder while idle.
#[doc(hidden)]
pub fn capture_active() -> bool {
    feature_enabled() && CAPTURE_ACTIVE.load(Ordering::Acquire)
}

pub fn snapshot() -> ProfileSnapshot {
    lock_recorder().snapshot()
}

pub fn export_report() -> ProfileExportResult<ProfileExportReport> {
    if !feature_enabled() {
        return Err(ProfileExportError::FeatureDisabled);
    }
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
    if !capture_active() {
        return None;
    }
    scope::begin_scope_named(stream, category, name)
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
    if !capture_active() {
        return None;
    }
    scope::begin_frame(stream, name)
}

pub(crate) fn finish_frame(token: scope::ProfileFrameToken) {
    if feature_enabled() {
        scope::finish_frame(token);
    }
}

pub fn record_counter(stream: &'static str, name: &'static str, value: f64) {
    if !capture_active() {
        return;
    }
    scope::record_counter(stream, name, value);
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
mod tests {
    use super::{snapshot, test_capture_lock, with_recorder};

    #[cfg(feature = "profiling")]
    use super::{reset_capture, start_capture, ProfileCaptureConfig};

    #[test]
    fn profile_recorder_accessors_recover_poisoned_global_lock() {
        let _guard = test_capture_lock();
        let poison_result = std::panic::catch_unwind(|| {
            let _recorder = super::lock_recorder();
            panic!("poison profile recorder lock");
        });
        assert!(poison_result.is_err());

        let snapshot_after_poison = snapshot();
        assert!(!snapshot_after_poison.active);

        let status = with_recorder(|recorder| recorder.reset());
        assert_eq!(status.message, "profile capture reset");
        assert!(!snapshot().active);
    }

    #[cfg(feature = "profiling")]
    #[test]
    fn profile_macros_capture_nested_spans_inside_frame() {
        let _guard = test_capture_lock();
        let mut config = ProfileCaptureConfig::default();
        config.session_id = "nested-span-test".to_string();
        config.max_frames = 4;
        config.max_spans = 8;
        start_capture(config);

        {
            crate::profile_frame!("runtime", "test_frame");
            {
                crate::profile_scope!("runtime", "test", "outer");
                {
                    crate::profile_scope!("runtime", "test", "inner");
                }
            }
        }

        let snapshot = snapshot();
        reset_capture();
        assert_eq!(snapshot.frames.len(), 1);
        assert_eq!(snapshot.spans.len(), 2);
        let outer = snapshot
            .spans
            .iter()
            .find(|span| span.name == "outer")
            .expect("outer span");
        let inner = snapshot
            .spans
            .iter()
            .find(|span| span.name == "inner")
            .expect("inner span");
        assert_eq!(outer.parent_id, None);
        assert_eq!(inner.parent_id, Some(outer.id));
        assert_eq!(inner.depth, 1);
        assert_eq!(inner.frame_index, Some(0));
    }

    #[cfg(feature = "profiling")]
    #[test]
    fn profile_scope_enter_named_captures_runtime_generated_names() {
        let _guard = test_capture_lock();
        let mut config = ProfileCaptureConfig::default();
        config.session_id = "dynamic-span-test".to_string();
        config.max_spans = 4;
        start_capture(config);

        {
            let pass_name = format!("{}-{}", "graph-pass", 7);
            let _scope =
                super::ProfileScope::enter_named("runtime", "render_graph.pass", pass_name);
        }

        let snapshot = snapshot();
        reset_capture();
        let span = snapshot
            .spans
            .iter()
            .find(|span| span.category == "render_graph.pass")
            .expect("dynamic render graph pass span");
        assert_eq!(span.name, "graph-pass-7");
        assert_eq!(span.path, "runtime/render_graph.pass:graph-pass-7");
    }

    #[cfg(feature = "profiling")]
    #[test]
    fn profile_dynamic_scope_macro_captures_runtime_generated_names() {
        let _guard = test_capture_lock();
        let mut config = ProfileCaptureConfig::default();
        config.session_id = "dynamic-macro-span-test".to_string();
        config.max_spans = 4;
        start_capture(config);

        {
            crate::profile_dynamic_scope!(
                "runtime",
                "render_graph.stage",
                format!("{:?}", crate::graphics::RenderPassStage::PostProcess),
            );
        }

        let snapshot = snapshot();
        reset_capture();
        let span = snapshot
            .spans
            .iter()
            .find(|span| span.category == "render_graph.stage")
            .expect("dynamic macro render graph stage span");
        assert_eq!(span.name, "PostProcess");
        assert_eq!(span.path, "runtime/render_graph.stage:PostProcess");
    }

    #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
    #[test]
    fn inactive_profile_macros_skip_dynamic_payload_evaluation() {
        let _guard = test_capture_lock();
        reset_capture();

        crate::profile_dynamic_scope!(
            "runtime",
            "test",
            panic!("inactive dynamic scope payload was evaluated"),
        );
        crate::profile_counter!(
            "runtime",
            "inactive.counter",
            panic!("inactive counter payload was evaluated"),
        );

        assert!(!super::capture_active());
    }

    #[cfg(not(feature = "profiling"))]
    #[test]
    fn disabled_profile_macros_do_not_evaluate_arguments() {
        crate::profile_scope!(panic!("stream"), panic!("category"), panic!("name"));
        crate::profile_dynamic_scope!(panic!("stream"), panic!("category"), panic!("name"));
        crate::profile_frame!(panic!("stream"), panic!("name"));
        crate::profile_counter!(panic!("stream"), panic!("name"), panic!("value"));

        assert!(!super::feature_enabled());
    }
}
