use crate::core::editing::intent::EditorIntent;
use crate::scene::viewport::ViewportInput;
use crate::ui::workbench::state::EditorState;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::math::{Transform, Vec2, Vec3};

use super::super::support::{cube_and_camera, test_state};

#[test]
fn active_camera_transform_command_refreshes_the_editor_camera_snapshot() {
    let mut state = test_state();
    let (_, active_camera) = cube_and_camera(&state);
    let before = viewport_camera_transform(&state);
    let mut authored = state
        .world
        .expect_with_world(|scene| scene.find_node(active_camera).unwrap().transform);
    authored.translation += Vec3::new(3.0, 1.0, -2.0);

    assert!(state
        .apply_intent(EditorIntent::SetTransform(active_camera, authored))
        .unwrap());

    let authoritative = state
        .world
        .expect_with_world(|scene| scene.world_transform(active_camera).unwrap());
    assert_ne!(authoritative, before);
    assert_eq!(viewport_camera_transform(&state), authoritative);
}

#[test]
fn active_camera_parent_transform_undo_redo_keeps_the_editor_camera_authoritative() {
    let mut state = test_state();
    let (_, active_camera) = cube_and_camera(&state);
    let parent = state.world.expect_with_world_mut(|scene| {
        let parent = scene.spawn_node(NodeKind::Empty).unwrap();
        scene
            .set_parent_checked(active_camera, Some(parent))
            .unwrap();
        parent
    });
    reset_viewport_from_scene(&mut state);
    let before = viewport_camera_transform(&state);
    let mut authored = state
        .world
        .expect_with_world(|scene| scene.find_node(parent).unwrap().transform);
    authored.translation += Vec3::new(-4.0, 2.0, 1.0);

    assert!(state
        .apply_intent(EditorIntent::SetTransform(parent, authored))
        .unwrap());
    let after = state
        .world
        .expect_with_world(|scene| scene.world_transform(active_camera).unwrap());
    assert_ne!(after, before);
    assert_eq!(viewport_camera_transform(&state), after);

    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(viewport_camera_transform(&state), before);

    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(viewport_camera_transform(&state), after);
}

#[test]
fn unrelated_transform_command_preserves_a_navigated_editor_camera() {
    let mut state = test_state();
    let (cube, _) = cube_and_camera(&state);
    let before = viewport_camera_transform(&state);

    state
        .handle_viewport_input(ViewportInput::RightPressed(Vec2::new(420.0, 310.0)))
        .unwrap();
    state
        .handle_viewport_input(ViewportInput::PointerMoved(Vec2::new(500.0, 365.0)))
        .unwrap();
    state
        .handle_viewport_input(ViewportInput::RightReleased)
        .unwrap();
    let navigated = viewport_camera_transform(&state);
    assert_ne!(navigated, before);

    let mut authored = state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().transform);
    authored.translation += Vec3::new(1.0, 2.0, 3.0);
    assert!(state
        .apply_intent(EditorIntent::SetTransform(cube, authored))
        .unwrap());

    assert_eq!(viewport_camera_transform(&state), navigated);
}

fn viewport_camera_transform(state: &EditorState) -> Transform {
    state
        .viewport_camera_snapshot()
        .unwrap()
        .expect("open scene camera")
        .transform
}

fn reset_viewport_from_scene(state: &mut EditorState) {
    let world = &state.world;
    let viewport = &mut state.viewport_controller;
    world.expect_with_world(|scene| viewport.reset_from_scene(Some(scene)));
}
