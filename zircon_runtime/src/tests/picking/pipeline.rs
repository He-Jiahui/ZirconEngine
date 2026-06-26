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
