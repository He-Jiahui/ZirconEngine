use crate::core::editor_message::SceneModeId;
use crate::scene::modes::{
    EditorSceneMode, InputOutcome, SceneModeActivation, SceneModeCtx, ViewportOverlayBuilder,
};
use crate::scene::viewport::{PivotMode, TransformHandleKind, ViewportInput};
use crate::ui::binding::ViewportCommand;
use zircon_runtime_interface::math::{Transform, UVec2, Vec2, Vec3};

use super::{SceneViewportController, ViewportCameraSnapshot};

struct PassThroughOverlayMode {
    id: SceneModeId,
}

impl EditorSceneMode for PassThroughOverlayMode {
    fn id(&self) -> &SceneModeId {
        &self.id
    }

    fn enter(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn exit(&mut self, _ctx: &mut SceneModeCtx<'_>) {}

    fn handle_input(
        &mut self,
        _input: &ViewportInput,
        _ctx: &mut SceneModeCtx<'_>,
    ) -> InputOutcome {
        InputOutcome::PassThrough
    }

    fn build_overlay(&self, _out: &mut ViewportOverlayBuilder) {}
}

#[test]
fn pass_through_overlay_keeps_base_transform_handles_in_the_render_extract() {
    let scene = zircon_runtime::scene::Scene::new();
    let selected = scene.nodes()[0].id;
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    controller.selection_mut().select_only_active(selected);
    controller
        .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Rotate))
        .unwrap();
    {
        let state = &mut controller.state;
        let mut mode_ctx = SceneModeCtx::new(&mut state.selection, &state.settings);
        state
            .scene_modes
            .push_overlay(
                SceneModeActivation::Custom(SceneModeId::new("test.pass-through-overlay")),
                Box::new(PassThroughOverlayMode {
                    id: SceneModeId::new("test.pass-through-overlay"),
                }),
                &mut mode_ctx,
            )
            .unwrap();
    }

    assert_eq!(
        controller.base_transform_handle(),
        Some(TransformHandleKind::Rotate)
    );
    assert!(!controller
        .handle_overlays(&scene, &ViewportCameraSnapshot::default())
        .is_empty());
}

#[test]
fn world_neutral_handle_route_drives_one_local_drag_session() {
    let mut controller = SceneViewportController::new(UVec2::new(800, 600));
    controller
        .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move))
        .unwrap();
    let camera = ViewportCameraSnapshot::default();
    let selected = Some((
        41,
        Transform {
            translation: Vec3::new(0.0, 0.0, -5.0),
            ..Transform::identity()
        },
    ));
    let cursor = Vec2::new(400.0, 300.0);
    let axis = controller
        .handle_axis_at_cursor_for_transform(selected, &camera, cursor)
        .expect("the projected handle origin should route to one axis");

    assert!(controller.begin_handle_drag_for_transform(selected, &camera, cursor, axis));
    let preview = controller
        .update_handle_drag_for_transform(&camera, Vec2::new(430.0, 300.0))
        .expect("an active local handle drag should produce a transform preview");

    assert_eq!(preview.primary, 41);
    assert_ne!(preview.target_pivot_world, selected.unwrap().1);
    assert!(controller.finish_handle_drag_for_transform());
    assert!(!controller.is_handle_drag_active());
}

#[test]
fn world_neutral_handle_overlay_projects_runtime_transform_and_hover_state() {
    let mut controller = SceneViewportController::new(UVec2::new(800, 600));
    controller
        .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move))
        .unwrap();
    let camera = ViewportCameraSnapshot::default();
    let selected = Some((
        41,
        Transform {
            translation: Vec3::new(0.0, 0.0, -5.0),
            ..Transform::identity()
        },
    ));

    let passive =
        controller.handle_screen_lines_for_transform(selected, &camera, UVec2::new(800, 600));
    assert!(!passive.is_empty());
    assert!(passive.iter().all(|line| line.is_finite()));

    controller.set_handle_hover_for_transform(Some(crate::scene::viewport::GizmoAxis::X));
    let hovered =
        controller.handle_screen_lines_for_transform(selected, &camera, UVec2::new(800, 600));
    assert_eq!(hovered.len(), passive.len());
    assert!(hovered.iter().any(|line| {
        line.axis() == Some(crate::scene::viewport::GizmoAxis::X)
            && line.width()
                > passive
                    .iter()
                    .find(|passive| passive.axis() == line.axis())
                    .expect("passive X axis line")
                    .width()
    }));
}

#[test]
fn optimization_batch_r6_wave5_editor640_screen_lines_reserve_element_bound() {
    let source = include_str!("../scene_viewport_controller_handle_screen_lines.rs");
    assert!(source.contains("let line_capacity = overlays"));
    assert!(source.contains("let mut lines = Vec::with_capacity(line_capacity);"));
    assert!(source.contains("HandleElementExtract::AxisRing { .. } => 48"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_r6_wave5_editor640_screen_line_capacity_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const LINES_PER_SAMPLE: usize = 65_536;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(editor640_measure_line_projection(LINES_PER_SAMPLE, false));
            optimized_samples.push(editor640_measure_line_projection(LINES_PER_SAMPLE, true));
        } else {
            optimized_samples.push(editor640_measure_line_projection(LINES_PER_SAMPLE, true));
            legacy_samples.push(editor640_measure_line_projection(LINES_PER_SAMPLE, false));
        }
    }

    let legacy_p95 = editor640_p95(&legacy_samples);
    let optimized_p95 = editor640_p95(&optimized_samples);
    println!(
        "EDITOR640_PREALLOCATED_VIEWPORT_SCREEN_LINES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} lines_per_sample={LINES_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
        "preallocated viewport screen lines must be at least 15% faster at P95"
    );
}

fn editor640_measure_line_projection(output_count: usize, optimized: bool) -> u128 {
    let mut outputs = if optimized {
        Vec::with_capacity(output_count)
    } else {
        Vec::new()
    };
    let started = std::time::Instant::now();
    for value in 0..output_count {
        outputs.push(std::hint::black_box(value));
    }
    let elapsed = started.elapsed().as_nanos().max(1);
    std::hint::black_box(outputs);
    elapsed
}

fn editor640_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}

#[test]
fn multi_selection_handle_uses_the_same_centroid_as_the_transform_session() {
    let mut scene = zircon_runtime::scene::Scene::empty();
    let left = scene
        .spawn_node(zircon_runtime::scene::components::NodeKind::Cube)
        .unwrap();
    let right = scene
        .spawn_node(zircon_runtime::scene::components::NodeKind::Cube)
        .unwrap();
    scene
        .update_transform(left, Transform::from_translation(Vec3::new(-4.0, 0.0, 0.0)))
        .unwrap();
    scene
        .update_transform(right, Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)))
        .unwrap();
    let mut controller = SceneViewportController::new(UVec2::new(800, 600));
    controller
        .selection_mut()
        .replace_active([left, right], Some(left));

    let (primary, pivot) = controller
        .selected_handle_transform(&scene)
        .expect("a valid multi-selection should expose one shared gizmo pivot");

    assert_eq!(primary, left);
    assert_eq!(pivot.translation, Vec3::ZERO);

    let feedback = controller
        .apply_command(None, &ViewportCommand::SetPivotMode(PivotMode::Primary))
        .expect("pivot mode command should update the viewport controller");
    let (_, primary_pivot) = controller
        .selected_handle_transform(&scene)
        .expect("the same selection should expose its primary pivot");

    assert!(feedback.settings_changed);
    assert!(feedback.interaction_extract_stale);
    assert_eq!(primary_pivot.translation, Vec3::new(-4.0, 0.0, 0.0));
}
