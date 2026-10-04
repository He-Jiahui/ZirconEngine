use super::*;
use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::projection::project_point;
use crate::scene::viewport::{HandleElementExtract, OverlayAxis, TransformHandleKind};
use zircon_runtime::scene::{components::NodeKind, DefaultLevelManager};
use zircon_runtime_interface::math::UVec2;

#[test]
fn current_extract_navigation_keeps_its_button_owner_through_foreign_edges() {
    assert_navigation_button_ownership(false);
}

#[test]
fn stale_extract_navigation_keeps_its_button_owner_through_foreign_edges() {
    assert_navigation_button_ownership(true);
}

fn assert_navigation_button_ownership(stale: bool) {
    for orbit in [true, false] {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        let mut scene = DefaultLevelManager::default().create_default_level();
        let start = Vec2::new(320.0, 180.0);
        let (pressed, released, other_pressed, other_released) = if orbit {
            (
                ViewportInput::RightPressed(start),
                ViewportInput::RightReleased,
                ViewportInput::MiddlePressed(start),
                ViewportInput::MiddleReleased,
            )
        } else {
            (
                ViewportInput::MiddlePressed(start),
                ViewportInput::MiddleReleased,
                ViewportInput::RightPressed(start),
                ViewportInput::RightReleased,
            )
        };
        scene
            .with_world_mut(|world| controller.handle_input(world, pressed))
            .unwrap();
        scene.with_world(|world| controller.build_render_snapshot(world));
        if stale {
            scene
                .with_world_mut(|world| world.spawn_node(NodeKind::Empty))
                .unwrap();
        }
        let camera_before = scene.with_world(|world| controller.current_camera(world));
        scene
            .with_world_mut(|world| {
                controller.handle_input(
                    world,
                    ViewportInput::LeftPressed {
                        position: start,
                        selection_mutation: SelectionMutation::Replace,
                    },
                )
            })
            .unwrap();
        assert_navigation_owner(&controller, orbit);
        let after_down = scene
            .with_world_mut(|world| {
                controller.handle_input(
                    world,
                    ViewportInput::PointerMoved(start + Vec2::new(24.0, 12.0)),
                )
            })
            .unwrap();
        assert!(after_down.camera_updated);
        assert_ne!(
            scene.with_world(|world| controller.current_camera(world)),
            camera_before
        );

        for foreign in [ViewportInput::LeftReleased, other_pressed, other_released] {
            scene
                .with_world_mut(|world| controller.handle_input(world, foreign))
                .unwrap();
            assert_navigation_owner(&controller, orbit);
        }
        let after_up = scene
            .with_world_mut(|world| {
                controller.handle_input(
                    world,
                    ViewportInput::PointerMoved(start + Vec2::new(48.0, 24.0)),
                )
            })
            .unwrap();
        assert!(after_up.camera_updated);
        scene
            .with_world_mut(|world| controller.handle_input(world, released))
            .unwrap();
        assert!(controller.state.drag.is_none());
        let camera_after_release = scene.with_world(|world| controller.current_camera(world));
        let idle = scene
            .with_world_mut(|world| {
                controller.handle_input(
                    world,
                    ViewportInput::PointerMoved(start + Vec2::new(72.0, 36.0)),
                )
            })
            .unwrap();
        assert!(!idle.camera_updated);
        assert_eq!(
            scene.with_world(|world| controller.current_camera(world)),
            camera_after_release
        );
    }
}

fn assert_navigation_owner(controller: &SceneViewportController, orbit: bool) {
    assert!(if orbit {
        matches!(
            controller.state.drag,
            Some(ViewportDragSession::Orbit { .. })
        )
    } else {
        matches!(controller.state.drag, Some(ViewportDragSession::Pan { .. }))
    });
}

#[test]
fn primary_selection_keeps_its_drag_through_secondary_and_middle_edges() {
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    let mut scene = DefaultLevelManager::default().create_default_level();
    scene.with_world(|world| controller.build_render_snapshot(world));
    let start = Vec2::new(4.0, 4.0);
    scene
        .with_world_mut(|world| {
            controller.handle_input(
                world,
                ViewportInput::LeftPressed {
                    position: start,
                    selection_mutation: SelectionMutation::Replace,
                },
            )
        })
        .unwrap();
    let camera = scene.with_world(|world| controller.current_camera(world));
    for foreign in navigation_edges(start) {
        scene
            .with_world_mut(|world| controller.handle_input(world, foreign))
            .unwrap();
        assert!(matches!(
            controller.state.drag,
            Some(ViewportDragSession::PrimarySelection { .. })
        ));
    }
    let moved = start + Vec2::new(48.0, 24.0);
    scene
        .with_world_mut(|world| controller.handle_input(world, ViewportInput::PointerMoved(moved)))
        .unwrap();
    assert!(matches!(
        controller.state.drag,
        Some(ViewportDragSession::PrimarySelection { current, active: true, .. })
            if current == moved
    ));
    assert_eq!(
        scene.with_world(|world| controller.current_camera(world)),
        camera
    );
    scene
        .with_world_mut(|world| controller.handle_input(world, ViewportInput::LeftReleased))
        .unwrap();
    assert!(controller.state.drag.is_none());
}

#[test]
fn primary_handle_keeps_preview_requests_through_foreign_navigation_edges() {
    let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
    let mut scene = DefaultLevelManager::default().create_default_level();
    let cube = scene.with_world(|world| {
        world
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id
    });
    controller.selection_mut().select_only_active(cube);
    controller
        .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move))
        .unwrap();
    let packet = scene.with_world(|world| controller.build_render_snapshot(world));
    let (axis_start, axis_end) = packet
        .overlays
        .handles
        .iter()
        .find(|handle| handle.owner == cube)
        .unwrap()
        .elements
        .iter()
        .find_map(|element| match element {
            HandleElementExtract::AxisLine {
                axis: OverlayAxis::X,
                start,
                end,
                ..
            } => Some((*start, *end)),
            _ => None,
        })
        .unwrap();
    let viewport = controller.viewport().size;
    let axis_start = project_point(axis_start, &packet.scene.camera, viewport).unwrap();
    let axis_end = project_point(axis_end, &packet.scene.camera, viewport).unwrap();
    let direction = (axis_end - axis_start).normalize();
    let press = axis_start + direction * 24.0;
    scene
        .with_world_mut(|world| {
            controller.handle_input(
                world,
                ViewportInput::LeftPressed {
                    position: press,
                    selection_mutation: SelectionMutation::Replace,
                },
            )
        })
        .unwrap();
    assert!(controller.is_handle_drag_active());
    for foreign in navigation_edges(press) {
        scene
            .with_world_mut(|world| controller.handle_input(world, foreign))
            .unwrap();
        assert!(controller.is_handle_drag_active());
    }
    let preview = scene
        .with_world_mut(|world| {
            controller.handle_input(world, ViewportInput::PointerMoved(press + direction * 96.0))
        })
        .unwrap()
        .transform_request
        .expect("the original handle must still publish a transaction preview");
    assert_eq!(preview.primary, cube);
    scene
        .with_world_mut(|world| controller.handle_input(world, ViewportInput::LeftReleased))
        .unwrap();
    assert!(!controller.is_handle_drag_active());
}

fn navigation_edges(position: Vec2) -> [ViewportInput; 4] {
    [
        ViewportInput::RightPressed(position),
        ViewportInput::RightReleased,
        ViewportInput::MiddlePressed(position),
        ViewportInput::MiddleReleased,
    ]
}
