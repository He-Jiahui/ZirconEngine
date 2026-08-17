use std::sync::{Arc, Mutex};

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::{DefaultLevelManager, NodeKind, World};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::{ZrRuntimeApiV6, ZrRuntimeSessionHandle};

use crate::core::editing::authoring_world::EditorAuthoringWorld;
use crate::core::editing::intent::EditorIntent;
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, ModeMessage, PlayStateKind, SharedEditorMessageBus,
    TOPIC_MODE,
};
use crate::core::gateway::{
    EditorRuntimeGatewayHandle, GatewayError, RuntimeCapabilities, SessionGateway,
};
use crate::ui::host::{EditorHostEventController, EditorManager};
use crate::ui::workbench::state::EditorState;

use super::*;

#[derive(Default)]
struct RecordingActivation {
    calls: Mutex<Vec<&'static str>>,
}

struct FailingDeactivateActivation;

struct OrderedActivation {
    calls: Arc<Mutex<Vec<&'static str>>>,
}

struct OrderedBackend {
    calls: Arc<Mutex<Vec<&'static str>>>,
    start_error: Option<&'static str>,
    poll_exit_code: Option<Option<i32>>,
}

#[test]
fn play_gateway_attachment_preserves_authoring_world_access_across_detach() {
    let authoring_level = DefaultLevelManager::default().create_default_level();
    let authoring_gateway = EditorRuntimeGatewayHandle::detached();
    let _authoring_facade =
        EditorAuthoringWorld::loaded(&authoring_gateway, authoring_level.clone())
            .expect("the authoring facade should install the edit-domain gateway");
    let session_gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            ZrRuntimeApiV6::empty(),
            ZrRuntimeSessionHandle::new(1),
            RuntimeCapabilities::editor_default(),
            Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
        )
    }
    .expect("a valid session handle should construct a serialized gateway");
    let controller = PlaySessionController::new();

    let play_instance = controller
        .attach_play_gateway(Arc::new(session_gateway))
        .expect("a session transport should attach as a play-domain link");
    assert_eq!(
        controller.attached_world_domain(),
        Some(WorldDomain::Play(play_instance))
    );

    let mut authoring_generation = None;
    authoring_gateway
        .with_world(&mut |world| authoring_generation = Some(world.world_generation()))
        .expect("authoring facade must remain readable while play is attached");
    let mut authoring_entity = None;
    authoring_gateway
        .with_world_mut(&mut |world| authoring_entity = Some(world.spawn_node(NodeKind::Empty)))
        .expect("authoring facade must remain mutable while play is attached");

    let mut play_read = |_world: &World| {};
    let mut play_write = |_world: &mut World| {};
    let play_gateway = controller
        .play_gateway(play_instance)
        .expect("the attached play domain exposes its gateway");
    assert_eq!(
        play_gateway.with_world(&mut play_read),
        Err(GatewayError::RequiresSerializedAccess)
    );
    assert_eq!(
        play_gateway.with_world_mut(&mut play_write),
        Err(GatewayError::RequiresSerializedAccess)
    );

    controller
        .detach_play_gateway(play_instance)
        .expect("the attached play domain should detach by its instance identity");
    assert_eq!(controller.attached_world_domain(), None);
    assert!(controller.play_gateway(play_instance).is_none());

    let authoring_entity = authoring_entity.expect("authoring mutation should return its entity");
    assert!(authoring_level.with_world(|world| world.contains_entity(authoring_entity)));
    assert!(authoring_level.with_world(World::world_generation) > authoring_generation.unwrap());
}

#[test]
fn host_play_attachment_preserves_edit_selection_and_undo_history() {
    let core = CoreRuntime::new();
    let manager = Arc::new(
        EditorManager::new(&core.handle())
            .expect("the host test should construct an editor manager"),
    );
    let authoring_level = DefaultLevelManager::default().create_default_level();
    let state = EditorState::with_default_selection_with_context(
        authoring_level.clone(),
        UVec2::new(1280, 720),
        Arc::clone(manager.context()),
    );
    let controller = EditorHostEventController::new(state, manager);
    let selection_before = controller
        .shell()
        .lock()
        .state
        .viewport_controller
        .selection()
        .active_primary()
        .expect("the default authoring level should select a node");

    let created = {
        let mut shell = controller.shell().lock();
        assert!(shell
            .state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Empty))
            .expect("the edit domain should accept a scene command"));
        shell
            .state
            .viewport_controller
            .selection()
            .active_primary()
            .expect("the created node should be selected")
    };

    let session_gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            ZrRuntimeApiV6::empty(),
            ZrRuntimeSessionHandle::new(2),
            RuntimeCapabilities::editor_default(),
            Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
        )
    }
    .expect("a valid session handle should construct a serialized gateway");
    let play_instance = controller
        .attach_play_gateway(Arc::new(session_gateway))
        .expect("the host should attach only the play-domain gateway");

    let edit_gateway = controller
        .gateway_for(WorldDomain::Edit)
        .expect("the host must retain its stable edit gateway");
    let mut contains_created = false;
    edit_gateway
        .with_world(&mut |world| contains_created = world.contains_entity(created))
        .expect("the edit gateway must remain readable while play is attached");
    assert!(contains_created);

    let play_gateway = controller
        .gateway_for(WorldDomain::Play(play_instance))
        .expect("the host should route the attached play-domain gateway by instance");
    let mut play_read = |_world: &World| {};
    assert_eq!(
        play_gateway.with_world(&mut play_read),
        Err(GatewayError::RequiresSerializedAccess)
    );

    controller
        .detach_play_gateway(play_instance)
        .expect("detaching play must leave the edit facade in place");
    assert!(controller
        .gateway_for(WorldDomain::Play(play_instance))
        .is_none());

    {
        let mut shell = controller.shell().lock();
        assert!(shell
            .state
            .apply_intent(EditorIntent::Undo)
            .expect("undo after play detaches should use the preserved edit history"));
        assert_eq!(
            shell.state.viewport_controller.selection().active_primary(),
            Some(selection_before)
        );
    }
    assert!(!authoring_level.with_world(|world| world.contains_entity(created)));
}

impl PluginBridgeActivation for FailingDeactivateActivation {
    fn activate(
        &self,
        _project_root: Option<&std::path::Path>,
    ) -> Result<PluginBridgeActivationReport, String> {
        Ok(PluginBridgeActivationReport::default())
    }

    fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
        Err("deactivate failed".to_string())
    }
}

impl PluginBridgeActivation for RecordingActivation {
    fn activate(
        &self,
        _project_root: Option<&std::path::Path>,
    ) -> Result<PluginBridgeActivationReport, String> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push("activate");
        Ok(PluginBridgeActivationReport::default())
    }

    fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push("deactivate");
        Ok(PluginBridgeActivationReport::default())
    }
}

impl PluginBridgeActivation for OrderedActivation {
    fn activate(
        &self,
        _project_root: Option<&std::path::Path>,
    ) -> Result<PluginBridgeActivationReport, String> {
        self.calls.lock().unwrap().push("activation.start");
        Ok(PluginBridgeActivationReport::default())
    }

    fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
        self.calls.lock().unwrap().push("activation.stop");
        Ok(PluginBridgeActivationReport::default())
    }
}

impl PlayBackend for OrderedBackend {
    fn start(&self, _request: &PlayStartRequest) -> Result<PlayBackendStartReport, String> {
        self.calls.lock().unwrap().push("backend.start");
        match self.start_error {
            Some(error) => Err(error.to_string()),
            None => Ok(PlayBackendStartReport::default()),
        }
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        self.calls.lock().unwrap().push("backend.stop");
        Ok(PlayBackendStopReport::default())
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        self.calls.lock().unwrap().push("backend.poll");
        match self.poll_exit_code {
            Some(exit_code) => Ok(PlayBackendPoll::Exited {
                exit_code,
                diagnostics: vec!["process exited".to_string()],
            }),
            None => Ok(PlayBackendPoll::Running {
                diagnostics: Vec::new(),
            }),
        }
    }
}

#[test]
fn edit_request_play_without_build_enters_playing() {
    let controller = PlaySessionController::new();
    let activation = Arc::new(RecordingActivation::default());
    controller.set_plugin_activation(activation.clone());

    let report = controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("edit should enter playing");

    assert!(report.changed);
    assert_eq!(report.mode, PlayModeKind::Playing);
    assert_eq!(controller.mode(), PlayModeKind::Playing);
    assert_eq!(
        activation
            .calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_slice(),
        ["activate"]
    );
}

#[test]
fn edit_request_play_with_build_waits_for_build_result() {
    let controller = PlaySessionController::new();
    let activation = Arc::new(RecordingActivation::default());
    controller.set_plugin_activation(activation.clone());

    let waiting = controller
        .request_play(PlayStartRequest::after_build(PlayKind::Simulate, None))
        .expect("edit should enter building");
    assert_eq!(waiting.mode, PlayModeKind::Building);
    assert!(activation.calls.lock().unwrap().is_empty());

    let playing = controller
        .on_build_finished(true)
        .expect("successful build should enter playing");
    assert_eq!(playing.mode, PlayModeKind::Playing);
    assert_eq!(
        controller.mode_snapshot(),
        PlayMode::Playing {
            kind: PlayKind::Simulate
        }
    );
}

#[test]
fn failed_build_returns_to_edit_without_activation() {
    let controller = PlaySessionController::new();
    let activation = Arc::new(RecordingActivation::default());
    controller.set_plugin_activation(activation.clone());
    controller
        .request_play(PlayStartRequest::after_build(PlayKind::Play, None))
        .unwrap();

    let report = controller.on_build_finished(false).unwrap();

    assert_eq!(report.mode, PlayModeKind::Edit);
    assert_eq!(controller.mode(), PlayModeKind::Edit);
    assert!(activation.calls.lock().unwrap().is_empty());
}

#[test]
fn playing_rejects_second_play_request() {
    let controller = PlaySessionController::new();
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();

    assert!(matches!(
        controller.request_play(PlayStartRequest::immediate(PlayKind::Play, None)),
        Err(PlaySessionError::InvalidTransition {
            mode: PlayModeKind::Playing,
            event: "request_play"
        })
    ));
}

#[test]
fn stop_is_noop_in_edit_and_cancels_building() {
    let controller = PlaySessionController::new();
    assert!(!controller.request_stop().unwrap().changed);

    controller
        .request_play(PlayStartRequest::after_build(PlayKind::Play, None))
        .unwrap();
    let stopped = controller.request_stop().unwrap();

    assert!(stopped.changed);
    assert_eq!(stopped.mode, PlayModeKind::Edit);
}

#[test]
fn failed_deactivation_keeps_playing_for_cleanup_retry() {
    let controller = PlaySessionController::new();
    controller.set_plugin_activation(Arc::new(FailingDeactivateActivation));
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();

    let error = controller.request_stop().unwrap_err();

    assert!(matches!(error, PlaySessionError::PluginActivation(_)));
    assert_eq!(controller.mode(), PlayModeKind::Playing);
}

#[test]
fn native_plugin_activation_roundtrips_empty_live_host() {
    let activation = NativePluginBridgeActivation::new(
        zircon_runtime::plugin::native::NativePluginHostHandle::default(),
    );

    let entered = activation
        .activate(None)
        .expect("empty native live host should activate");
    assert!(entered
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("project root is unavailable")));

    let exited = activation
        .deactivate()
        .expect("empty native live host should deactivate");
    assert!(exited.is_clean());
}

#[test]
fn controller_orders_activation_backend_start_and_inverse_stop() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let controller = PlaySessionController::new();
    controller.set_plugin_activation(Arc::new(OrderedActivation {
        calls: calls.clone(),
    }));
    controller.set_play_backend(Arc::new(OrderedBackend {
        calls: calls.clone(),
        start_error: None,
        poll_exit_code: None,
    }));

    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller.request_stop().unwrap();

    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [
            "activation.start",
            "backend.start",
            "backend.stop",
            "activation.stop"
        ]
    );
}

#[test]
fn backend_start_failure_rolls_back_activation_and_keeps_edit_mode() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let controller = PlaySessionController::new();
    controller.set_plugin_activation(Arc::new(OrderedActivation {
        calls: calls.clone(),
    }));
    controller.set_play_backend(Arc::new(OrderedBackend {
        calls: calls.clone(),
        start_error: Some("spawn failed"),
        poll_exit_code: None,
    }));

    let error = controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap_err();

    assert!(matches!(error, PlaySessionError::BackendStart { .. }));
    assert_eq!(controller.mode(), PlayModeKind::Edit);
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        ["activation.start", "backend.start", "activation.stop"]
    );
}

#[test]
fn terminal_backend_poll_reports_crash_and_returns_to_edit() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let controller = PlaySessionController::new();
    controller.set_plugin_activation(Arc::new(OrderedActivation {
        calls: calls.clone(),
    }));
    controller.set_play_backend(Arc::new(OrderedBackend {
        calls: calls.clone(),
        start_error: None,
        poll_exit_code: Some(Some(101)),
    }));
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();

    let report = controller.poll_backend().unwrap();

    assert_eq!(report.mode, PlayModeKind::Edit);
    assert_eq!(
        report.cause,
        PlayTransitionCause::Crashed {
            exit_code: Some(101)
        }
    );
    assert_eq!(
        report.backend_diagnostics,
        vec!["process exited".to_string()]
    );
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [
            "activation.start",
            "backend.start",
            "backend.poll",
            "activation.stop"
        ]
    );
}

#[test]
fn controller_publishes_each_accepted_mode_boundary_without_noop_or_rejected_duplicates() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_MODE).expect("the built-in mode topic should be valid");
    let subscriber = bus
        .register_subscriber([topic.clone()])
        .expect("the mode subscriber should register");
    let controller = PlaySessionController::with_message_bus(bus.clone());

    assert!(!controller.request_stop().unwrap().changed);
    controller
        .request_play(PlayStartRequest::after_build(PlayKind::Play, None))
        .unwrap();
    assert!(
        !controller
            .request_play(PlayStartRequest::after_build(PlayKind::Play, None))
            .unwrap()
            .changed
    );
    controller.on_build_finished(true).unwrap();
    assert!(matches!(
        controller.request_play(PlayStartRequest::immediate(PlayKind::Play, None)),
        Err(PlaySessionError::InvalidTransition { .. })
    ));
    controller.request_stop().unwrap();

    let transitions = bus
        .drain_deliveries(subscriber)
        .into_iter()
        .map(|delivery| {
            assert_eq!(delivery.topic(), &topic);
            match delivery.message().payload() {
                EditorMessagePayload::Mode(ModeMessage::PlayStateChanged { from, to }) => {
                    (*from, *to)
                }
                payload => panic!("expected a typed mode transition, got {payload:?}"),
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        transitions,
        vec![
            (PlayStateKind::Edit, PlayStateKind::Building),
            (PlayStateKind::Building, PlayStateKind::Playing),
            (PlayStateKind::Playing, PlayStateKind::Edit),
        ]
    );
}

#[test]
fn controller_publishes_build_failure_and_backend_crash_after_committing_mode() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_MODE).expect("the built-in mode topic should be valid");
    let subscriber = bus
        .register_subscriber([topic.clone()])
        .expect("the mode subscriber should register");
    let controller = PlaySessionController::with_message_bus(bus.clone());
    controller.set_play_backend(Arc::new(OrderedBackend {
        calls: Arc::new(Mutex::new(Vec::new())),
        start_error: None,
        poll_exit_code: Some(Some(101)),
    }));

    controller
        .request_play(PlayStartRequest::after_build(PlayKind::Play, None))
        .unwrap();
    controller.on_build_finished(false).unwrap();
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller.poll_backend().unwrap();

    assert_eq!(controller.mode(), PlayModeKind::Edit);
    let transitions = bus
        .drain_deliveries(subscriber)
        .into_iter()
        .map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::Mode(ModeMessage::PlayStateChanged { from, to }) => (*from, *to),
            payload => panic!("expected a typed mode transition, got {payload:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        transitions,
        vec![
            (PlayStateKind::Edit, PlayStateKind::Building),
            (PlayStateKind::Building, PlayStateKind::Edit),
            (PlayStateKind::Edit, PlayStateKind::Playing),
            (PlayStateKind::Playing, PlayStateKind::Edit),
        ]
    );
}
