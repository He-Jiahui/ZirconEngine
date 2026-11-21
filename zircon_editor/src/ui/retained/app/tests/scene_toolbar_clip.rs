use super::support::*;
use crate::core::project::{NewProjectDraft, ProjectAuthority, ProjectTemplateId};
use crate::ui::retained_host::host_contract::{
    record_scene_toolbar_control_ink_for_test, scene_toolbar_route_for_test, FloatingWindowData,
    HostPresentationGeneration, TemplatePaneNodeData,
};
use crate::ui::workbench::view::ViewHost;
use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::{
    project::{ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource},
    ui::surface::{UiHitTestEntry, UiSurfaceFrame},
};

const FLOAT: &str = "window:scene-toolbar-clip";
const URI: &str = "res://scenes/main.scene.toml";
const REQUESTED: ShellFrame = ShellFrame::new(320.0, 215.0, 320.0, 256.0);
const WIDTH: u32 = 1672;
const HEIGHT: u32 = 941;

#[test]
fn reopened_scene_overlay_rejects_clipped_toolbar_commands() {
    let _guard = lock_env();
    let location = unique_temp_path("zircon_scene_toolbar_clip").with_extension("");
    fs::create_dir(&location).expect("fresh, private project parent");
    eprintln!("SCENE_TOOLBAR_FIXTURE={}", location.display());
    let (project, view) = write_project_with_scene_overlay(&location);
    let (_core, ui, host) = cold_open(&location, &project);
    let view_id = EventViewInstanceId::new(view.0.clone());
    {
        let host = host.borrow();
        let manager = host.editor_manager.enter().expect("project manager");
        let (root, _, _) = manager
            .active_project_session_focus_target()
            .expect("normal Open must reach Ready");
        let scene = manager
            .active_scene_identity_for_session()
            .expect("real Scene document");
        assert_eq!(
            root.canonicalize().unwrap(),
            project.canonicalize().unwrap()
        );
        assert_eq!(scene.scene_uri(), URI);
        assert!(scene.source_revision().is_some());
        assert!(matches!(manager.view_host_for_instance_key(&view.0),
            Some(ViewHost::FloatingWindow(id, _)) if id.0 == FLOAT));
        assert!(host.committed_shell_state.is_some());
    }
    let generation = ui.get_host_presentation_generation();
    let float = scene_float(&generation);
    assert_eq!(float.active_pane.kind.as_str(), "Scene");
    assert_eq!(float.active_pane.id.as_str(), view.0.as_str());
    let viewport = &float.active_pane.viewport;
    assert!(float.active_pane.show_toolbar);
    assert_eq!(viewport.toolbar_surface_key.as_str(), FLOAT);
    assert_eq!(ui.window().scale_factor(), 1.0);
    assert_eq!(float.frame.width, REQUESTED.width);
    let derived = viewport
        .toolbar_surface_frame
        .as_ref()
        .expect("published toolbar cache frame");
    let root = viewport
        .toolbar_template_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "SceneViewportToolbarRoot")
        .expect("native toolbar, not UnrealViewportToolbar");
    let group = viewport
        .toolbar_template_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "SceneViewportToolbarLeftGroup")
        .expect("real authored scrollport");
    assert_eq!(root.frame.width, float.frame.width - 2.0);
    assert_eq!(root.frame.height, 28.0);
    let controls = viewport
        .toolbar_template_nodes
        .iter()
        .filter(|node| {
            !node.disabled
                && node.parent_node_id == group.node_id
                && is_state_control(node.control_id.as_str())
        })
        .collect::<Vec<_>>();
    let origin = UiPoint::new(
        float.frame.x + 1.0,
        float.frame.y + float.header_frame.height + 1.0,
    );
    let source = controls
        .iter()
        .find_map(|node| node.source_surface_frame.as_ref())
        .expect("SCENE_TOOLBAR_ADMISSION: the live toolbar has no Runtime publication");
    eprintln!("SCENE_TOOLBAR_GEOMETRY key={FLOAT} view={} client={}x{} scale={} outer={:?} scrollport=({}, {}, {}, {}) source={}@{} cache={}@{} entries={:?}",
        view.0, ui.window().size().width, ui.window().size().height, ui.window().scale_factor(),
        float.frame, group.frame.x, group.frame.y, group.frame.width, group.frame.height, source.tree_id.0,
        source.generation, derived.tree_id.0, derived.generation,
        source.hit_grid.entries.iter().map(|e| (e.control_id.as_deref(), e.frame, e.clip_frame)).collect::<Vec<_>>());
    let point_candidates = |node: &TemplatePaneNodeData, visible: bool| {
        if visible
            && !node
                .source_surface_frame
                .as_ref()
                .is_some_and(|frame| Arc::ptr_eq(frame, source))
        {
            return None;
        }
        let source_entry = entry_for(source, node.control_id.as_str());
        assert_eq!(Some(source_entry.node_id), node.surface_node_id);
        assert_eq!(
            source_entry.frame,
            UiFrame::new(
                node.frame.x,
                node.frame.y,
                node.frame.width,
                node.frame.height
            )
        );
        let search = source_entry.frame.intersection(UiFrame::new(
            0.0,
            0.0,
            root.frame.width,
            root.frame.height,
        ))?;
        let y = search.center().y;
        ((search.x.ceil() as i32)..=(search.right().floor() as i32)).find_map(|x| {
            let local = UiPoint::new(x as f32, y);
            // Stay off the raw edge and the source scrollport edge. A real source hit stack
            // rejects right-side controls, ScrollableBox itself, and any other foreground owner.
            if local.x <= search.x + 0.25 || local.x >= search.right() - 0.25 {
                return None;
            }
            let source_hit = hit_test_surface_frame(source, local);
            if if visible {
                source_hit.top_hit != Some(source_entry.node_id)
            } else {
                !source_hit.stacked.is_empty()
                    || local.x <= group.frame.x + group.frame.width + 0.25
            } {
                return None;
            }
            let window = UiPoint::new(origin.x + local.x, origin.y + local.y);
            let (blocked, _) = scene_toolbar_route_for_test(&generation, window);
            (!blocked).then_some((local, window))
        })
    };
    let (hidden, hidden_local, hidden_window) = controls.iter().find_map(|node|
        point_candidates(node, false).map(|(local, window)| (*node, local, window))
    ).expect("SCENE_TOOLBAR_ADMISSION: no unoccluded raw-but-source-clipped point at the published extent");
    let (visible, visible_local, visible_window) = std::iter::once(hidden)
        .chain(controls.iter().copied())
        .find_map(|node| point_candidates(node, true).map(|(local, window)| (node, local, window)))
        .expect("SCENE_TOOLBAR_ADMISSION: no source-visible primary positive control");

    let log_point = |label: &str, node: &TemplatePaneNodeData, local, window| {
        let src = entry_for(source, node.control_id.as_str());
        let h = hit_test_surface_frame(source, local);
        let d = hit_test_surface_frame(derived, local);
        let mapped = derived
            .hit_grid
            .entries
            .iter()
            .find(|entry| entry.frame == src.frame)
            .expect("the cached hit input must retain the raw frame/identity");
        let (_, route) = scene_toolbar_route_for_test(&generation, window);
        let ink = record_scene_toolbar_control_ink_for_test(
            &generation,
            WIDTH,
            HEIGHT,
            source,
            src.node_id,
        );
        eprintln!("SCENE_TOOLBAR_POINT {label} key={FLOAT} view={} outer={:?} scrollport={:?} source={}@{}:{:?} control={} raw={:?} source_clip={:?} cached_clip={:?} local={local:?} window={window:?} source_stack={:?} pane_stack={:?} route={route:?} paint={ink:?}",
            view.0, float.frame, (group.frame.x, group.frame.y, group.frame.width, group.frame.height), source.tree_id.0, source.generation, src.node_id,
            node.control_id.as_str(), src.frame, src.clip_frame, mapped.clip_frame, h.stacked, d.stacked);
        (route, ink)
    };
    let (hidden_route, hidden_ink) = log_point("clipped", hidden, hidden_local, hidden_window);
    let (visible_route, visible_ink) = log_point("visible", visible, visible_local, visible_window);
    assert_eq!(
        visible_route,
        Some((
            FLOAT.into(),
            format!("{FLOAT}::{}", visible.control_id.as_str())
        ))
    );
    assert!(
        !visible_ink.is_empty(),
        "SCENE_TOOLBAR_ADMISSION: the positive must paint through the submitted source owner"
    );
    assert!(
        !hidden_ink
            .iter()
            .any(|ink| ink.contains_point(hidden_window)),
        "clipped point must not draw this source control"
    );
    // A valid painted positive and a real typed event must precede any negative verdict.
    let settings = || {
        host.borrow()
            .runtime
            .scene_viewport_settings_for_view(&view_id)
            .unwrap()
    };
    let click = |point: UiPoint| {
        let baseline = host.borrow().runtime.journal().records().len();
        ui.dispatch_native_primary_press_for_test(point.x, point.y);
        ui.dispatch_native_primary_release_for_test(point.x, point.y);
        host.borrow().runtime.journal().records()[baseline..]
            .iter()
            .map(|record| record.event.clone())
            .collect::<Vec<_>>()
    };
    let before_hidden = settings();
    let hidden_events = click(hidden_window);
    let after_hidden = settings();
    let visible_events = click(visible_window);
    let after_visible = settings();
    let visible_actions = state_actions(&visible_events, &view_id, visible.control_id.as_str());
    let wrong_actions = state_actions(&hidden_events, &view_id, hidden.control_id.as_str());
    let wrong_route = hidden_route.as_ref().is_some_and(|(key, control)| {
        key == FLOAT && control == &format!("{FLOAT}::{}", hidden.control_id.as_str())
    });
    let changed = before_hidden != after_hidden;
    eprintln!("SCENE_TOOLBAR_PRIMARY clipped={hidden_events:?} visible={visible_events:?} before={before_hidden:?} after_clipped={after_hidden:?} after_visible={after_visible:?}");
    assert_eq!(visible_actions, 1,
        "SCENE_TOOLBAR_ADMISSION: positive input must execute one typed command on the restored Scene");
    assert_ne!(
        after_hidden, after_visible,
        "SCENE_TOOLBAR_ADMISSION: positive input must change the Scene setting"
    );
    eprintln!("SCENE_TOOLBAR_ADMITTED visible_actions={visible_actions} wrong_route={wrong_route} wrong_actions={wrong_actions} changed={changed} any_toolbar_route={}", hidden_route.is_some());
    assert!(hidden_route.is_none() && wrong_actions == 0 && !changed,
        "SCENE_TOOLBAR_CLIPPED_COMMAND: no native toolbar target, typed command or chrome change at the clipped point");
    host.borrow_mut()
        .commit_project_close()
        .expect("close only the private project");
    drop(host);
    drop(ui);
    drop(_core);
    fs::remove_dir_all(location).expect("remove only the private successful test fixture");
}

fn scene_float(generation: &HostPresentationGeneration) -> &FloatingWindowData {
    let structure = generation.structure();
    assert!(
        !structure.host_shell.native_floating_window_mode,
        "observe the real root canvas, not a child mirror"
    );
    structure
        .host_scene_data
        .floating_layer
        .floating_windows
        .iter()
        .find(|window| window.window_id.as_str() == FLOAT)
        .expect("restored layout must publish a root floating Scene")
}

fn entry_for<'a>(frame: &'a UiSurfaceFrame, id: &str) -> &'a UiHitTestEntry {
    frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.control_id.as_deref() == Some(id))
        .expect(
            "real source grid must retain the control entry, even if its effective area is empty",
        )
}

fn is_state_control(id: &str) -> bool {
    matches!(
        id,
        "SetPivotMode"
            | "SetDisplayMode"
            | "SetGridMode"
            | "SetTranslateSnap"
            | "SetRotateSnapDegrees"
            | "SetScaleSnap"
            | "SetPreviewLighting"
            | "SetPreviewSkybox"
            | "SetGizmosEnabled"
    )
}

fn state_actions(events: &[EditorEvent], view: &EventViewInstanceId, id: &str) -> usize {
    events
        .iter()
        .filter(|event| {
            let EditorEvent::Viewport(EditorViewportEvent::ForView { view_id, event }) = event
            else {
                return false;
            };
            view_id == view
                && matches!(
                    (id, event.as_ref()),
                    ("SetPivotMode", EditorViewportEvent::SetPivotMode { .. })
                        | ("SetDisplayMode", EditorViewportEvent::SetDisplayMode { .. })
                        | ("SetGridMode", EditorViewportEvent::SetGridMode { .. })
                        | (
                            "SetTranslateSnap",
                            EditorViewportEvent::SetTranslateSnap { .. }
                        )
                        | (
                            "SetRotateSnapDegrees",
                            EditorViewportEvent::SetRotateSnapDegrees { .. }
                        )
                        | ("SetScaleSnap", EditorViewportEvent::SetScaleSnap { .. })
                        | (
                            "SetPreviewLighting",
                            EditorViewportEvent::SetPreviewLighting { .. }
                        )
                        | (
                            "SetPreviewSkybox",
                            EditorViewportEvent::SetPreviewSkybox { .. }
                        )
                        | (
                            "SetGizmosEnabled",
                            EditorViewportEvent::SetGizmosEnabled { .. }
                        )
                )
        })
        .count()
}

fn write_project_with_scene_overlay(location: &std::path::Path) -> (PathBuf, ViewInstanceId) {
    let seed = ChildWindowHostHarness::new_at_config_path(location.join("seed-config.json"));
    seed.activate_scene_for_editor09_close_fixture();
    let manager = seed
        ._core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let session = manager
        .create_project_and_open(NewProjectDraft {
            project_name: "SceneToolbarClip".into(),
            location: location.to_string_lossy().into_owned(),
            template: ProjectTemplateId::RenderableEmpty,
        })
        .expect("create the real RenderableEmpty project");
    let project = session.project.as_ref().expect("admitted seed project");
    // Author a persisted workspace input through the real owner. This is deliberately not
    // a claim that the ordinary componentized Scene exposes a desktop detach gesture.
    let scene = manager
        .open_view(ViewDescriptorId::new("editor.scene"), None)
        .unwrap();
    assert!(manager
        .apply_layout_command(LayoutCommand::DetachViewToWindow {
            instance_id: scene.clone(),
            new_window: MainPageId::new(FLOAT),
        })
        .unwrap());
    let mut workspace = manager.project_workspace();
    workspace
        .workbench
        .floating_windows
        .iter_mut()
        .find(|window| window.window_id.0 == FLOAT)
        .expect("persisted detached Scene row")
        .frame = REQUESTED;
    manager
        .save_active_scene_with_workspace(&project.root_path, &project.world, &workspace)
        .expect("persist the real Scene and versioned workspace together");
    let close = manager.begin_project_close().unwrap().unwrap();
    manager.commit_project_close(&close).unwrap();
    manager.finalize_project_close(&close).unwrap();
    (project.root_path.clone(), scene)
}

fn cold_open(
    location: &std::path::Path,
    project: &std::path::Path,
) -> (CoreRuntime, UiHostWindow, Rc<RefCell<RetainedEditorHost>>) {
    let config = crate::tests::editor_event::support::EnvironmentVariableRestoreGuard::capture(
        "ZIRCON_CONFIG_PATH",
    );
    std::env::set_var("ZIRCON_CONFIG_PATH", location.join("config.json"));
    let core = CoreRuntime::new();
    for engine_module in zircon_runtime::builtin::runtime_core_modules() {
        core.register_module(engine_module.descriptor()).unwrap();
    }
    core.register_module(zircon_runtime::ui::module_descriptor())
        .unwrap();
    core.register_module(module::module_descriptor()).unwrap();
    core.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    core.activate_module(zircon_runtime::core::framework::scene::SCENE_MODULE_NAME)
        .unwrap();
    core.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    let launch = crate::tests::support::test_project_creation_provenance();
    let intent = ProjectLaunchIntent::open_existing(
        launch.operation_id(),
        ProjectLaunchSource::Cli,
        ProjectLaunchProfile::Normal,
        project,
    )
    .unwrap();
    let startup = EditorGuiStartupRequest::project(intent)
        .preflight_project(&ProjectAuthority::default())
        .unwrap();
    let ui = UiHostWindow::new().expect("retained root host");
    ui.window().set_size(PhysicalSize::new(WIDTH, HEIGHT));
    let mut host = RetainedEditorHost::new(
        core.handle(),
        Arc::new(crate::core::gateway::DetachedEditorRuntimeGateway),
        ui.clone_strong(),
        Some(startup),
        Some(launch.build_set_id().clone()),
        None,
    )
    .expect("normal cold Open");
    drop(config);
    assert_eq!(host.startup_session.mode, EditorSessionMode::Project);
    let settings = host.runtime.context().settings().snapshot();
    apply_host_appearance_from_tokens(settings.design_tokens_handle(), Some(settings.generation()));
    apply_host_paint_scale_factor(
        ResolutionContext::from_physical_size_with_scale_mode(
            host.shell_size,
            host.shell_scale_factor,
            host.shell_scale_mode,
        )
        .effective_scale_factor(),
    );
    ui.sync_host_paint_theme();
    install_editor_v2_design_tokens(settings.as_ref());
    host.sync_plugin_template_documents_if_changed()
        .expect("real Editor templates");
    let host = Rc::new(RefCell::new(host));
    wire_callbacks(&ui, &host, None);
    host.borrow_mut().self_handle = Some(Rc::downgrade(&host));
    ui.show().unwrap();
    host.borrow_mut().sync_shell_size();
    host.borrow_mut().refresh_ui();
    host.borrow_mut().recompute_if_dirty();
    assert!(ui.take_fatal_failure().is_none());
    (core, ui, host)
}
