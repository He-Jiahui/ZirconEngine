//! 命中优先级先于后端顺序和深度；共享排序计数与存储身份断言验证单次投影及写时分离，不依赖墙钟时间。
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

#[test]
fn picking_output_resolution_shares_one_projection_between_hover_and_report() {
    let first_pointer = PointerId::new(1);
    let second_pointer = PointerId::new(2);
    let outputs = [
        PointerHits::new(
            first_pointer,
            vec![
                hit(HitTarget::handle_axis(1, PickingAxis::X), 0.1).with_pickable(Pickable::IGNORE),
                hit(HitTarget::renderable(3), 0.3),
                hit(HitTarget::renderable(4), 0.4),
            ],
            0.0,
        ),
        PointerHits::new(
            first_pointer,
            vec![hit(HitTarget::scene_gizmo(2), 0.2).with_pickable(Pickable::NON_BLOCKING)],
            1.0,
        ),
        PointerHits::new(
            second_pointer,
            vec![hit(HitTarget::renderable(5), 0.5)],
            0.0,
        ),
    ];

    crate::core::framework::picking::reset_sorted_hit_projection_metrics();
    let (hover_map, report) = crate::core::framework::picking::resolve_picking_outputs(&outputs);
    let (projection_builds, pointer_group_sorts) =
        crate::core::framework::picking::sorted_hit_projection_metrics();

    assert_eq!(projection_builds, 1);
    assert_eq!(pointer_group_sorts, 2);
    assert_eq!(
        hover_map
            .get(first_pointer)
            .iter()
            .map(|hit| hit.target)
            .collect::<Vec<_>>(),
        vec![HitTarget::scene_gizmo(2), HitTarget::renderable(3)]
    );
    let first_report = report
        .pointer(first_pointer)
        .expect("first pointer should be represented in the report");
    assert_eq!(first_report.backend_output_count, 2);
    assert_eq!(first_report.sorted_hit_count, 4);
    assert_eq!(first_report.hovered_hit_count, 2);
    assert_eq!(first_report.non_hoverable_hit_count, 1);
    assert_eq!(first_report.blocking_target, Some(HitTarget::renderable(3)));
    assert_eq!(hover_map.get(second_pointer).len(), 1);
    assert_eq!(report.pointer(second_pointer).unwrap().hovered_hit_count, 1);
}

#[test]
fn picking_pipeline_shares_one_sorted_hit_projection_between_hover_and_report() {
    let pointers = [PointerId::new(1), PointerId::new(2)];
    let viewport = RenderViewportHandle::new(1);
    let pointer_locations =
        pointers.map(|pointer| PointerLocation::new(pointer, viewport, Vec2::new(50.0, 50.0)));
    let cameras = [CameraRaySource::new(
        42,
        viewport,
        UVec2::new(100, 100),
        test_camera(ProjectionMode::Perspective),
    )];
    let backend = PrimitivePickingBackend::new("shared-projection-test").with_primitive(
        PickingPrimitive::sphere(HitTarget::renderable(9), Vec3::ZERO, 1.0),
    );
    let backends: [&dyn PickingBackend; 1] = [&backend];
    let mut state = PickingEventState::default();

    crate::core::framework::picking::reset_sorted_hit_projection_metrics();
    let output = run_picking_pipeline(
        &mut state,
        PickingPipelineInput::new(&pointer_locations, &[], &cameras, &backends),
    );
    let (projection_builds, pointer_group_sorts) =
        crate::core::framework::picking::sorted_hit_projection_metrics();

    assert_eq!(projection_builds, 1);
    assert_eq!(pointer_group_sorts, pointers.len());
    assert!(
        output.hover_map.shares_storage_with(state.previous_hover()),
        "pipeline output and next-frame event state should share hover storage"
    );
    assert_eq!(output.report.pointer_count, pointers.len());
    assert_eq!(output.report.raw_hit_count, pointers.len());
    assert_eq!(output.report.hovered_hit_count, pointers.len());
    for pointer in pointers {
        assert_eq!(output.hover_map.get(pointer).len(), 1);
        let pointer_report = output
            .report
            .pointer(pointer)
            .expect("each hovered pointer must be represented in the report");
        assert_eq!(pointer_report.sorted_hit_count, 1);
        assert_eq!(pointer_report.hovered_hit_count, 1);
        assert_eq!(pointer_report.top_target, Some(HitTarget::renderable(9)));
    }

    state.clear_pointer(pointers[0]);
    assert!(state.previous_hover().get(pointers[0]).is_empty());
    assert_eq!(output.hover_map.get(pointers[0]).len(), 1);
}

#[test]
fn legacy_and_viewport_hit_domains_do_not_merge_or_fallback() {
    use crate::core::framework::picking::{hovered_hits_for_key, PickingPointerKey};
    let pointer = PointerId::new(83);
    let a = RenderViewportHandle::new(821);
    let b = RenderViewportHandle::new(822);
    let absent = RenderViewportHandle::new(823);
    let legacy = HitTarget::renderable(10);
    let target_a = HitTarget::renderable(20);
    let target_b = HitTarget::renderable(30);
    let outputs = [
        PointerHits::new(pointer, vec![hit(legacy, 0.01)], 100.0),
        PointerHits::in_viewport(pointer, a, vec![hit(target_a, 0.1)], 0.0),
        PointerHits::in_viewport(pointer, b, vec![hit(target_b, 0.2)], 0.0),
    ];
    let legacy_key = PickingPointerKey::legacy(pointer);
    let key_a = PickingPointerKey::in_viewport(pointer, a);
    let key_b = PickingPointerKey::in_viewport(pointer, b);
    let absent_key = PickingPointerKey::in_viewport(pointer, absent);
    let hover = PickingHoverMap::from_outputs(&outputs);
    assert!(sorted_hits_for_pointer(&outputs, pointer).is_empty());
    assert!(hovered_hits_for_pointer(&outputs, pointer).is_empty());
    assert!(hover.get(pointer).is_empty());
    assert!(hover.hit(pointer, legacy).is_none());
    for (key, target) in [(legacy_key, legacy), (key_a, target_a), (key_b, target_b)] {
        assert_eq!(
            hover
                .get_key(key)
                .iter()
                .map(|hit| hit.target)
                .collect::<Vec<_>>(),
            vec![target]
        );
        assert_eq!(
            hovered_hits_for_key(&outputs, key)
                .iter()
                .map(|hit| hit.target)
                .collect::<Vec<_>>(),
            vec![target]
        );
        assert!(hover.is_hovered_key(key, target));
    }
    assert!(hover.get_key(absent_key).is_empty());
    assert!(hovered_hits_for_key(&outputs, absent_key).is_empty());
    let location = PointerLocation::new(pointer, absent, Vec2::new(1.0, 1.0));
    let input = PointerInput::new(location, PointerAction::Press(PointerButton::Primary));
    let scoped =
        PickingEventState::default().dispatch_viewport_frame(hover.clone(), &[location], &[input]);
    assert!(
        scoped.is_empty(),
        "an absent viewport cannot consume legacy or another viewport"
    );
    let legacy_events = PickingEventState::default().dispatch_frame(hover, &[location], &[input]);
    assert!(!legacy_events.is_empty());
    assert!(legacy_events.iter().all(|event| event.target == legacy));
}
