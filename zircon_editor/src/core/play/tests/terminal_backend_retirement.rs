use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::{
    PlayBackend, PlayBackendPoll, PlayBackendRetireReport, PlayBackendStartFailure,
    PlayBackendStartReport, PlayBackendStopReport, PlayKind, PlayModeKind, PlaySessionController,
    PlaySessionError, PlayStartRequest, PluginBridgeActivation, PluginBridgeActivationReport,
};

#[derive(Default)]
struct PendingRetirementBackend {
    start_count: AtomicUsize,
    retire_count: AtomicUsize,
}

impl PlayBackend for PendingRetirementBackend {
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        self.start_count.fetch_add(1, Ordering::Relaxed);
        Ok(PlayBackendStartReport::default())
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        Ok(PlayBackendStopReport {
            diagnostics: Vec::new(),
            retirement_pending: true,
        })
    }

    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        self.retire_count.fetch_add(1, Ordering::Relaxed);
        Ok(PlayBackendRetireReport::default())
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        Ok(PlayBackendPoll::Running {
            diagnostics: Vec::new(),
        })
    }
}

#[derive(Default)]
struct CountingActivation {
    activate_count: AtomicUsize,
    deactivate_count: AtomicUsize,
    remaining_deactivate_failures: AtomicUsize,
}

impl PluginBridgeActivation for CountingActivation {
    fn activate(
        &self,
        _project_root: Option<&Path>,
    ) -> Result<PluginBridgeActivationReport, String> {
        self.activate_count.fetch_add(1, Ordering::Relaxed);
        Ok(PluginBridgeActivationReport::default())
    }

    fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
        self.deactivate_count.fetch_add(1, Ordering::Relaxed);
        let should_fail = self
            .remaining_deactivate_failures
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok();
        if should_fail {
            return Err("deactivate failed".to_string());
        }
        Ok(PluginBridgeActivationReport::default())
    }
}

struct StartFailureBackend;

impl PlayBackend for StartFailureBackend {
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        Err(PlayBackendStartFailure::new("start failed"))
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        Ok(PlayBackendStopReport::default())
    }

    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        Ok(PlayBackendRetireReport::default())
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        Ok(PlayBackendPoll::Running {
            diagnostics: Vec::new(),
        })
    }
}

#[derive(Default)]
struct StartCleanupPendingBackend {
    start_count: AtomicUsize,
    retire_count: AtomicUsize,
}

impl PlayBackend for StartCleanupPendingBackend {
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        self.start_count.fetch_add(1, Ordering::Relaxed);
        Err(PlayBackendStartFailure::retirement_pending(
            "start failed with cleanup pending",
        ))
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        Ok(PlayBackendStopReport::default())
    }

    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        self.retire_count.fetch_add(1, Ordering::Relaxed);
        Ok(PlayBackendRetireReport::default())
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        Ok(PlayBackendPoll::Running {
            diagnostics: Vec::new(),
        })
    }
}

#[test]
fn play_restart_waits_for_terminal_backend_retirement() {
    let backend = Arc::new(PendingRetirementBackend::default());
    let controller = PlaySessionController::new();
    controller.set_play_backend(backend.clone());
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("the first Play session should start");
    controller
        .request_stop()
        .expect("the backend should reach its terminal retirement phase");

    assert_eq!(controller.mode(), PlayModeKind::Edit);
    assert!(controller.terminal_backend_retirement_pending());
    assert!(matches!(
        controller.request_play(PlayStartRequest::immediate(PlayKind::Play, None)),
        Err(PlaySessionError::InvalidTransition {
            mode: PlayModeKind::Edit,
            event: "request_play_with_pending_backend_retirement",
        })
    ));
    assert_eq!(backend.start_count.load(Ordering::Relaxed), 1);

    controller
        .retire_terminal_backend()
        .expect("terminal backend retirement should release the old session owner");
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("a new Play session may start after terminal retirement");
    assert_eq!(backend.start_count.load(Ordering::Relaxed), 2);
}

#[test]
fn terminal_retirement_uses_the_backend_owned_by_the_stopped_session() {
    let stopped_backend = Arc::new(PendingRetirementBackend::default());
    let next_backend = Arc::new(PendingRetirementBackend::default());
    let controller = PlaySessionController::new();
    controller.set_play_backend(stopped_backend.clone());
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller.request_stop().unwrap();

    controller.set_play_backend(next_backend.clone());
    controller.retire_terminal_backend().unwrap();

    assert_eq!(stopped_backend.retire_count.load(Ordering::Relaxed), 1);
    assert_eq!(next_backend.retire_count.load(Ordering::Relaxed), 0);
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    assert_eq!(next_backend.start_count.load(Ordering::Relaxed), 1);
}

#[test]
fn active_session_deactivates_its_exact_plugin_owner_after_configuration_changes() {
    let active = Arc::new(CountingActivation::default());
    let next = Arc::new(CountingActivation::default());
    let controller = PlaySessionController::new();
    controller.set_plugin_activation(active.clone());
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();

    controller.set_plugin_activation(next.clone());
    controller.request_stop().unwrap();

    assert_eq!(active.deactivate_count.load(Ordering::Relaxed), 1);
    assert_eq!(next.deactivate_count.load(Ordering::Relaxed), 0);
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    assert_eq!(next.activate_count.load(Ordering::Relaxed), 1);
}

#[test]
fn failed_start_rollback_retains_the_exact_plugin_owner_for_cleanup_retry() {
    let active = Arc::new(CountingActivation {
        remaining_deactivate_failures: AtomicUsize::new(1),
        ..CountingActivation::default()
    });
    let replacement = Arc::new(CountingActivation::default());
    let controller = PlaySessionController::new();
    controller.set_plugin_activation(active.clone());
    controller.set_play_backend(Arc::new(StartFailureBackend));

    let error = controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect_err("the backend should fail after plugin activation");
    assert!(matches!(
        error,
        PlaySessionError::BackendStart {
            activation_rollback: Some(_),
            ..
        }
    ));
    assert_eq!(controller.mode(), PlayModeKind::CleanupFailed);

    controller.set_plugin_activation(replacement.clone());
    controller
        .request_stop()
        .expect("cleanup should retry the exact retained activation");

    assert_eq!(controller.mode(), PlayModeKind::Edit);
    assert_eq!(active.deactivate_count.load(Ordering::Relaxed), 2);
    assert_eq!(replacement.deactivate_count.load(Ordering::Relaxed), 0);
}

#[test]
fn failed_start_with_pending_cleanup_retires_the_exact_backend_before_restart() {
    let failed = Arc::new(StartCleanupPendingBackend::default());
    let replacement = Arc::new(PendingRetirementBackend::default());
    let controller = PlaySessionController::new();
    controller.set_play_backend(failed.clone());

    let error = controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect_err("startup should surface the backend failure");
    assert!(matches!(error, PlaySessionError::BackendStart { .. }));
    assert_eq!(controller.mode(), PlayModeKind::CleanupFailed);
    assert!(controller.terminal_backend_retirement_pending());

    controller.set_play_backend(replacement.clone());
    assert!(matches!(
        controller.request_play(PlayStartRequest::immediate(PlayKind::Play, None)),
        Err(PlaySessionError::InvalidTransition {
            mode: PlayModeKind::CleanupFailed,
            event: "request_play",
        })
    ));
    assert_eq!(replacement.start_count.load(Ordering::Relaxed), 0);

    controller
        .retire_terminal_backend()
        .expect("startup cleanup should retire the exact failed backend");
    assert_eq!(failed.retire_count.load(Ordering::Relaxed), 1);
    assert_eq!(controller.mode(), PlayModeKind::Edit);

    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("restart should proceed only after startup cleanup retirement");
    assert_eq!(replacement.start_count.load(Ordering::Relaxed), 1);
}
