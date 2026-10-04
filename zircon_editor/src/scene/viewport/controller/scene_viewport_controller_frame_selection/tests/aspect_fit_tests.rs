use super::SceneViewportController;
use crate::scene::viewport::projection::project_point;
use crate::scene::viewport::{ProjectionMode, ViewOrientation};
use crate::ui::binding::ViewportCommand;
use zircon_runtime::scene::components::{CameraComponent, NodeKind};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::{Transform, UVec2, Vec2, Vec3};

const SELECTION_CORNERS: [Vec3; 8] = [
    Vec3::new(-8.0, -8.0, -4.0),
    Vec3::new(-8.0, -8.0, 4.0),
    Vec3::new(-8.0, 8.0, -4.0),
    Vec3::new(-8.0, 8.0, 4.0),
    Vec3::new(8.0, -8.0, -4.0),
    Vec3::new(8.0, -8.0, 4.0),
    Vec3::new(8.0, 8.0, -4.0),
    Vec3::new(8.0, 8.0, 4.0),
];

fn selected_scene(
    viewport: UVec2,
    projection_mode: ProjectionMode,
    positions: &[Vec3],
    eye: Vec3,
    fov_y_degrees: f32,
) -> (Scene, SceneViewportController) {
    let mut scene = Scene::new();
    let camera = scene.active_camera();
    scene
        .update_transform(camera, Transform::from_translation(eye))
        .unwrap();
    scene
        .insert(
            camera,
            CameraComponent {
                fov_y_radians: fov_y_degrees.to_radians(),
                ..CameraComponent::default()
            },
        )
        .unwrap();
    let entities: Vec<_> = positions
        .iter()
        .map(|position| {
            let entity = scene.spawn_node(NodeKind::Empty).unwrap();
            scene
                .update_transform(entity, Transform::from_translation(*position))
                .unwrap();
            entity
        })
        .collect();
    let mut controller = SceneViewportController::new(viewport);
    controller
        .apply_command(
            Some(&scene),
            &ViewportCommand::SetProjectionMode(projection_mode),
        )
        .unwrap();
    controller
        .selection_mut()
        .replace_active(entities.iter().copied(), entities.first().copied());
    (scene, controller)
}

fn frame_selection(scene: &Scene, controller: &mut SceneViewportController) {
    let feedback = controller
        .apply_command(Some(scene), &ViewportCommand::FrameSelection)
        .unwrap();
    assert!(feedback.camera_updated);
}

fn assert_selection_is_visible(scene: &Scene, controller: &SceneViewportController) -> Vec<Vec2> {
    let camera = controller.current_camera(scene);
    let viewport = controller.viewport().size;
    controller
        .selection()
        .active_items()
        .iter()
        .map(|entity| {
            let position = scene.world_transform(*entity).unwrap().translation;
            let pixel = project_point(position, &camera, viewport)
                .expect("the framed selection must remain inside the test camera's clip range");
            assert!(
                pixel.x >= 0.0
                    && pixel.x <= viewport.x as f32
                    && pixel.y >= 0.0
                    && pixel.y <= viewport.y as f32,
                "{position:?} projected outside {viewport:?} in {:?}: {pixel:?}",
                camera.projection_mode,
            );
            pixel
        })
        .collect()
}

#[test]
fn frame_selection_command_fits_perspective_selection_in_portrait_and_landscape() {
    for viewport in [UVec2::new(320, 1280), UVec2::new(1280, 320)] {
        let (scene, mut controller) = selected_scene(
            viewport,
            ProjectionMode::Perspective,
            &SELECTION_CORNERS,
            Vec3::Z * 8.0,
            60.0,
        );
        frame_selection(&scene, &mut controller);
        assert_selection_is_visible(&scene, &controller);
        assert_eq!(controller.orbit_target(), Vec3::ZERO);
    }
}

#[test]
fn frame_selection_command_fits_orthographic_selection_in_portrait_and_landscape() {
    let mut distances = Vec::new();
    for viewport in [UVec2::new(320, 1280), UVec2::new(1280, 320)] {
        let (scene, mut controller) = selected_scene(
            viewport,
            ProjectionMode::Orthographic,
            &SELECTION_CORNERS,
            Vec3::Z * 8.0,
            60.0,
        );
        frame_selection(&scene, &mut controller);
        assert_selection_is_visible(&scene, &controller);
        assert_eq!(controller.orbit_target(), Vec3::ZERO);
        distances.push(
            controller
                .current_camera(&scene)
                .transform
                .translation
                .length(),
        );
    }
    assert!((distances[0] - distances[1]).abs() < 0.0001);
}

#[test]
fn frame_selection_command_uses_horizontal_fov_for_a_wide_vertical_lens() {
    let (scene, mut controller) = selected_scene(
        UVec2::new(320, 1280),
        ProjectionMode::Perspective,
        &[-Vec3::X * 8.0, Vec3::X * 8.0],
        Vec3::Z * 8.0,
        120.0,
    );
    frame_selection(&scene, &mut controller);
    let pixels = assert_selection_is_visible(&scene, &controller);

    // Screen-space golden for the existing padded sphere fit. Scaling distance by
    // inverse aspect instead of converting the horizontal angle zooms out too far.
    assert!((pixels[0].x - 32.325_157).abs() < 0.01);
    assert!((pixels[1].x - 287.674_84).abs() < 0.01);
}

#[test]
fn frame_selection_command_preserves_larger_existing_distance_and_lens() {
    for projection in [ProjectionMode::Perspective, ProjectionMode::Orthographic] {
        let (scene, mut controller) = selected_scene(
            UVec2::new(320, 1280),
            projection,
            &SELECTION_CORNERS,
            Vec3::Z * 150.0,
            60.0,
        );
        let before = controller.current_camera(&scene);
        frame_selection(&scene, &mut controller);
        let after = controller.current_camera(&scene);

        assert!((after.transform.translation.length() - 150.0).abs() < 0.0001);
        assert_eq!(after.projection_mode, before.projection_mode);
        assert_eq!(after.fov_y_radians, before.fov_y_radians);
        assert_eq!(after.z_near, before.z_near);
        assert_eq!(after.z_far, before.z_far);
        assert_selection_is_visible(&scene, &controller);
    }
}

#[test]
fn frame_selection_command_preserves_single_point_floors_and_zero_offset_fallback() {
    for projection in [ProjectionMode::Perspective, ProjectionMode::Orthographic] {
        for eye in [Vec3::Z, Vec3::ZERO] {
            let (scene, mut controller) =
                selected_scene(UVec2::new(320, 1280), projection, &[Vec3::ZERO], eye, 60.0);
            frame_selection(&scene, &mut controller);
            let camera = controller.current_camera(&scene);

            assert!((camera.transform.translation.length() - 6.0).abs() < 0.0001);
            if projection == ProjectionMode::Orthographic {
                assert_eq!(camera.ortho_size, 2.5);
            }
            assert_selection_is_visible(&scene, &controller);
        }
    }
}

#[test]
fn frame_selection_command_without_selection_or_scene_preserves_camera_and_orientation() {
    let (scene, mut controller) = selected_scene(
        UVec2::new(320, 1280),
        ProjectionMode::Perspective,
        &[],
        Vec3::Z * 8.0,
        60.0,
    );
    controller
        .apply_command(
            Some(&scene),
            &ViewportCommand::AlignView(ViewOrientation::PosX),
        )
        .unwrap();
    let before = controller.current_camera(&scene);
    let target = controller.orbit_target();

    for scene_input in [Some(&scene), None] {
        let feedback = controller
            .apply_command(scene_input, &ViewportCommand::FrameSelection)
            .unwrap();
        assert!(!feedback.camera_updated);
        assert_eq!(controller.current_camera(&scene), before);
        assert_eq!(controller.orbit_target(), target);
        assert_eq!(
            controller.settings().view_orientation,
            ViewOrientation::PosX
        );
    }
}

#[test]
fn frame_selection_command_uses_resized_and_normalized_viewport_dimensions() {
    for projection in [ProjectionMode::Perspective, ProjectionMode::Orthographic] {
        for requested in [
            UVec2::new(320, 1280),
            UVec2::ZERO,
            UVec2::new(0, 32),
            UVec2::new(32, 0),
        ] {
            let (scene, mut controller) = selected_scene(
                UVec2::new(1280, 320),
                projection,
                &[-Vec3::ONE * 0.5, Vec3::ONE * 0.5],
                Vec3::Z * 8.0,
                60.0,
            );
            frame_selection(&scene, &mut controller);
            controller.resize(requested);
            frame_selection(&scene, &mut controller);

            let normalized = UVec2::new(requested.x.max(1), requested.y.max(1));
            assert_eq!(controller.viewport().size, normalized);
            assert_eq!(
                controller.current_camera(&scene).aspect_ratio,
                normalized.x as f32 / normalized.y as f32,
            );
            assert_selection_is_visible(&scene, &controller);
        }
    }
}
