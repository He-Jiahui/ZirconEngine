use super::*;

#[test]
fn picking_pipeline_report_counts_rays_outputs_and_hover_reduction() {
    let pointer = PointerId::new(1);
    let other_pointer = PointerId::new(2);
    let viewport = RenderViewportHandle::new(1);
    let mut ray_map = RayMap::default();
    let ray = ray_from_viewport_point(
        &test_camera(ProjectionMode::Perspective),
        UVec2::new(100, 100),
        Vec2::new(50.0, 50.0),
    )
    .expect("center pointer should produce a camera ray");
    ray_map.insert(RayId::new(7, pointer, viewport), ray);
    ray_map.insert(RayId::new(8, other_pointer, viewport), ray);

    let outputs = [
        PointerHits::new(
            pointer,
            vec![
                hit(HitTarget::scene_gizmo(10), 0.2).with_pickable(Pickable::NON_BLOCKING),
                hit(HitTarget::renderable(11), 0.3),
            ],
            0.0,
        ),
        PointerHits::new(pointer, vec![hit(HitTarget::renderable(12), 0.1)], 1.0),
    ];

    let report = PickingPipelineReport::from_ray_map_and_outputs(&ray_map, &outputs);

    assert_eq!(report.ray_count, 2);
    assert_eq!(report.pointer_count, 2);
    assert_eq!(report.backend_output_count, 2);
    assert_eq!(report.raw_hit_count, 3);
    assert_eq!(report.hovered_hit_count, 2);
    assert_eq!(report.blocked_pointer_count, 1);

    let pointer_report = report.pointer(pointer).expect("pointer report exists");
    assert_eq!(pointer_report.ray_count, 1);
    assert_eq!(pointer_report.backend_output_count, 2);
    assert_eq!(pointer_report.raw_hit_count, 3);
    assert_eq!(pointer_report.sorted_hit_count, 3);
    assert_eq!(pointer_report.hovered_hit_count, 2);
    assert_eq!(pointer_report.top_target, Some(HitTarget::scene_gizmo(10)));
    assert_eq!(
        pointer_report.blocking_target,
        Some(HitTarget::renderable(12))
    );

    let other_report = report
        .pointer(other_pointer)
        .expect("ray-only pointer report exists");
    assert_eq!(other_report.ray_count, 1);
    assert_eq!(other_report.backend_output_count, 0);
    assert_eq!(other_report.raw_hit_count, 0);
    assert_eq!(other_report.hovered_hit_count, 0);
    assert_eq!(other_report.top_target, None);
}

#[test]
fn picking_pipeline_report_exposes_blocking_non_hoverable_targets() {
    let pointer = PointerId::new(1);
    let outputs = [PointerHits::new(
        pointer,
        vec![
            hit(HitTarget::renderable(1), 0.1).with_pickable(Pickable::BLOCKING_NON_HOVERABLE),
            hit(HitTarget::renderable(2), 0.2),
        ],
        0.0,
    )];

    let report = PickingPipelineReport::from_outputs(&outputs);
    let pointer_report = report.pointer(pointer).expect("pointer report exists");

    assert_eq!(report.hovered_hit_count, 0);
    assert_eq!(report.blocked_pointer_count, 1);
    assert_eq!(pointer_report.non_hoverable_hit_count, 1);
    assert_eq!(
        pointer_report.blocking_target,
        Some(HitTarget::renderable(1))
    );
    assert_eq!(pointer_report.top_target, Some(HitTarget::renderable(1)));
}

#[test]
fn picking_debug_feed_exposes_summary_metrics_and_ray_only_rows() {
    let pointer = PointerId::new(1);
    let ray_only_pointer = PointerId::new(2);
    let viewport = RenderViewportHandle::new(1);
    let mut ray_map = RayMap::default();
    let ray = ray_from_viewport_point(
        &test_camera(ProjectionMode::Perspective),
        UVec2::new(100, 100),
        Vec2::new(50.0, 50.0),
    )
    .expect("center pointer should produce a camera ray");
    ray_map.insert(RayId::new(7, pointer, viewport), ray);
    ray_map.insert(RayId::new(8, ray_only_pointer, viewport), ray);

    let report = PickingPipelineReport::from_ray_map_and_outputs(
        &ray_map,
        &[PointerHits::new(
            pointer,
            vec![hit(HitTarget::renderable(3), 0.1)],
            0.0,
        )],
    );
    let feed = PickingDebugFeed::from_report(&report);

    assert_eq!(feed.metric(PickingDebugMetricKind::Pointers), Some(2));
    assert_eq!(feed.metric(PickingDebugMetricKind::Rays), Some(2));
    assert_eq!(feed.metric(PickingDebugMetricKind::RawHits), Some(1));
    assert_eq!(feed.metric(PickingDebugMetricKind::HoveredHits), Some(1));
    assert_eq!(
        feed.pointer(pointer).map(|row| row.top_target),
        Some(Some(HitTarget::renderable(3)))
    );

    let ray_only = feed
        .pointer(ray_only_pointer)
        .expect("ray-only pointer should remain visible in debug feed");
    assert_eq!(ray_only.ray_count, 1);
    assert_eq!(ray_only.raw_hit_count, 0);
    assert_eq!(ray_only.hovered_hit_count, 0);
    assert_eq!(ray_only.top_target, None);
}

#[test]
fn picking_debug_feed_lists_blocked_non_hoverable_pointers() {
    let pointer = PointerId::new(1);
    let report = PickingPipelineReport::from_outputs(&[PointerHits::new(
        pointer,
        vec![
            hit(HitTarget::renderable(1), 0.1).with_pickable(Pickable::BLOCKING_NON_HOVERABLE),
            hit(HitTarget::renderable(2), 0.2),
        ],
        0.0,
    )]);
    let feed = PickingDebugFeed::from_report(&report);
    let blocked = feed.blocked_pointers().collect::<Vec<_>>();

    assert_eq!(
        feed.metric(PickingDebugMetricKind::BlockedPointers),
        Some(1)
    );
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].pointer, pointer);
    assert_eq!(blocked[0].non_hoverable_hit_count, 1);
    assert_eq!(blocked[0].hovered_hit_count, 0);
    assert_eq!(blocked[0].blocking_target, Some(HitTarget::renderable(1)));
}
