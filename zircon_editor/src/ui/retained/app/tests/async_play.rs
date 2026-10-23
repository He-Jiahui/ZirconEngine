use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use crate::core::editor_event::{EditorEventDispatcher, EditorEventSource};
use crate::core::play::*;

use super::*;

struct GatedBackend {
    entered: mpsc::SyncSender<&'static str>,
    proceed: Mutex<mpsc::Receiver<bool>>,
}

impl GatedBackend {
    fn gate(&self, stage: &'static str) -> bool {
        self.entered.send(stage).unwrap();
        self.proceed
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
    }
}

impl PlayBackend for GatedBackend {
    fn start(
        &self,
        _: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        if self.gate("start") {
            Ok(Default::default())
        } else {
            Err(PlayBackendStartFailure::new("controlled startup failure"))
        }
    }
    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        if self.gate("stop") {
            Ok(PlayBackendStopReport {
                retirement_pending: true,
                ..Default::default()
            })
        } else {
            Err("controlled stop failure".into())
        }
    }
    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        assert!(self.gate("retire"));
        Ok(Default::default())
    }
    fn poll(&self) -> Result<PlayBackendPoll, String> {
        Ok(PlayBackendPoll::Running {
            diagnostics: vec![],
        })
    }
}

fn current_world_harness(prefix: &str) -> ChildWindowHostHarness {
    let harness = ChildWindowHostHarness::new(prefix);
    let previous = harness.host.borrow().runtime.clone();
    // Native test-host startup replaces the manager's authoring gateway. Construct
    // this fixture's project state after that replacement so its lease is current.
    let mut state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
        previous.context().clone(),
    );
    state.mark_project_open();
    harness.host.borrow_mut().runtime = Arc::new(
        crate::ui::host::EditorHostEventController::new(state, previous.editor_manager_handle())
            .unwrap(),
    );
    harness.host.borrow_mut().refresh_ui();
    harness
}

fn setup(
    prefix: &str,
) -> (
    ChildWindowHostHarness,
    Arc<crate::ui::host::EditorHostEventController>,
    mpsc::Receiver<&'static str>,
    mpsc::SyncSender<bool>,
) {
    let harness = current_world_harness(prefix);
    let controller = harness.host.borrow().runtime.clone();
    let (entered_tx, entered) = mpsc::sync_channel(4);
    let (proceed, proceed_rx) = mpsc::sync_channel(4);
    controller.set_plugin_bridge_activation(Arc::new(NoopPluginBridgeActivation));
    controller.set_play_backend(Arc::new(GatedBackend {
        entered: entered_tx,
        proceed: Mutex::new(proceed_rx),
    }));
    controller.enable_async_play(Arc::new(|| {}));
    (harness, controller, entered, proceed)
}

fn dispatch(controller: &crate::ui::host::EditorHostEventController, action: MenuAction) {
    controller
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(action),
        )
        .unwrap();
}

fn completion(
    controller: &crate::ui::host::EditorHostEventController,
) -> Result<crate::core::editor_event::EditorEventRecord, String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(result) = controller.take_completed_play_command() {
            return result;
        }
        assert!(Instant::now() < deadline, "Play worker must complete");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn shell_is_available(controller: &Arc<crate::ui::host::EditorHostEventController>) {
    let controller = controller.clone();
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _shell = controller.shell().lock();
        tx.send(()).unwrap();
    });
    rx.recv_timeout(Duration::from_secs(1))
        .expect("backend work must release the shell lock");
}

#[test]
fn async_play_keeps_shell_available_during_start_stop_and_retirement() {
    let _guard = lock_env();
    let (harness, controller, entered, proceed) = setup("async-play-roundtrip");
    let admission = controller
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode),
        )
        .unwrap();
    assert!(admission.result.value.is_none());
    assert!(admission.result.error.is_none());
    assert!(!controller
        .journal()
        .records()
        .iter()
        .any(|record| record.event == EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode)));
    assert_eq!(
        entered.recv_timeout(Duration::from_secs(5)).unwrap(),
        "start"
    );
    assert_eq!(controller.play_sessions().mode(), PlayModeKind::Starting);
    shell_is_available(&controller);
    let started = Instant::now();
    harness.host.borrow_mut().tick();
    harness.host.borrow_mut().commit_interactive_frame_update();
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "native lifecycle must remain responsive while backend startup is blocked"
    );
    assert!(harness.host.borrow_mut().poll_async_play_command());
    assert!(harness
        .root_ui
        .get_host_presentation()
        .play_transition
        .as_deref()
        .unwrap()
        .contains("Starting"));
    assert!(harness
        .host
        .borrow()
        .workbench_window_bridge
        .control_bool_for_test("WorkbenchRunPlay", "disabled"));
    assert!(
        harness
            .root_ui
            .get_host_presentation()
            .workbench_window_nodes
            .iter()
            .find(|node| node.control_id.as_str() == "WorkbenchRunPlay")
            .unwrap()
            .disabled,
        "the committed native toolbar must reflect the transition"
    );
    assert!(controller
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode)
        )
        .is_err());
    proceed.send(true).unwrap();
    completion(&controller).unwrap();
    let play_records = controller
        .journal()
        .records()
        .iter()
        .filter(|record| record.event == EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(play_records.len(), 1, "only completed Play is replayable");
    assert!(play_records[0].result.value.is_some());
    assert!(play_records[0].result.error.is_none());
    assert_eq!(controller.play_sessions().mode(), PlayModeKind::Playing);
    dispatch(&controller, MenuAction::ExitPlayMode);
    assert_eq!(
        entered.recv_timeout(Duration::from_secs(5)).unwrap(),
        "stop"
    );
    assert_eq!(controller.play_sessions().mode(), PlayModeKind::Stopping);
    shell_is_available(&controller);
    assert_eq!(
        harness
            .host
            .borrow_mut()
            .native_main_window_close_requested(),
        crate::ui::retained_host::primitives::CloseRequestResponse::KeepWindowShown,
    );
    assert!(harness.host.borrow().close_after_play_transition);
    assert!(controller.pending_play_label().is_some());
    proceed.send(true).unwrap();
    assert_eq!(
        entered.recv_timeout(Duration::from_secs(5)).unwrap(),
        "retire"
    );
    assert_eq!(controller.play_sessions().mode(), PlayModeKind::Stopping);
    shell_is_available(&controller);
    proceed.send(true).unwrap();
    completion(&controller).unwrap();
    assert_eq!(controller.play_sessions().mode(), PlayModeKind::Edit);
    assert!(!controller.shell().lock().state.is_playing());
    assert!(!harness.host.borrow_mut().poll_async_play_command());
    assert!(!harness.host.borrow().close_after_play_transition);
}

#[test]
fn async_play_failure_restores_edit_state_and_allows_another_attempt() {
    let _guard = lock_env();
    let (_harness, controller, entered, proceed) = setup("async-play-failure");
    for _ in 0..2 {
        dispatch(&controller, MenuAction::EnterPlayMode);
        assert_eq!(
            entered.recv_timeout(Duration::from_secs(5)).unwrap(),
            "start"
        );
        proceed.send(false).unwrap();
        assert!(completion(&controller)
            .unwrap_err()
            .contains("controlled startup failure"));
        assert_eq!(controller.play_sessions().mode(), PlayModeKind::Edit);
        assert!(!controller.shell().lock().state.is_playing());
    }
}

#[test]
fn async_play_dispatch_latency_with_a_delayed_backend() {
    struct DelayedBackend;
    impl PlayBackend for DelayedBackend {
        fn start(
            &self,
            _: &PlayStartRequest,
        ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
            std::thread::sleep(Duration::from_millis(200));
            Ok(Default::default())
        }
        fn stop(&self) -> Result<PlayBackendStopReport, String> {
            Ok(Default::default())
        }
        fn retire(&self) -> Result<PlayBackendRetireReport, String> {
            Ok(Default::default())
        }
        fn poll(&self) -> Result<PlayBackendPoll, String> {
            Ok(PlayBackendPoll::Running {
                diagnostics: vec![],
            })
        }
    }
    let _guard = lock_env();
    let harness = current_world_harness("async-play-latency");
    let controller = harness.host.borrow().runtime.clone();
    controller.set_plugin_bridge_activation(Arc::new(NoopPluginBridgeActivation));
    controller.set_play_backend(Arc::new(DelayedBackend));
    let started = Instant::now();
    dispatch(&controller, MenuAction::EnterPlayMode);
    let synchronous = started.elapsed();
    dispatch(&controller, MenuAction::ExitPlayMode);
    controller.enable_async_play(Arc::new(|| {}));
    let started = Instant::now();
    dispatch(&controller, MenuAction::EnterPlayMode);
    let asynchronous = started.elapsed();
    completion(&controller).unwrap();
    dispatch(&controller, MenuAction::ExitPlayMode);
    completion(&controller).unwrap();
    eprintln!("interaction_latency backend_delay_ms=200 synchronous_dispatch_ms={:.3} async_dispatch_ms={:.3}",
        synchronous.as_secs_f64() * 1000.0, asynchronous.as_secs_f64() * 1000.0);
    assert!(synchronous >= Duration::from_millis(200));
    assert!(
        asynchronous < Duration::from_millis(100),
        "dispatch must return before the delayed backend"
    );
}
