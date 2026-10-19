//! 完整拾取调用验证阶段报告与下一帧事件状态；禁用分支须清空上帧交互，避免恢复时继承悬停或拖拽。
use super::*;

#[test]
fn picking_pipeline_runs_stages_and_carries_report() {
    let pointer = PointerId::new(1);
    let viewport = RenderViewportHandle::new(1);
    let location = PointerLocation::new(pointer, viewport, Vec2::new(50.0, 50.0));
    let camera = CameraRaySource::new(
        42,
        viewport,
        UVec2::new(100, 100),
        test_camera(ProjectionMode::Perspective),
    );
    let backend = PrimitivePickingBackend::new("pipeline-test").with_primitive(
        PickingPrimitive::sphere(HitTarget::renderable(9), Vec3::ZERO, 1.0),
    );
    let pointer_locations = [location];
    let pointer_inputs = [];
    let cameras = [camera];
    let backends: [&dyn PickingBackend; 1] = [&backend];
    let mut state = PickingEventState::default();

    let output = run_picking_pipeline(
        &mut state,
        PickingPipelineInput::new(&pointer_locations, &pointer_inputs, &cameras, &backends),
    );

    let labels = output
        .stages
        .iter()
        .map(|stage| stage.label)
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        vec![
            PickingScheduleLabel::Input,
            PickingScheduleLabel::RayMap,
            PickingScheduleLabel::Backend,
            PickingScheduleLabel::Hover,
            PickingScheduleLabel::Events,
        ]
    );
    assert_eq!(output.ray_map.len(), 1);
    assert_eq!(output.backend_outputs.len(), 1);
    assert_eq!(output.hover_map.get(pointer).len(), 1);
    assert_eq!(
        event_labels(&output.events),
        vec![PickingEventLabel::Enter, PickingEventLabel::Over]
    );
    assert_eq!(output.report.pointer_count, 1);
    assert_eq!(output.report.raw_hit_count, 1);
    assert_eq!(output.report.hovered_hit_count, 1);
}

#[test]
fn disabled_picking_pipeline_clears_previous_interaction_state() {
    let pointer = PointerId::new(1);
    let location = pointer_location(pointer, 10.0, 10.0);
    let target = HitTarget::renderable(1);
    let mut state = PickingEventState::default();

    state.dispatch_frame(
        PickingHoverMap::new(pointer, vec![hit(target, 0.1)]),
        &[location],
        &[PointerInput::new(
            location,
            PointerAction::Press(PointerButton::Primary),
        )],
    );
    assert!(state.previous_hover().is_hovered(pointer, target));

    let pointer_locations = [location];
    let pointer_inputs = [];
    let cameras = [];
    let backends = [];
    let output = run_picking_pipeline(
        &mut state,
        PickingPipelineInput::new(&pointer_locations, &pointer_inputs, &cameras, &backends)
            .with_settings(PickingSettings {
                enabled: false,
                ..PickingSettings::DEFAULT
            }),
    );

    assert!(state.previous_hover().is_empty());
    assert!(output.ray_map.is_empty());
    assert!(output.backend_outputs.is_empty());
    assert!(output.hover_map.is_empty());
    assert!(output.events.is_empty());
    assert!(output.stages.iter().all(|stage| !stage.enabled));
}

#[test]
fn same_pointer_viewports_keep_real_ray_hit_and_event_domains_separate() {
    use crate::core::framework::picking::{PickingDomain, PickingPointerKey};
    let pointer = PointerId::new(81);
    let a = RenderViewportHandle::new(801);
    let b = RenderViewportHandle::new(802);
    let locations = [
        PointerLocation::new(pointer, a, Vec2::new(50.0, 50.0)),
        PointerLocation::new(pointer, b, Vec2::new(50.0, 50.0)),
    ];
    let first = test_camera(ProjectionMode::Perspective);
    let mut second = test_camera(ProjectionMode::Perspective);
    second.transform = Transform::looking_at(
        Vec3::new(20.0, 0.0, 8.0),
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::Y,
    );
    let cameras = [
        CameraRaySource::new(42, a, UVec2::new(100, 100), first.clone()),
        CameraRaySource::new(43, a, UVec2::new(100, 100), first),
        CameraRaySource::new(42, b, UVec2::new(100, 100), second),
    ];
    let target_a = HitTarget::renderable(101);
    let target_b = HitTarget::renderable(202);
    let backend = PrimitivePickingBackend::new("real-domain-rays")
        .with_primitive(PickingPrimitive::sphere(target_a, Vec3::ZERO, 1.0))
        .with_primitive(PickingPrimitive::sphere(
            target_b,
            Vec3::new(20.0, 0.0, 0.0),
            1.0,
        ));
    let backends: [&dyn PickingBackend; 1] = [&backend];
    let inputs = locations
        .map(|location| PointerInput::new(location, PointerAction::Press(PointerButton::Primary)));
    let output = run_picking_pipeline(
        &mut PickingEventState::default(),
        PickingPipelineInput::new(&locations, &inputs, &cameras, &backends),
    );
    assert_eq!(output.ray_map.len(), 3);
    assert_eq!(output.backend_outputs.len(), 3);
    for batch in &output.backend_outputs {
        let target = match batch.domain {
            PickingDomain::Viewport(viewport) if viewport == a => target_a,
            PickingDomain::Viewport(viewport) if viewport == b => target_b,
            _ => panic!("real rays must retain their viewport"),
        };
        assert_eq!(batch.pointer, pointer);
        assert_eq!(batch.hits.len(), 1);
        assert_eq!(batch.hits[0].target, target);
    }
    let key_a = PickingPointerKey::in_viewport(pointer, a);
    let key_b = PickingPointerKey::in_viewport(pointer, b);
    assert_eq!(
        output
            .hover_map
            .get_key(key_a)
            .iter()
            .map(|hit| hit.target)
            .collect::<Vec<_>>(),
        vec![target_a]
    );
    assert_eq!(
        output
            .hover_map
            .get_key(key_b)
            .iter()
            .map(|hit| hit.target)
            .collect::<Vec<_>>(),
        vec![target_b]
    );
    assert!(output.hover_map.get(pointer).is_empty());
    assert_eq!(output.report.pointer_count, 1);
    assert_eq!(output.report.hovered_hit_count, 2);
    assert_eq!(output.report.blocked_pointer_count, 1);
    assert!(output.report.pointer(pointer).is_none());
    assert_eq!(
        output
            .report
            .pointer_in_domain(key_a)
            .unwrap()
            .sorted_hit_count,
        2
    );
    assert_eq!(
        output
            .report
            .pointer_in_domain(key_b)
            .unwrap()
            .sorted_hit_count,
        1
    );
    let debug = PickingDebugFeed::from_report(&output.report);
    assert!(debug.pointer(pointer).is_none());
    assert_eq!(
        debug.pointer_in_domain(key_a).unwrap().domain,
        PickingDomain::Viewport(a)
    );
    assert_eq!(
        debug.pointer_in_domain(key_b).unwrap().top_target,
        Some(target_b)
    );
    assert_eq!(
        output
            .events
            .iter()
            .filter(|event| matches!(event.kind, PickingEventKind::Press { .. }))
            .count(),
        2
    );
    for event in &output.events {
        assert_eq!(
            event.target,
            if event.location.viewport == a {
                target_a
            } else {
                target_b
            }
        );
        assert!(event.location.viewport == a || event.location.viewport == b);
    }
}
