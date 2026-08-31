use std::sync::{Arc, Mutex};

use zircon_runtime::scene::{DefaultLevelManager, NodeKind, World};
use zircon_runtime_interface::{
    ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeApiV8, ZrRuntimeEventV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2, ZrRuntimeSessionHandle,
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
    ZrRuntimeViewportSizeV1, ZrStatus, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2,
};

use super::*;
use crate::core::editing::authoring_world::EditorAuthoringWorld;
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, ModeMessage, PlayStateKind, SharedEditorMessageBus,
    TOPIC_MODE,
};
use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError,
    GatewaySessionIdentity, RuntimeCapabilities, SessionGateway, SharedEditorRuntimeGateway,
};

#[derive(Default)]
struct RecordingActivation {
    calls: Mutex<Vec<&'static str>>,
}

struct FailOnceDeactivateActivation {
    calls: Arc<Mutex<Vec<&'static str>>>,
    remaining_failures: Mutex<usize>,
}

struct OrderedActivation {
    calls: Arc<Mutex<Vec<&'static str>>>,
}

struct OrderedBackend {
    calls: Arc<Mutex<Vec<&'static str>>>,
    start_error: Option<&'static str>,
    poll_exit_code: Option<Option<i32>>,
}

struct AttachableBackend;

struct PreviewFrameBackend;

struct RecordingInputBackend {
    events: Arc<Mutex<Vec<ZrRuntimeEventV1>>>,
}

struct RecordingInputGateway {
    events: Arc<Mutex<Vec<ZrRuntimeEventV1>>>,
}

static PREVIEW_FRAME_RGBA: [u8; 4] = [17, 34, 51, 255];
static PREVIEW_FRAME_CAPTURES: Mutex<Vec<(u64, u32, u32)>> = Mutex::new(Vec::new());
static PREVIEW_FRAME_RELEASES: Mutex<Vec<u64>> = Mutex::new(Vec::new());

impl PlayBackend for AttachableBackend {
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        Ok(PlayBackendStartReport::with_gateway(
            Vec::new(),
            Arc::new(DetachedEditorRuntimeGateway),
        ))
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

impl PlayBackend for PreviewFrameBackend {
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        let mut api = test_runtime_api();
        api.capture_frame = Some(capture_preview_frame);
        api.release_allocation = Some(release_preview_frame);
        let session = ZrRuntimeSessionHandle::new(41);
        let gateway = unsafe {
            SessionGateway::new_with_identity(
                Arc::new(()),
                api,
                session,
                GatewaySessionIdentity::new(41, session, 1, None),
                RuntimeCapabilities::editor_default(),
                Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
            )
        }
        .map_err(|error| PlayBackendStartFailure::new(error.to_string()))?;
        Ok(PlayBackendStartReport::with_gateway(
            Vec::new(),
            Arc::new(gateway),
        ))
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

impl PlayBackend for RecordingInputBackend {
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        Ok(PlayBackendStartReport::with_gateway(
            Vec::new(),
            Arc::new(RecordingInputGateway {
                events: Arc::clone(&self.events),
            }),
        ))
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

impl EditorRuntimeGateway for RecordingInputGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::new(51)
    }

    fn session_identity(&self) -> GatewaySessionIdentity {
        GatewaySessionIdentity::new(51, self.session_handle(), 1, None)
    }

    fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

struct FailOnceRetirementBackend {
    retire_attempts: Mutex<usize>,
}

impl PlayBackend for FailOnceRetirementBackend {
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        Ok(PlayBackendStartReport::default())
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        Ok(PlayBackendStopReport {
            diagnostics: Vec::new(),
            retirement_pending: true,
        })
    }

    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        let mut attempts = self.retire_attempts.lock().unwrap();
        *attempts += 1;
        if *attempts == 1 {
            Err("session owner is still referenced".to_string())
        } else {
            Ok(PlayBackendRetireReport::default())
        }
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        Ok(PlayBackendPoll::Running {
            diagnostics: Vec::new(),
        })
    }
}

unsafe extern "C" fn release_test_allocation(
    _session: ZrRuntimeSessionHandle,
    _allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn capture_preview_frame(
    _session: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
    frame: *mut ZrRuntimeFrameV2,
) -> ZrStatus {
    PREVIEW_FRAME_CAPTURES.lock().unwrap().push((
        request.viewport.raw(),
        request.size.width,
        request.size.height,
    ));
    unsafe {
        frame.write(ZrRuntimeFrameV2 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
            width: 1,
            height: 1,
            generation: 9,
            rgba: ZrOwnedResultV2 {
                data: PREVIEW_FRAME_RGBA.as_ptr(),
                len: PREVIEW_FRAME_RGBA.len() as u64,
                allocation: ZrRuntimeAllocationId::new(77),
            },
        });
    }
    ZrStatus::ok()
}

unsafe extern "C" fn release_preview_frame(
    _session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    PREVIEW_FRAME_RELEASES
        .lock()
        .unwrap()
        .push(allocation.raw());
    ZrStatus::ok()
}

unsafe extern "C" fn request_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeViewportPickRequestV1,
    _out_ticket: *mut ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn poll_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
    _out_result: *mut ZrRuntimeViewportPickResultV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn cancel_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

fn test_runtime_api() -> ZrRuntimeApiV8 {
    let mut api = ZrRuntimeApiV8::empty();
    api.release_allocation = Some(release_test_allocation);
    api.request_viewport_pick = Some(request_test_viewport_pick);
    api.poll_viewport_pick = Some(poll_test_viewport_pick);
    api.cancel_viewport_pick = Some(cancel_test_viewport_pick);
    api
}

fn start_test_play_gateway(
    controller: &PlaySessionController,
    gateway: SharedEditorRuntimeGateway,
) -> PlayInstanceId {
    controller.set_play_backend(Arc::new(TestAttachablePlayBackend::new(gateway)));
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("attachable test backend should start");
    match controller.attached_world_domain() {
        Some(WorldDomain::Play(instance)) => instance,
        domain => panic!("attachable test backend did not publish a Play domain: {domain:?}"),
    }
}

#[test]
fn play_preview_copies_the_default_viewport_frame_before_releasing_runtime_output() {
    PREVIEW_FRAME_CAPTURES.lock().unwrap().clear();
    PREVIEW_FRAME_RELEASES.lock().unwrap().clear();
    let controller = PlaySessionController::new();
    controller.set_play_backend(Arc::new(PreviewFrameBackend));
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("preview backend should enter Play");
    let Some(WorldDomain::Play(instance)) = controller.attached_world_domain() else {
        panic!("preview backend should attach its runtime gateway");
    };

    let frame = controller
        .capture_preview_frame(ZrRuntimeViewportSizeV1::new(320, 180))
        .expect("preview capture should succeed")
        .expect("Playing mode should return a frame");

    assert_eq!(frame.instance(), instance);
    assert_eq!(
        (frame.width(), frame.height(), frame.generation()),
        (1, 1, 9)
    );
    assert_eq!(frame.rgba().as_ref(), PREVIEW_FRAME_RGBA.as_slice());
    assert_eq!(
        PREVIEW_FRAME_CAPTURES.lock().unwrap().as_slice(),
        &[(
            zircon_runtime_interface::ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1.raw(),
            320,
            180
        )]
    );
    assert_eq!(PREVIEW_FRAME_RELEASES.lock().unwrap().as_slice(), &[77]);
}

#[test]
fn play_preview_input_routes_only_to_the_active_play_gateway() {
    let play_events = Arc::new(Mutex::new(Vec::new()));
    let play = PlaySessionController::new();
    play.set_play_backend(Arc::new(RecordingInputBackend {
        events: Arc::clone(&play_events),
    }));
    play.request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("Play input backend should start");
    let event = ZrRuntimeEventV1::pointer_moved(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        zircon_runtime_interface::ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
        12.0,
        24.0,
    );

    assert!(play
        .route_preview_input(event)
        .expect("active Play should route input"));
    let focus_lost = ZrRuntimeEventV1::lifecycle(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        zircon_runtime_interface::ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
        zircon_runtime_interface::ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    );
    assert!(play
        .route_preview_input(focus_lost)
        .expect("active Play should route focus lifecycle"));
    assert_eq!(play_events.lock().unwrap().as_slice(), &[event, focus_lost]);

    let simulate_events = Arc::new(Mutex::new(Vec::new()));
    let simulate = PlaySessionController::new();
    simulate.set_play_backend(Arc::new(RecordingInputBackend {
        events: Arc::clone(&simulate_events),
    }));
    simulate
        .request_play(PlayStartRequest::immediate(PlayKind::Simulate, None))
        .expect("Simulate input backend should start");

    assert!(!simulate
        .route_preview_input(event)
        .expect("Simulate should reject runtime input without error"));
    assert!(!simulate
        .route_preview_input(focus_lost)
        .expect("Simulate should reject focus lifecycle without error"));
    assert!(simulate_events.lock().unwrap().is_empty());
}

#[test]
fn play_gateway_attachment_preserves_authoring_world_access_across_detach() {
    let authoring_level = DefaultLevelManager::default().create_default_level();
    let authoring_gateway = EditorRuntimeGatewayHandle::detached();
    let _authoring_facade =
        EditorAuthoringWorld::loaded(&authoring_gateway, authoring_level.clone())
            .expect("the authoring facade should install the edit-domain gateway");
    let session_gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            test_runtime_api(),
            ZrRuntimeSessionHandle::new(1),
            GatewaySessionIdentity::new(1, ZrRuntimeSessionHandle::new(1), 1, None),
            RuntimeCapabilities::editor_default(),
            Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
        )
    }
    .expect("a valid session handle should construct a serialized gateway");
    let controller = PlaySessionController::new();

    let play_instance = start_test_play_gateway(&controller, Arc::new(session_gateway));
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
        .with_world_mut(&mut |world| {
            authoring_entity = Some(
                world
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed"),
            )
        })
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
        .request_stop()
        .expect("the owned test backend should stop before terminal detach");

    controller
        .detach_terminal_play_gateway(|instance| {
            assert_eq!(instance, play_instance);
            assert!(matches!(
                controller
                    .detach_terminal_play_gateway(|_| { Ok::<(), std::convert::Infallible>(()) }),
                Err(PlayTerminalGatewayDetachError::Domain(
                    PlayDomainLinkError::TerminalDetachInProgress
                ))
            ));
            assert!(matches!(
                controller.retire_terminal_backend(),
                Err(PlaySessionError::InvalidTransition {
                    event: "retire_terminal_backend_during_gateway_detach",
                    ..
                })
            ));
            Ok::<(), std::convert::Infallible>(())
        })
        .expect("the terminal play domain should detach by its instance identity");
    assert_eq!(controller.attached_world_domain(), None);
    assert!(controller.play_gateway(play_instance).is_none());

    let authoring_entity = authoring_entity.expect("authoring mutation should return its entity");
    assert!(authoring_level.with_world(|world| world.contains_entity(authoring_entity)));
    assert!(authoring_level.with_world(World::world_generation) > authoring_generation.unwrap());
}

impl PluginBridgeActivation for FailOnceDeactivateActivation {
    fn activate(
        &self,
        _project_root: Option<&std::path::Path>,
    ) -> Result<PluginBridgeActivationReport, String> {
        self.calls.lock().unwrap().push("activation.start");
        Ok(PluginBridgeActivationReport::default())
    }

    fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
        self.calls.lock().unwrap().push("activation.stop");
        let mut remaining = self.remaining_failures.lock().unwrap();
        if *remaining > 0 {
            *remaining -= 1;
            return Err("deactivate failed".to_string());
        }
        Ok(PluginBridgeActivationReport::default())
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
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        self.calls.lock().unwrap().push("backend.start");
        match self.start_error {
            Some(error) => Err(PlayBackendStartFailure::new(error)),
            None => Ok(PlayBackendStartReport::default()),
        }
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        self.calls.lock().unwrap().push("backend.stop");
        Ok(PlayBackendStopReport::default())
    }

    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        self.calls.lock().unwrap().push("backend.retire");
        Ok(PlayBackendRetireReport::default())
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
fn backend_start_gateway_is_attached_before_play_is_reported() {
    let controller = PlaySessionController::new();
    controller.set_play_backend(Arc::new(AttachableBackend));

    let report = controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("attachable backend should enter Play");

    assert!(report.backend_attachable);
    assert!(matches!(
        controller.attached_world_domain(),
        Some(WorldDomain::Play(_))
    ));
}

#[test]
fn terminal_backend_retirement_failure_remains_retryable() {
    let controller = PlaySessionController::new();
    controller.set_play_backend(Arc::new(FailOnceRetirementBackend {
        retire_attempts: Mutex::new(0),
    }));
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("test backend should start");
    controller.request_stop().expect("test backend should stop");

    let failed = controller
        .retire_terminal_backend()
        .expect("retirement failure should become retryable cleanup state");
    assert!(matches!(
        failed.cause,
        PlayTransitionCause::CleanupFailed {
            failure: PlayCleanupFailure::BackendRetirement { .. }
        }
    ));
    assert!(failed.changed);
    assert!(controller.terminal_backend_retirement_pending());

    let retired = controller
        .retire_terminal_backend()
        .expect("second retirement attempt should succeed");
    assert_eq!(retired.mode, PlayModeKind::Edit);
    assert!(!controller.terminal_backend_retirement_pending());
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
fn failed_deactivation_enters_cleanup_state_and_retries_without_re_stopping_backend() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_MODE).expect("the mode topic should be valid");
    let subscriber = bus
        .register_subscriber([topic.clone()])
        .expect("the mode subscriber should register");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let controller = PlaySessionController::with_message_bus(bus.clone());
    controller.set_plugin_activation(Arc::new(FailOnceDeactivateActivation {
        calls: calls.clone(),
        remaining_failures: Mutex::new(1),
    }));
    controller.set_play_backend(Arc::new(OrderedBackend {
        calls: calls.clone(),
        start_error: None,
        poll_exit_code: None,
    }));
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();

    let failed = controller.request_stop().unwrap();

    assert!(failed.changed);
    assert_eq!(failed.mode, PlayModeKind::CleanupFailed);
    assert_eq!(
        failed.cause,
        PlayTransitionCause::CleanupFailed {
            failure: PlayCleanupFailure::PluginDeactivation {
                message: "deactivate failed".to_string()
            }
        }
    );
    assert_eq!(
        controller.mode_snapshot(),
        PlayMode::CleanupFailed {
            kind: PlayKind::Play,
            failure: PlayCleanupFailure::PluginDeactivation {
                message: "deactivate failed".to_string()
            },
        }
    );

    let repaired = controller.request_stop().unwrap();

    assert!(repaired.changed);
    assert_eq!(repaired.mode, PlayModeKind::Edit);
    assert_eq!(controller.mode(), PlayModeKind::Edit);
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [
            "activation.start",
            "backend.start",
            "backend.stop",
            "activation.stop",
            "activation.stop",
        ]
    );
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
            (PlayStateKind::Edit, PlayStateKind::Playing),
            (PlayStateKind::Playing, PlayStateKind::CleanupFailed),
            (PlayStateKind::CleanupFailed, PlayStateKind::Edit),
        ]
    );
}

#[test]
fn terminal_backend_poll_never_leaves_an_exited_runtime_in_playing_mode() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let controller = PlaySessionController::new();
    controller.set_plugin_activation(Arc::new(FailOnceDeactivateActivation {
        calls: calls.clone(),
        remaining_failures: Mutex::new(1),
    }));
    controller.set_play_backend(Arc::new(OrderedBackend {
        calls: calls.clone(),
        start_error: None,
        poll_exit_code: Some(Some(101)),
    }));
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();

    let failed = controller.poll_backend().unwrap();

    assert!(failed.changed);
    assert_eq!(failed.mode, PlayModeKind::CleanupFailed);
    assert_eq!(controller.mode(), PlayModeKind::CleanupFailed);
    assert!(matches!(
        failed.cause,
        PlayTransitionCause::CleanupFailed {
            failure: PlayCleanupFailure::PluginDeactivation { .. }
        }
    ));

    let repaired = controller.request_stop().unwrap();

    assert_eq!(repaired.mode, PlayModeKind::Edit);
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [
            "activation.start",
            "backend.start",
            "backend.poll",
            "activation.stop",
            "activation.stop",
        ]
    );
}

#[test]
fn native_plugin_activation_roundtrips_empty_live_host() {
    let activation = NativePluginBridgeActivation::new(
        zircon_runtime::plugin::native::host::NativePluginHostHandle::default(),
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
fn build_completion_backend_start_failure_returns_to_edit_and_publishes_mode_boundary() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_MODE).expect("the built-in mode topic should be valid");
    let subscriber = bus
        .register_subscriber([topic.clone()])
        .expect("the mode subscriber should register");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let controller = PlaySessionController::with_message_bus(bus.clone());
    controller.set_plugin_activation(Arc::new(OrderedActivation {
        calls: calls.clone(),
    }));
    controller.set_play_backend(Arc::new(OrderedBackend {
        calls: calls.clone(),
        start_error: Some("spawn failed"),
        poll_exit_code: None,
    }));

    controller
        .request_play(PlayStartRequest::after_build(PlayKind::Play, None))
        .expect("the play request should enter building");
    let error = controller
        .on_build_finished(true)
        .expect_err("a failed backend start must reject the completed build");

    assert!(matches!(error, PlaySessionError::BackendStart { .. }));
    assert_eq!(controller.mode(), PlayModeKind::Edit);
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        ["activation.start", "backend.start", "activation.stop"]
    );
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
        ]
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

#[path = "tests/terminal_backend_retirement.rs"]
mod terminal_backend_retirement_tests;
