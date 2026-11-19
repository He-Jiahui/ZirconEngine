// Winit event translation, retained routing, and a real V8 Play orbit. No HWND claim.
use std::path::Path;
use std::sync::Arc;

use winit::dpi::PhysicalPosition;
use winit::event::{MouseButton, PointerSource, WindowEvent};
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime_interface::math::Transform;
use zircon_runtime_interface::project::{
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource, ProjectTemplateId,
};
use zircon_runtime_interface::ui::layout::UiPoint;
use zircon_runtime_interface::ui::surface::UiPointerEventKind;
use zircon_runtime_interface::world_sync::{WorldQuery, WorldQueryResult};
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, RuntimeInputDiagnosticsSnapshot, ZrByteSlice,
    ZrRuntimeApiV8, ZrRuntimeSessionConfigV3, ZrRuntimeSessionHandle, ZrStatusCode,
};

use super::super::support::*;
use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewaySessionIdentity,
    RuntimeCapabilities, SessionGateway, SessionProfileKind, SharedEditorRuntimeGateway,
};
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::play::{
    EmbeddedPlayBackend, PlayKind, PlayMode, PlaySessionFactory, PlaySessionLaunchRequest,
    PlaySessionLease, PlaySessionRetireReport, WorldDomain,
};
use crate::core::project::ProjectAuthority;
use crate::core::settings::SETTINGS_USER_ROOT_ENV;
use crate::tests::editor_event::support::EnvironmentVariableRestoreGuard;
use crate::tests::support::test_project_creation_provenance;
use crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_level_viewport;

struct LinkedCorePlayFactory;

impl PlaySessionFactory for LinkedCorePlayFactory {
    fn create(
        &self,
        launch: &PlaySessionLaunchRequest,
    ) -> Result<Box<dyn PlaySessionLease>, String> {
        // The same static V8 table used by the existing real-runtime-ABI pump test.
        let api =
            unsafe { zircon_runtime::dynamic_api::zircon_runtime_get_api_v8(std::ptr::null()) };
        let api = *unsafe { api.as_ref() }.ok_or("fixture: real V8 API unavailable")?;
        let root = launch
            .project_root()
            .to_str()
            .ok_or("fixture: non-UTF-8 project path")?;
        let scene = launch.scene().as_str();
        let config = ZrRuntimeSessionConfigV3 {
            profile: ZrByteSlice::from_static(b"minimal"), // real ClientRuntime without render bridge
            project_root: ZrByteSlice {
                data: root.as_ptr(),
                len: root.len(),
            },
            play_scene: ZrByteSlice {
                data: scene.as_ptr(),
                len: scene.len(),
            },
            ..ZrRuntimeSessionConfigV3::empty()
        };
        let mut handle = ZrRuntimeSessionHandle::invalid();
        // Both borrowed slices stay valid across the synchronous create call.
        let status =
            unsafe { (api.create_session.ok_or("fixture: no V8 create")?)(config, &mut handle) };
        if status.status_code() != ZrStatusCode::Ok || !handle.is_valid() {
            return Err(format!(
                "fixture: create core-only V8 Play failed: {status:?}"
            ));
        }
        let owner = Arc::new(LinkedCoreSession { api, handle });
        // The ABI provider is linked for the process lifetime; the gateway also pins this session.
        let gateway = unsafe {
            SessionGateway::new_with_identity(
                owner.clone(),
                api,
                handle,
                GatewaySessionIdentity::new(1, handle, 1, Some(Arc::from(root))),
                RuntimeCapabilities::new(
                    SessionProfileKind::Minimal,
                    RuntimeCapabilities::editor_default()
                        .core_capabilities()
                        .iter()
                        .cloned(),
                    [],
                ),
                Arc::new(zircon_runtime_host::foreign_output::RuntimeForeignOutputState::default()),
            )
        }
        .map_err(|error| error.to_string())?;
        Ok(Box::new(LinkedCorePlayLease {
            gateway: Some(Arc::new(gateway)),
            owner,
        }))
    }
}

struct LinkedCorePlayLease {
    gateway: Option<SharedEditorRuntimeGateway>,
    owner: Arc<LinkedCoreSession>,
}

impl PlaySessionLease for LinkedCorePlayLease {
    fn gateway(&self) -> SharedEditorRuntimeGateway {
        self.gateway.as_ref().expect("live Play gateway").clone()
    }

    fn retire(&mut self) -> Result<PlaySessionRetireReport, String> {
        drop(self.gateway.take());
        Arc::get_mut(&mut self.owner)
            .ok_or("fixture: original Play still has gateway owners after detach")?
            .destroy()?;
        Ok(PlaySessionRetireReport::default())
    }
}

struct LinkedCoreSession {
    api: ZrRuntimeApiV8,
    handle: ZrRuntimeSessionHandle,
}

impl LinkedCoreSession {
    fn destroy(&mut self) -> Result<(), String> {
        if self.handle.is_valid() {
            let status =
                unsafe { (self.api.destroy_session.ok_or("fixture: no V8 destroy")?)(self.handle) };
            if status.status_code() != ZrStatusCode::Ok {
                return Err(format!("fixture: V8 Play destroy failed: {status:?}"));
            }
            self.handle = ZrRuntimeSessionHandle::invalid();
        }
        Ok(())
    }
}

impl Drop for LinkedCoreSession {
    fn drop(&mut self) {
        if let Err(error) = self.destroy() {
            eprintln!("{error}");
        }
    }
}

fn fixture_root() -> PathBuf {
    let requested = PathBuf::from(
        std::env::var_os("ZIRCON_PREVIEW168_TEST_ROOT")
            .expect("fixture: select a lane-owned directory on D/E/F cargo-targets"),
    );
    let resolved =
        ProjectPaths::resolve_existing(&requested).expect("fixture: physical output root");
    let parent = resolved.display_path();
    assert!(
        ["D:/cargo-targets", "E:/cargo-targets", "F:/cargo-targets"]
            .iter()
            .any(|root| parent.starts_with(*root)),
        "fixture: invalid physical root {parent:?}"
    );
    let tag = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = resolved
        .operation_path()
        .join(format!("p168-{}-{tag}", std::process::id()));
    fs::create_dir(&root).expect("fixture: create fresh evidence directory");
    root
}

fn cold_project_host(
    core: &CoreRuntime,
    ui: &UiHostWindow,
    root: &Path,
) -> Rc<RefCell<RetainedEditorHost>> {
    // Same module closure as the child-window fixture, but never build Welcome or publish a spare Edit world.
    for engine_module in zircon_runtime::builtin::runtime_core_modules() {
        core.register_module(engine_module.descriptor()).unwrap();
    }
    core.register_module(zircon_runtime::ui::module_descriptor())
        .unwrap();
    core.register_module(module::module_descriptor()).unwrap();
    for name in [
        FOUNDATION_MODULE_NAME,
        zircon_runtime::asset::ASSET_MODULE_NAME,
        zircon_runtime::core::framework::scene::SCENE_MODULE_NAME,
        module::EDITOR_MODULE_NAME,
    ] {
        core.activate_module(name).unwrap();
    }
    let provenance = test_project_creation_provenance();
    let intent = ProjectLaunchIntent::create_project(
        provenance.operation_id(),
        ProjectLaunchSource::Cli,
        ProjectLaunchProfile::Normal,
        "Preview168",
        ProjectPaths::display_path(root),
        ProjectTemplateId::RenderableEmpty,
    )
    .expect("fixture: project intent");
    let startup = EditorGuiStartupRequest::project(intent)
        .preflight_project(&ProjectAuthority::default())
        .expect("fixture: project preflight");
    let mut retained = RetainedEditorHost::new(
        core.handle(),
        Arc::new(DetachedEditorRuntimeGateway),
        ui.clone_strong(),
        Some(startup),
        Some(provenance.build_set_id().clone()),
        None,
    )
    .expect("fixture: cold host and scene document admission");
    // CPU-only editor presenter seam; the distinct Play gateway below is not detached or stubbed.
    let viewport = RetainedViewportController::new_test_stub();
    viewport.bind_jobs(retained.runtime.context().jobs().clone());
    ui.set_runtime_presenter_factory(viewport.runtime_presenter_factory());
    retained.viewport = viewport;
    retained
        .runtime
        .set_play_backend(Arc::new(EmbeddedPlayBackend::new(Arc::new(
            LinkedCorePlayFactory,
        ))));
    let host = Rc::new(RefCell::new(retained));
    wire_callbacks(ui, &host, None);
    host.borrow_mut().self_handle = Some(Rc::downgrade(&host));
    host.borrow_mut().refresh_ui();
    host
}

fn menu(host: &mut RetainedEditorHost, action: &str) {
    let effects = callback_dispatch::dispatch_menu_action(&host.runtime, action)
        .unwrap_or_else(|error| panic!("fixture: {action}: {error}"));
    host.apply_dispatch_effects(effects);
    host.refresh_ui();
}

fn camera(
    gateway: &EditorRuntimeGatewayHandle,
    identity: &GatewaySessionIdentity,
    entity: u64,
) -> (u64, Transform) {
    match gateway
        .query_world_at_identity(identity, WorldQuery::transform_snapshot(entity))
        .unwrap()
    {
        WorldQueryResult::TransformSnapshot {
            world_replacement_epoch,
            transform,
            ..
        } => (world_replacement_epoch, transform),
        other => panic!("fixture: project camera unavailable: {other:?}"),
    }
}

fn play_input(
    gateway: &EditorRuntimeGatewayHandle,
    identity: &GatewaySessionIdentity,
) -> RuntimeInputDiagnosticsSnapshot {
    assert_eq!(&gateway.identity(), identity);
    let response = gateway
        .profile_control(&ProfileControlRequest {
            command: ProfileControlCommand::RuntimeDiagnosticsSnapshot,
            config: None,
        })
        .unwrap()
        .expect("fixture: real V8 diagnostics response");
    assert_eq!(&gateway.identity(), identity);
    assert_eq!(response.status, "ok", "{}", response.message);
    let diagnostics = response
        .runtime_diagnostics
        .expect("fixture: no runtime diagnostics");
    assert_eq!(diagnostics.project_identity.as_deref(), Some("Preview168"));
    assert!(diagnostics
        .scene_uri
        .as_deref()
        .is_some_and(|uri| uri.starts_with(".zircon/play/")));
    diagnostics.input
}

#[test]
#[ignore = "isolated native Editor lane; requires ZIRCON_PREVIEW168_TEST_ROOT"]
fn game_drag_release_outside_clip_reaches_original_play() {
    use UiPointerEventKind::{Down, Move, Up};
    let _lock = lock_env();
    let root = fixture_root();
    let _env = [
        "ZIRCON_CONFIG_PATH",
        "ZIRCON_LOG_ROOT",
        "USERPROFILE",
        SETTINGS_USER_ROOT_ENV,
    ]
    .map(EnvironmentVariableRestoreGuard::capture);
    std::env::set_var("ZIRCON_CONFIG_PATH", root.join("config.json"));
    std::env::set_var("ZIRCON_LOG_ROOT", root.join("log"));
    std::env::set_var("USERPROFILE", &root);
    std::env::set_var(SETTINGS_USER_ROOT_ENV, root.join("settings"));
    let core = CoreRuntime::new();
    let ui = UiHostWindow::new().expect("fixture: retained window contract");
    ui.window().set_size(PhysicalSize::new(1280, 800));
    ui.window().set_scale_factor(1.0);
    ui.show().unwrap();
    let host = cold_project_host(&core, &ui, &root);
    let edit = host
        .borrow()
        .runtime
        .gateway_for(WorldDomain::Edit)
        .unwrap();
    let edit_identity = edit.identity();
    let entity = host
        .borrow()
        .runtime
        .project_scene_snapshot()
        .expect("fixture: current Edit gateway")
        .expect("fixture: admitted project scene")
        .active_camera();
    let authoring_before = camera(&edit, &edit_identity, entity);
    menu(&mut host.borrow_mut(), "workbench.play_mode.enter");
    let runtime = host.borrow().runtime.clone();
    assert!(matches!(
        runtime.play_sessions().mode_snapshot(),
        PlayMode::Playing {
            kind: PlayKind::Play
        }
    ));
    let Some(domain @ WorldDomain::Play(instance)) =
        runtime.play_sessions().attached_world_domain()
    else {
        panic!("fixture: no genuine Play owner attached");
    };
    let play = runtime
        .gateway_for(domain)
        .expect("fixture: no Play gateway");
    let identity = play.identity();
    assert!(identity.runtime_session().is_valid());
    assert_eq!(identity.play_instance(), Some(instance.raw()));
    let read_camera = || camera(&play, &identity, entity);
    let read_input = || play_input(&play, &identity);

    let presentation = ui.get_host_presentation();
    let mut docks = presentation
        .host_scene_data
        .document_surfaces()
        .iter()
        .filter(|dock| dock.pane.kind.as_str() == "Game");
    let dock = docks.next().expect("fixture: no published Game pane");
    assert!(
        docks.next().is_none(),
        "fixture: expected one Game document"
    );
    let viewport =
        authored_level_viewport(&presentation, dock).expect("fixture: no authored Game clip");
    let clip = viewport.clip;
    let status = presentation.host_layout.status_bar_frame;
    assert!(clip.width > 160.0 && clip.height > 100.0 && status.height > 0.0);
    let inside = |u: f32| UiPoint::new(clip.x + clip.width * u, clip.y + clip.height * 0.5);
    let down = inside(0.32);
    let drag = inside(0.44);
    let probe = inside(0.61);
    let outside = UiPoint::new(drag.x, status.y + status.height * 0.5);
    let window = ui.window().size();
    assert!(outside.y > clip.y + clip.height && outside.y < window.height as f32);
    assert!(
        outside.x >= status.x
            && outside.x < status.x + status.width
            && outside.x < window.width as f32
    );
    assert!(runtime.play_preview_view_focused().unwrap());
    // Use one loop and let the real Winit adapter assign mouse metadata and local coordinates.
    let dispatch = RefCell::new(ui.winit_pointer_driver_for_test());
    let mut pointer = |kind, point: UiPoint| {
        let position = PhysicalPosition::new(f64::from(point.x), f64::from(point.y));
        let event = match kind {
            Move => WindowEvent::PointerMoved {
                device_id: None,
                position,
                primary: true,
                source: PointerSource::Mouse,
            },
            Down | Up => WindowEvent::PointerButton {
                device_id: None,
                position,
                primary: true,
                button: MouseButton::Right.into(),
                state: if kind == Down {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                },
            },
            _ => unreachable!("ordinary mouse drag edges only"),
        };
        dispatch.borrow_mut()(event);
    };

    let start_drag = |pointer: &mut dyn FnMut(UiPointerEventKind, UiPoint)| {
        let pre = read_input();
        let (epoch, pre_transform) = read_camera();
        pointer(Down, down);
        pointer(Move, drag);
        let (drag_epoch, drag_transform) = read_camera();
        assert_eq!(drag_epoch, epoch);
        assert_ne!(
            drag_transform, pre_transform,
            "fixture: RMB Move with Down must enter the real Play orbit"
        );
        let armed = read_input();
        assert_eq!(
            armed.mouse_button_press_count,
            pre.mouse_button_press_count + 1
        );
        assert_eq!(
            armed.mouse_button_release_count,
            pre.mouse_button_release_count
        );
        assert_eq!(armed.pointer_move_count, pre.pointer_move_count + 1);
    };
    let initial = read_input();
    let initial_camera = read_camera();
    pointer(Up, outside); // no preceding Down
    pointer(Down, outside);
    pointer(Up, outside); // regular status-band click, still not a Game gesture
    let unowned = read_input();
    assert_eq!(
        unowned.mouse_button_press_count,
        initial.mouse_button_press_count
    );
    assert_eq!(
        unowned.mouse_button_release_count, initial.mouse_button_release_count,
        "an unowned outside Up must not be broadcast to Play"
    );
    pointer(Move, probe);
    assert_eq!(
        read_input().pointer_move_count,
        initial.pointer_move_count + 1
    );
    assert_eq!(
        read_camera(),
        initial_camera,
        "plain Move must not start a drag"
    );

    start_drag(&mut pointer);
    let inside_before = read_input();
    pointer(Up, drag);
    let inside_released = read_input();
    assert_eq!(
        inside_released.mouse_button_release_count,
        inside_before.mouse_button_release_count + 1
    );
    let inside_pose = read_camera();
    pointer(Move, probe);
    assert_eq!(
        read_input().pointer_move_count,
        inside_released.pointer_move_count + 1
    );
    assert_eq!(read_camera(), inside_pose, "inside Up must disarm orbit");

    start_drag(&mut pointer);
    let blur_pose = read_camera();
    let blur_input = read_input();
    dispatch.borrow_mut()(WindowEvent::Focused(false));
    pointer(Move, probe);
    let after_blur = read_input();
    assert_eq!(
        after_blur.pointer_move_count,
        blur_input.pointer_move_count + 1
    );
    assert_eq!(
        after_blur.mouse_button_release_count,
        blur_input.mouse_button_release_count
    );
    assert_eq!(
        read_camera(),
        blur_pose,
        "native blur is a separate reset, not a paired Up"
    );

    start_drag(&mut pointer);
    pointer(Move, outside);
    assert!(
        runtime.play_preview_view_focused().unwrap(),
        "hover exit must not change Game focus"
    );
    let before = read_input();
    pointer(Up, outside);
    let after = read_input();
    let after_up_pose = read_camera();
    assert!(
        runtime.play_preview_view_focused().unwrap(),
        "outside Up must not be a focus-leave test"
    );
    pointer(Move, probe);
    let after_probe = read_input();
    assert_eq!(
        after_probe.pointer_move_count,
        after.pointer_move_count + 1,
        "probe Move must actually reach this Play"
    );
    let after_probe_pose = read_camera();
    assert_eq!(
        runtime.play_sessions().attached_world_domain(),
        Some(domain)
    );
    assert_eq!(camera(&edit, &edit_identity, entity), authoring_before);
    let released_once = after.mouse_button_release_count == before.mouse_button_release_count + 1;
    let drag_ended = after_up_pose == after_probe_pose;
    eprintln!("PREVIEW168_GAME_CLIP_RELEASE owner={identity:?} releases={}→{} probe_moves={}→{} drag_ended={drag_ended}",
        before.mouse_button_release_count, after.mouse_button_release_count,
        after.pointer_move_count, after_probe.pointer_move_count);

    menu(&mut host.borrow_mut(), "workbench.play_mode.exit");
    assert_eq!(runtime.play_sessions().mode_snapshot(), PlayMode::Edit);
    assert!(runtime.play_sessions().attached_world_domain().is_none());
    assert!(released_once && drag_ended,
        "PREVIEW168_GAME_CLIP_RELEASE: outside RMB Up must reach and disarm the original Play orbit (release={released_once}, ended={drag_ended})");
}
