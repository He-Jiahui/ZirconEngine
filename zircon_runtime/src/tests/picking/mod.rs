use crate::core::framework::picking::{
    hovered_hits_for_pointer, ray_from_viewport_point, sorted_hits_for_pointer, CameraRaySource,
    HitData, HitRecord, HitTarget, Pickable, PickingAxis, PickingBackend, PickingEventKind,
    PickingEventLabel, PickingEventState, PickingHoverMap, PickingPrimitive, PointerAction,
    PointerButton, PointerHits, PointerId, PointerInput, PointerLocation, PointerScrollUnit,
    PrimitivePickingBackend, RayId, RayMap,
};
use crate::core::framework::render::{
    ProjectionMode, RenderViewportHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3};

#[test]
fn perspective_pointer_location_builds_center_camera_ray() {
    let camera = test_camera(ProjectionMode::Perspective);
    let viewport = UVec2::new(1280, 720);

    let ray = ray_from_viewport_point(&camera, viewport, Vec2::new(640.0, 360.0))
        .expect("center pointer should produce a camera ray");

    assert_eq!(ray.origin, camera.transform.translation);
    assert!(ray
        .direction
        .abs_diff_eq(camera.transform.forward(), 0.0001));
}

#[test]
fn ray_map_respects_pointer_viewport_and_camera_activity() {
    let pointer = PointerId::new(7);
    let viewport = RenderViewportHandle::new(2);
    let other_viewport = RenderViewportHandle::new(3);
    let mut ray_map = RayMap::default();

    ray_map.rebuild(
        &[
            PointerLocation::new(pointer, viewport, Vec2::new(320.0, 180.0)),
            PointerLocation::new(PointerId::new(8), other_viewport, Vec2::new(320.0, 180.0)),
        ],
        &[
            CameraRaySource::new(
                11,
                viewport,
                UVec2::new(640, 360),
                test_camera(ProjectionMode::Perspective),
            ),
            CameraRaySource::new(
                12,
                viewport,
                UVec2::new(640, 360),
                test_camera(ProjectionMode::Perspective),
            )
            .inactive(),
        ],
    );

    assert_eq!(ray_map.len(), 1);
    assert!(ray_map.get(&RayId::new(11, pointer, viewport)).is_some());
    assert!(ray_map.get(&RayId::new(12, pointer, viewport)).is_none());
    assert!(ray_map
        .get(&RayId::new(11, PointerId::new(8), other_viewport))
        .is_none());
}

#[test]
fn ray_generation_uses_actual_viewport_aspect_for_off_center_pointers() {
    let mut camera = test_camera(ProjectionMode::Perspective);
    camera.aspect_ratio = 16.0 / 9.0;
    let square_viewport = UVec2::new(100, 100);

    let ray = ray_from_viewport_point(&camera, square_viewport, Vec2::new(100.0, 50.0))
        .expect("right-edge pointer should produce a camera ray");

    let half_fov_tan = (camera.fov_y_radians * 0.5).tan();
    let expected = Vec3::new(half_fov_tan, 0.0, -1.0).normalize();
    assert!(ray.direction.abs_diff_eq(expected, 0.0001));
}

#[test]
fn hit_sorting_keeps_handle_gizmo_renderable_priority_before_depth() {
    let pointer = PointerId::new(1);
    let sorted = sorted_hits_for_pointer(
        &[PointerHits::new(
            pointer,
            vec![
                hit(HitTarget::renderable(30), 0.2),
                hit(HitTarget::scene_gizmo(20), 1.5),
                hit(HitTarget::handle_axis(10, PickingAxis::X), 4.0),
            ],
            0.0,
        )],
        pointer,
    );

    let targets: Vec<_> = sorted.into_iter().map(|hit| hit.target).collect();
    assert_eq!(
        targets,
        vec![
            HitTarget::handle_axis(10, PickingAxis::X),
            HitTarget::scene_gizmo(20),
            HitTarget::renderable(30),
        ]
    );
}

#[test]
fn hit_sorting_keeps_target_priority_before_backend_order() {
    let pointer = PointerId::new(1);
    let sorted = sorted_hits_for_pointer(
        &[
            PointerHits::new(
                pointer,
                vec![hit(HitTarget::handle_axis(10, PickingAxis::X), 4.0)],
                0.0,
            ),
            PointerHits::new(pointer, vec![hit(HitTarget::renderable(30), 0.1)], 10.0),
        ],
        pointer,
    );

    let targets: Vec<_> = sorted.into_iter().map(|hit| hit.target).collect();
    assert_eq!(
        targets,
        vec![
            HitTarget::handle_axis(10, PickingAxis::X),
            HitTarget::renderable(30),
        ]
    );
}

#[test]
fn hover_resolution_honors_non_hoverable_and_blocking_semantics() {
    let pointer = PointerId::new(1);
    let hovered = hovered_hits_for_pointer(
        &[PointerHits::new(
            pointer,
            vec![
                hit(HitTarget::handle_axis(1, PickingAxis::X), 0.1).with_pickable(Pickable::IGNORE),
                hit(HitTarget::scene_gizmo(2), 0.2).with_pickable(Pickable::NON_BLOCKING),
                hit(HitTarget::renderable(3), 0.3),
                hit(HitTarget::renderable(4), 0.4),
            ],
            0.0,
        )],
        pointer,
    );

    let targets: Vec<_> = hovered.into_iter().map(|hit| hit.target).collect();
    assert_eq!(
        targets,
        vec![HitTarget::scene_gizmo(2), HitTarget::renderable(3)]
    );
}

#[test]
fn primitive_backend_merges_multiple_ray_hits_by_existing_hover_rules() {
    let pointer = PointerId::new(1);
    let viewport = RenderViewportHandle::new(1);
    let mut ray_map = RayMap::default();
    ray_map.insert(
        RayId::new(99, pointer, viewport),
        ray_from_viewport_point(
            &test_camera(ProjectionMode::Perspective),
            UVec2::new(100, 100),
            Vec2::new(50.0, 50.0),
        )
        .expect("center pointer should produce a camera ray"),
    );

    let backend = PrimitivePickingBackend::new("test-primitives")
        .with_order(0.0)
        .with_primitive(PickingPrimitive::sphere(
            HitTarget::renderable(1),
            Vec3::new(0.0, 0.0, 0.0),
            0.75,
        ))
        .with_primitive(
            PickingPrimitive::sphere(HitTarget::scene_gizmo(2), Vec3::new(0.0, 0.0, 3.0), 0.75)
                .with_pickable(Pickable::NON_BLOCKING),
        );

    let outputs = backend.collect_hits(&ray_map);
    let hovered = hovered_hits_for_pointer(&outputs, pointer);
    let targets = hovered
        .into_iter()
        .map(|hit| hit.target)
        .collect::<Vec<_>>();

    assert_eq!(
        targets,
        vec![HitTarget::scene_gizmo(2), HitTarget::renderable(1)]
    );
}

#[test]
fn hover_map_builds_from_multiple_backend_outputs() {
    let pointer = PointerId::new(1);
    let hover_map = PickingHoverMap::from_outputs(&[
        PointerHits::new(pointer, vec![hit(HitTarget::renderable(3), 0.1)], 100.0),
        PointerHits::new(
            pointer,
            vec![hit(HitTarget::handle_axis(1, PickingAxis::Y), 4.0)],
            0.0,
        ),
    ]);

    let targets = hover_map
        .get(pointer)
        .iter()
        .map(|hit| hit.target)
        .collect::<Vec<_>>();
    assert_eq!(targets, vec![HitTarget::handle_axis(1, PickingAxis::Y)]);
}

#[test]
fn pointer_event_state_emits_hover_transitions_before_move() {
    let pointer = PointerId::new(1);
    let location = pointer_location(pointer, 10.0, 10.0);
    let mut state = PickingEventState::default();
    let current = PickingHoverMap::new(pointer, vec![hit(HitTarget::renderable(1), 0.1)]);

    let events = state.dispatch_frame(
        current,
        &[location],
        &[PointerInput::new(
            location,
            PointerAction::Move {
                delta: Vec2::new(1.0, 0.0),
            },
        )],
    );

    assert_eq!(
        event_labels(&events),
        vec![
            PickingEventLabel::Enter,
            PickingEventLabel::Over,
            PickingEventLabel::Move,
        ]
    );
    assert_eq!(events[0].propagate, false);
    assert_eq!(events[1].target, HitTarget::renderable(1));
}

#[test]
fn pointer_event_state_click_release_use_previous_hover() {
    let pointer = PointerId::new(1);
    let start = pointer_location(pointer, 10.0, 10.0);
    let release = pointer_location(pointer, 90.0, 90.0);
    let target = HitTarget::renderable(1);
    let mut state = PickingEventState::default();

    let first = state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(target, 0.1)]),
        &[start],
        &[PointerInput::new(
            start,
            PointerAction::Press(PointerButton::Primary),
        )],
    );
    assert_eq!(
        event_labels(&first),
        vec![
            PickingEventLabel::Enter,
            PickingEventLabel::Over,
            PickingEventLabel::Press,
        ]
    );

    let second = state.dispatch_frame(
        PickingHoverMap::default(),
        &[release],
        &[PointerInput::new(
            release,
            PointerAction::Release(PointerButton::Primary),
        )],
    );

    assert_eq!(
        event_labels(&second),
        vec![
            PickingEventLabel::Out,
            PickingEventLabel::Leave,
            PickingEventLabel::Click,
            PickingEventLabel::Release,
        ]
    );
    assert!(second.iter().all(|event| event.target == target));
}

#[test]
fn pointer_event_state_drag_drop_and_scroll_sequence() {
    let pointer = PointerId::new(1);
    let dragged = HitTarget::handle_axis(1, PickingAxis::X);
    let drop_target = HitTarget::renderable(2);
    let start = pointer_location(pointer, 10.0, 10.0);
    let drag_location = pointer_location(pointer, 20.0, 10.0);
    let release = pointer_location(pointer, 25.0, 10.0);
    let mut state = PickingEventState::default();

    state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(dragged, 0.1)]),
        &[start],
        &[PointerInput::new(
            start,
            PointerAction::Press(PointerButton::Primary),
        )],
    );

    let drag_events = state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(dragged, 0.1), hit(drop_target, 0.2)]),
        &[drag_location],
        &[PointerInput::new(
            drag_location,
            PointerAction::Move {
                delta: Vec2::new(10.0, 0.0),
            },
        )],
    );
    assert_eq!(
        event_labels(&drag_events),
        vec![
            PickingEventLabel::Enter,
            PickingEventLabel::Over,
            PickingEventLabel::DragStart,
            PickingEventLabel::DragEnter,
            PickingEventLabel::Drag,
            PickingEventLabel::DragOver,
            PickingEventLabel::Move,
            PickingEventLabel::Move,
        ]
    );
    assert!(drag_events.iter().any(|event| matches!(
        event.kind,
        PickingEventKind::DragOver { dragged: target, .. } if target == dragged
    )));

    let release_events = state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(drop_target, 0.1)]),
        &[release],
        &[
            PointerInput::new(
                release,
                PointerAction::Scroll {
                    unit: PointerScrollUnit::Pixel,
                    delta: Vec2::new(0.0, -4.0),
                },
            ),
            PointerInput::new(release, PointerAction::Release(PointerButton::Primary)),
        ],
    );

    assert_eq!(
        event_labels(&release_events),
        vec![
            PickingEventLabel::Out,
            PickingEventLabel::Leave,
            PickingEventLabel::DragLeave,
            PickingEventLabel::Scroll,
            PickingEventLabel::Click,
            PickingEventLabel::Release,
            PickingEventLabel::Release,
            PickingEventLabel::DragDrop,
            PickingEventLabel::DragEnd,
            PickingEventLabel::DragLeave,
        ]
    );
    assert!(release_events.iter().any(|event| matches!(
        event.kind,
        PickingEventKind::DragDrop { dropped: target, .. } if target == dragged
    )));
}

#[test]
fn pointer_event_state_cancel_filters_current_hover_and_clears_state() {
    let pointer = PointerId::new(1);
    let previous_target = HitTarget::renderable(1);
    let current_target = HitTarget::renderable(2);
    let start = pointer_location(pointer, 10.0, 10.0);
    let cancel = pointer_location(pointer, 20.0, 20.0);
    let mut state = PickingEventState::default();

    state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(previous_target, 0.1)]),
        &[start],
        &[PointerInput::new(
            start,
            PointerAction::Press(PointerButton::Primary),
        )],
    );

    let cancel_events = state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(current_target, 0.1)]),
        &[cancel],
        &[PointerInput::new(cancel, PointerAction::Cancel)],
    );

    assert_eq!(
        event_labels(&cancel_events),
        vec![
            PickingEventLabel::Out,
            PickingEventLabel::Leave,
            PickingEventLabel::Cancel,
        ]
    );
    assert!(cancel_events
        .iter()
        .all(|event| event.target == previous_target));

    let release_after_cancel = state.dispatch_frame(
        PickingHoverMap::default(),
        &[cancel],
        &[PointerInput::new(
            cancel,
            PointerAction::Release(PointerButton::Primary),
        )],
    );
    assert!(release_after_cancel.is_empty());
}

fn hit(target: HitTarget, depth: f32) -> HitRecord {
    HitRecord::new(target, HitData::new(99, depth, None, None))
}

fn pointer_location(pointer: PointerId, x: f32, y: f32) -> PointerLocation {
    PointerLocation::new(pointer, RenderViewportHandle::new(1), Vec2::new(x, y))
}

fn event_labels(
    events: &[crate::core::framework::picking::PickingPointerEvent],
) -> Vec<PickingEventLabel> {
    events.iter().map(|event| event.label()).collect()
}

fn test_camera(projection_mode: ProjectionMode) -> ViewportCameraSnapshot {
    ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.0, 8.0), Vec3::ZERO, Vec3::Y),
        projection_mode,
        fov_y_radians: 60.0_f32.to_radians(),
        ortho_size: 5.0,
        z_near: 0.1,
        z_far: 200.0,
        aspect_ratio: 16.0 / 9.0,
        ..ViewportCameraSnapshot::default()
    }
}
