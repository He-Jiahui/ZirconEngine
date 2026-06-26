use super::*;

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
