use std::any::Any;
use std::sync::Arc;

use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditCommandError, EditContext, HistoryContextId,
};
use crate::core::editing::intent::EditorIntent;
use crate::scene::modes::SceneModeActivation;
use crate::scene::selection::SelectionMutation;
use crate::scene::viewport::{
    DisplayMode, GridMode, HandleElementExtract, OverlayAxis, ProjectionMode, TransformHandleKind,
    ViewportCameraSnapshot,
};
use crate::ui::binding::ViewportCommand;
use crate::ui::workbench::startup::WelcomePaneSnapshot;
use crate::ui::workbench::state::EditorState;
use zircon_runtime::asset::pipeline::manager::{ProjectAssetManager, ProjectAssetManagerAccess};
use zircon_runtime::core::framework::render::{
    CapturedFrame, RenderFramework, RenderQualityProfile, RenderStats, RenderViewportDescriptor,
};
use zircon_runtime::core::manager::{RegisteredManagerService, manager_service_handle};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::{Transform, UVec2, Vec2};
use zircon_runtime_interface::ui::event_ui::UiTreeId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommandKind, UiRenderExtract, UiTextRenderMode,
};

use super::super::support::{cube_and_camera, cube_id, test_state};

#[test]
fn select_mode_click_selects_renderable_without_handle_overlay() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();

    let _ = state.apply_viewport_command(&ViewportCommand::ActivateSceneMode(
        SceneModeActivation::Select,
    ));

    let cursor = project_entity_cursor(
        &state,
        cube,
        zircon_runtime_interface::math::Vec3::new(0.55, 0.0, 0.0),
    );
    let _ = state.apply_viewport_command(&ViewportCommand::LeftPressed {
        x: cursor.x,
        y: cursor.y,
        selection_mutation: SelectionMutation::Replace,
    });
    let _ = state.apply_viewport_command(&ViewportCommand::LeftReleased);

    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(cube)
    );
    assert!(state.render_snapshot().unwrap().overlays.handles.is_empty());
}

#[test]
fn box_selection_applies_extend_and_toggle_mutations_across_pointer_lifecycle() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    let _ = state.apply_viewport_command(&ViewportCommand::ActivateSceneMode(
        SceneModeActivation::Select,
    ));
    let cursor = project_entity_cursor(&state, cube, zircon_runtime_interface::math::Vec3::ZERO);
    let start = cursor - Vec2::splat(12.0);
    let end = cursor + Vec2::splat(12.0);

    let _ = state.apply_viewport_command(&ViewportCommand::LeftPressed {
        x: start.x,
        y: start.y,
        selection_mutation: SelectionMutation::Extend,
    });
    let _ = state.apply_viewport_command(&ViewportCommand::PointerMoved { x: end.x, y: end.y });
    let _ = state.apply_viewport_command(&ViewportCommand::LeftReleased);

    assert!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .contains(&camera)
    );
    assert!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .contains(&cube)
    );

    let _ = state.apply_viewport_command(&ViewportCommand::LeftPressed {
        x: start.x,
        y: start.y,
        selection_mutation: SelectionMutation::Toggle,
    });
    let _ = state.apply_viewport_command(&ViewportCommand::PointerMoved { x: end.x, y: end.y });
    let _ = state.apply_viewport_command(&ViewportCommand::LeftReleased);

    assert!(
        state
            .viewport_controller
            .selection()
            .active_items()
            .contains(&camera)
    );
    assert!(
        !state
            .viewport_controller
            .selection()
            .active_items()
            .contains(&cube)
    );
}

#[test]
fn viewport_clicking_light_gizmo_selects_light_node() {
    let mut state = test_state();
    let light = state.world.with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| {
                matches!(
                    node.kind,
                    zircon_runtime::scene::components::NodeKind::DirectionalLight
                )
            })
            .map(|node| node.id)
            .expect("directional light")
    });

    let cursor = {
        let packet = state.render_snapshot().expect("render packet");
        let icon = packet
            .overlays
            .scene_gizmos
            .iter()
            .find(|gizmo| gizmo.owner == light)
            .and_then(|gizmo| gizmo.icons.first())
            .expect("light gizmo icon");
        project_world_position(
            &packet.scene.camera,
            state.viewport_state().size,
            icon.position,
        )
        .expect("light gizmo cursor")
    };

    let _ = state.apply_viewport_command(&ViewportCommand::LeftPressed {
        x: cursor.x,
        y: cursor.y,
        selection_mutation: SelectionMutation::Replace,
    });
    let _ = state.apply_viewport_command(&ViewportCommand::LeftReleased);

    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(light)
    );
}

#[test]
fn render_frame_extract_matches_legacy_render_snapshot_projection() {
    let state = test_state();

    let snapshot = state.render_snapshot().expect("render snapshot");
    let extract = state.render_frame_extract().expect("render frame extract");

    assert_eq!(extract.to_scene_snapshot(), snapshot);
}

#[test]
fn render_frame_submission_carries_editor_owned_viewport_text_overlay() {
    let state = test_state();

    let submission = state
        .render_frame_submission()
        .expect("render frame submission");
    let ui = submission.ui.expect("viewport hud overlay");
    let command = ui.list.commands.first().expect("viewport hud command");

    assert_eq!(ui.tree_id.0, "zircon.editor.viewport.hud");
    assert_eq!(command.kind, UiRenderCommandKind::Quad);
    assert_eq!(
        command.style.font.as_deref(),
        Some("res://fonts/default.font.toml")
    );
    assert_eq!(command.style.text_render_mode, UiTextRenderMode::Auto);
    assert!(
        command
            .text
            .as_deref()
            .is_some_and(|text| text.contains("Move") && text.contains("Persp"))
    );
}

#[test]
fn viewport_authoring_commands_do_not_mutate_runtime_world_or_default_extract() {
    let mut state = test_state();
    let world_before = state.world.with_world(|scene| scene.clone());
    let runtime_before = state.world.with_world(|scene| scene.to_render_extract());

    let _ = state.apply_viewport_command(&ViewportCommand::SetProjectionMode(
        ProjectionMode::Orthographic,
    ));
    let _ =
        state.apply_viewport_command(&ViewportCommand::SetDisplayMode(DisplayMode::WireOverlay));
    let _ = state.apply_viewport_command(&ViewportCommand::SetGridMode(GridMode::Hidden));
    let _ = state.apply_viewport_command(&ViewportCommand::SetPreviewLighting(false));
    let _ = state.apply_viewport_command(&ViewportCommand::SetPreviewSkybox(false));
    let _ = state.apply_viewport_command(&ViewportCommand::SetGizmosEnabled(false));

    let authored = state.render_snapshot().expect("editor render snapshot");
    assert_eq!(
        authored.scene.camera.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(authored.overlays.display_mode, DisplayMode::WireOverlay);
    assert!(authored.overlays.grid.is_none());
    assert!(authored.overlays.scene_gizmos.is_empty());
    assert!(!authored.preview.lighting_enabled);
    assert!(!authored.preview.skybox_enabled);

    let world_after = state.world.with_world(|scene| scene.clone());
    let runtime_after = state.world.with_world(|scene| scene.to_render_extract());
    assert_eq!(
        world_after, world_before,
        "viewport authoring commands should stay editor-owned instead of mutating the runtime world"
    );
    assert_eq!(
        runtime_after, runtime_before,
        "runtime default render extract should stay stable when only editor viewport state changes"
    );
}

#[test]
fn render_frame_submission_hud_text_renders_through_runtime_glyph_capture() {
    let state = test_state();
    let submission = state
        .render_frame_submission()
        .expect("render frame submission");
    let ui = submission.ui.clone().expect("viewport hud overlay");
    let hud_frame = ui
        .list
        .commands
        .first()
        .expect("viewport hud command")
        .frame;

    let (with_text, with_text_stats) = capture_editor_submission(
        submission.extract.clone(),
        Some(ui.clone()),
        state.viewport_state().size,
    );
    assert_eq!(with_text_stats.last_ui_command_count, 1);
    assert_eq!(with_text_stats.last_ui_quad_count, 1);
    assert_eq!(with_text_stats.last_ui_text_payload_count, 1);

    let mut background_only = ui;
    background_only.tree_id = UiTreeId::new("zircon.editor.viewport.hud.background-only");
    background_only
        .list
        .commands
        .first_mut()
        .expect("viewport hud command")
        .text = None;

    let (without_text, without_text_stats) = capture_editor_submission(
        submission.extract,
        Some(background_only),
        state.viewport_state().size,
    );
    assert_eq!(without_text_stats.last_ui_command_count, 1);
    assert_eq!(without_text_stats.last_ui_quad_count, 1);
    assert_eq!(without_text_stats.last_ui_text_payload_count, 0);

    let changed_pixels = count_changed_pixels_in_frame(
        &with_text.rgba,
        &without_text.rgba,
        with_text.width,
        with_text.height,
        hud_frame,
        12,
    );
    assert!(
        changed_pixels > 48,
        "expected editor-owned viewport HUD text to add visible glyph pixels through the shared runtime text backend; changed_pixels={changed_pixels}; glyphs={}; unmapped_glyphs={}; visible_raster_glyphs={}; raster_sources={}; worker_pending={}; worker_failed={}",
        with_text_stats.last_ui_text_glyph_count,
        with_text_stats.last_ui_text_unmapped_glyph_count,
        with_text_stats.last_ui_text_visible_raster_glyph_count,
        with_text_stats.last_ui_text_raster_source_image_count,
        with_text_stats.last_ui_text_raster_worker_pending_count,
        with_text_stats.last_ui_text_raster_worker_failed_count,
    );
}

#[test]
fn viewport_handle_drag_collapses_into_single_undoable_command() {
    let mut state = test_state();
    let (cube, start) = begin_moved_gizmo_drag(&mut state);
    state
        .apply_viewport_command(&ViewportCommand::LeftReleased)
        .unwrap();

    let after_drag = state.snapshot();
    assert!(after_drag.can_undo);
    assert!(!after_drag.can_redo);
    assert_ne!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );
    let after_undo = state.snapshot();
    assert!(!after_undo.can_undo);
    assert!(after_undo.can_redo);

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_ne!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );
}

#[test]
fn ordinary_scene_edit_cancels_gizmo_before_command_capture() {
    let mut state = test_state();
    let (cube, start) = begin_moved_gizmo_drag(&mut state);

    state
        .apply_intent(EditorIntent::RenameNode(cube, "Renamed cube".to_string()))
        .unwrap();

    let (name, transform) = state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        (node.name.clone(), node.transform)
    });
    assert_eq!(name, "Renamed cube");
    assert_eq!(transform, start);

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    let (name, transform) = state.world.with_world(|scene| {
        let node = scene.find_node(cube).unwrap();
        (node.name.clone(), node.transform)
    });
    assert_eq!(name, "Cube");
    assert_eq!(transform, start);
}

#[test]
fn deleting_during_drag_cancels_preview_before_command_capture() {
    let mut state = test_state();
    let (cube, _) = begin_moved_gizmo_drag(&mut state);

    assert!(state.apply_intent(EditorIntent::DeleteNode(cube)).unwrap());
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert!(state.begin_gizmo_transaction().unwrap());
    assert!(state.cancel_gizmo_transaction().unwrap());
}

#[test]
fn missing_drag_target_release_cleans_gizmo_lifecycle() {
    let mut state = test_state();
    let (cube, _) = begin_moved_gizmo_drag(&mut state);
    state.world.with_world_mut(|scene| {
        assert!(scene.remove_entity(cube));
    });

    state
        .apply_viewport_command(&ViewportCommand::LeftReleased)
        .expect_err("a vanished drag target must report release failure");
    assert!(!state.viewport_controller.is_handle_drag_active());

    let manager = DefaultLevelManager::default();
    state
        .replace_world(manager.create_default_level(), "replacement-project")
        .expect("release failure must not leave a latched gizmo capture");
}

#[test]
fn gizmo_transaction_failure_restores_transform() {
    let mut state = test_state();
    let (cube, start) = begin_moved_gizmo_drag(&mut state);

    fault_transaction_engine(&state);
    let (_, moved) = move_handle_drag_cursor_pair(&state, cube);
    let error = state
        .apply_viewport_command(&ViewportCommand::PointerMoved {
            x: moved.x + 20.0,
            y: moved.y,
        })
        .expect_err("a faulted transaction engine must reject the next gizmo mutation");

    assert!(error.contains("faulted"));
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );
    assert!(!state.viewport_controller.is_handle_drag_active());
}

#[test]
fn replacing_world_cancels_the_old_gizmo_capture() {
    let mut state = test_state();
    let _ = begin_moved_gizmo_drag(&mut state);
    let manager = DefaultLevelManager::default();

    state
        .replace_world(manager.create_default_level(), "replacement-project")
        .unwrap();

    let replacement_cube = cube_id(&state);
    state
        .apply_intent(EditorIntent::SelectNode(replacement_cube))
        .unwrap();
    assert!(state.begin_gizmo_transaction().unwrap());
    assert!(state.cancel_gizmo_transaction().unwrap());
}

#[test]
fn scene_mode_activation_cancels_an_active_gizmo_transaction() {
    let mut state = test_state();
    let (cube, initial) = begin_moved_gizmo_drag(&mut state);
    assert!(state.viewport_controller.is_handle_drag_active());

    state
        .apply_viewport_command(&ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Select,
        ))
        .unwrap();

    assert!(!state.viewport_controller.is_handle_drag_active());
    assert_eq!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        initial
    );
    assert_eq!(
        state.viewport_controller.active_scene_mode(),
        SceneModeActivation::Select
    );
    assert!(state.begin_gizmo_transaction().unwrap());
    assert!(state.cancel_gizmo_transaction().unwrap());
}

#[test]
fn faulted_transaction_engine_blocks_world_replacement() {
    let mut state = test_state();
    let original = state.world.snapshot();
    let manager = DefaultLevelManager::default();
    fault_transaction_engine(&state);

    let error = state
        .replace_world(manager.create_default_level(), "replacement-project")
        .expect_err("faulted transaction state must not cross into a new world");

    assert!(error.contains("faulted"));
    assert_eq!(state.world.snapshot(), original);
}

#[test]
fn faulted_transaction_engine_blocks_project_clear() {
    let mut state = test_state();
    let original = state.world.snapshot();
    fault_transaction_engine(&state);

    let error = state
        .clear_project(WelcomePaneSnapshot::default())
        .expect_err("faulted transaction state must not be hidden by project clear");

    assert!(error.contains("faulted"));
    assert_eq!(state.world.snapshot(), original);
    assert!(state.has_project_world());
}

pub(super) fn begin_moved_gizmo_drag(state: &mut EditorState) -> (u64, Transform) {
    let cube = cube_id(state);
    let start = state
        .world
        .with_world(|scene| scene.find_node(cube).unwrap().transform);
    state
        .apply_viewport_command(&ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Move),
        ))
        .unwrap();
    let (press, moved) = move_handle_drag_cursor_pair(state, cube);
    state
        .apply_viewport_command(&ViewportCommand::LeftPressed {
            x: press.x,
            y: press.y,
            selection_mutation: SelectionMutation::Replace,
        })
        .unwrap();
    state
        .apply_viewport_command(&ViewportCommand::PointerMoved {
            x: moved.x,
            y: moved.y,
        })
        .unwrap();
    assert_ne!(
        state
            .world
            .with_world(|scene| scene.find_node(cube).unwrap().transform),
        start
    );
    (cube, start)
}

#[derive(Debug)]
struct FaultTransactionEngineCommand;

impl EditCommand for FaultTransactionEngineCommand {
    fn label(&self) -> &str {
        "Fault transaction engine"
    }

    fn apply(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Err(CommandExecutionError::applied(
            EditCommandError::TargetMissing {
                target: "forced gizmo transaction failure".to_string(),
            },
        ))
    }

    fn revert(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Err(CommandExecutionError::unchanged(
            EditCommandError::TargetMissing {
                target: "forced gizmo transaction rollback failure".to_string(),
            },
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn fault_transaction_engine(state: &EditorState) {
    let mut scope = state
        .transactions()
        .begin("Force gizmo failure", HistoryContextId::Global)
        .unwrap();
    let error = scope
        .push(FaultTransactionEngineCommand)
        .expect_err("fixture command must fault the transaction engine");
    assert!(matches!(error, EditCommandError::RollbackFailed { .. }));
    drop(scope);
}

fn project_entity_cursor(
    state: &EditorState,
    entity: u64,
    offset: zircon_runtime_interface::math::Vec3,
) -> Vec2 {
    let packet = state.render_snapshot().expect("render packet");
    let transform = state
        .world
        .with_world(|scene| scene.world_transform(entity).unwrap());
    project_world_position(
        &packet.scene.camera,
        state.viewport_state().size,
        transform.matrix().transform_point3(offset),
    )
    .expect("entity cursor")
}

fn move_handle_drag_cursor_pair(state: &EditorState, cube: u64) -> (Vec2, Vec2) {
    let packet = state.render_snapshot().expect("render packet");
    let handle = packet
        .overlays
        .handles
        .iter()
        .find(|handle| handle.owner == cube)
        .expect("move handle");
    let (start, end) = handle
        .elements
        .iter()
        .find_map(|element| match element {
            HandleElementExtract::AxisLine {
                axis, start, end, ..
            } if *axis == OverlayAxis::X => Some((*start, *end)),
            _ => None,
        })
        .expect("x axis handle");
    let start_cursor =
        project_world_position(&packet.scene.camera, state.viewport_state().size, start)
            .expect("axis start");
    let end_cursor = project_world_position(&packet.scene.camera, state.viewport_state().size, end)
        .expect("axis end");
    let direction = (end_cursor - start_cursor).normalize_or_zero();
    let press = start_cursor + direction * 24.0;
    let release = press + direction * 96.0;
    (press, release)
}

fn project_world_position(
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
    world: zircon_runtime_interface::math::Vec3,
) -> Option<Vec2> {
    let aspect = viewport.x as f32 / viewport.y.max(1) as f32;
    let projection = match camera.projection_mode {
        ProjectionMode::Perspective => zircon_runtime_interface::math::perspective(
            camera.fov_y_radians,
            aspect,
            camera.z_near,
            camera.z_far,
        ),
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let half_width = half_height * aspect.max(0.001);
            zircon_runtime_interface::math::Mat4::orthographic_rh(
                -half_width,
                half_width,
                -half_height,
                half_height,
                camera.z_near.max(0.001),
                camera.z_far,
            )
        }
    };
    let clip = projection
        * zircon_runtime_interface::math::view_matrix(camera.transform)
        * world.extend(1.0);
    if clip.w <= f32::EPSILON {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    Some(Vec2::new(
        (ndc.x * 0.5 + 0.5) * viewport.x as f32,
        (-ndc.y * 0.5 + 0.5) * viewport.y as f32,
    ))
}

fn capture_editor_submission(
    extract: zircon_runtime::core::framework::render::RenderFrameExtract,
    ui: Option<UiRenderExtract>,
    viewport_size: UVec2,
) -> (CapturedFrame, RenderStats) {
    const ASYNC_TEXT_SETTLE_FRAME_COUNT: usize = 24;

    let asset_manager = Arc::new(ProjectAssetManager::default());
    let (_asset_runtime, asset_manager_access) = editor_render_asset_access(asset_manager);
    let framework = WgpuRenderFramework::new(asset_manager_access).expect("render framework");
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("viewport");
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("editor-viewport-hud")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false),
        )
        .expect("quality profile");
    for _ in 0..ASYNC_TEXT_SETTLE_FRAME_COUNT {
        framework
            .submit_frame_extract_with_ui(viewport, extract.clone(), ui.clone())
            .expect("frame submission");
    }
    let stats = framework.query_stats().expect("render stats");
    let capture = framework
        .capture_frame(viewport)
        .expect("capture frame query")
        .expect("captured frame");
    (capture, stats)
}

fn editor_render_asset_access(
    manager: Arc<ProjectAssetManager>,
) -> (CoreRuntime, ProjectAssetManagerAccess) {
    const MODULE_NAME: &str = "EditorRenderCaptureAssetRuntime";
    const SERVICE_NAME: &str = "EditorRenderCaptureAssetRuntime.Manager.ProjectAssetManager";

    let runtime = CoreRuntime::new();
    runtime
        .register_module(
            ModuleDescriptor::new(MODULE_NAME, "editor render capture asset runtime").with_manager(
                ManagerDescriptor::new(
                    RegistryName::from_parts(
                        MODULE_NAME,
                        ServiceKind::Manager,
                        "ProjectAssetManager",
                    ),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(
                            Arc::new(RegisteredManagerService::new(Arc::clone(&manager)))
                                as ServiceObject,
                        )
                    }),
                ),
            ),
        )
        .expect("editor render capture ProjectAssetManager service should register");
    runtime
        .activate_module(MODULE_NAME)
        .expect("editor render capture ProjectAssetManager module should activate");
    let core = runtime.handle();
    let handle = manager_service_handle(&core, SERVICE_NAME)
        .expect("editor render capture ProjectAssetManager handle should resolve");
    let access = ProjectAssetManagerAccess::new(core, handle);
    (runtime, access)
}

fn count_changed_pixels_in_frame(
    lhs: &[u8],
    rhs: &[u8],
    width: u32,
    height: u32,
    frame: UiFrame,
    threshold: u8,
) -> usize {
    let left = frame.x.max(0.0).floor() as usize;
    let top = frame.y.max(0.0).floor() as usize;
    let right = frame.right().max(0.0).ceil() as usize;
    let bottom = frame.bottom().max(0.0).ceil() as usize;
    let width = width as usize;
    let height = height as usize;

    let mut changed = 0usize;
    for y in top.min(height)..bottom.min(height) {
        for x in left.min(width)..right.min(width) {
            let index = (y * width + x) * 4;
            let delta = lhs[index..index + 4]
                .iter()
                .zip(&rhs[index..index + 4])
                .map(|(lhs, rhs)| lhs.abs_diff(*rhs))
                .max()
                .unwrap_or(0);
            if delta >= threshold {
                changed += 1;
            }
        }
    }

    changed
}
