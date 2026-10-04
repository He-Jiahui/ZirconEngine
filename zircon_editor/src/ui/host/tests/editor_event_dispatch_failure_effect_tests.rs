use super::*;

#[test]
fn viewport_failure_invalidates_render_and_presentation() {
    let effects = failure_effects_for_event(&EditorEvent::Viewport(
        crate::core::editor_event::EditorViewportEvent::LeftReleased,
    ));

    assert!(effects.contains(&EditorEventEffect::RenderChanged));
    assert!(effects.contains(&EditorEventEffect::PresentationChanged));
    assert!(effects.contains(&EditorEventEffect::ReflectionChanged));
}

#[test]
fn mvp_authoring_trace_keeps_binding_operation_and_generation_correlation() {
    let trace = mvp_authoring_product_trace_diagnostic(
        "completed",
        "inspector",
        Some("Inspector/TransformPositionXCommit"),
        Some("inspector.transform.position.x.commit"),
        Some(42),
        Some(8),
    );

    assert!(trace.contains("result=completed"));
    assert!(trace.contains("event=inspector"));
    assert!(trace.contains("binding=Inspector/TransformPositionXCommit"));
    assert!(trace.contains("operation=inspector.transform.position.x.commit"));
    assert!(trace.contains("transaction_id=42"));
    assert!(trace.contains("save_generation=8"));
    assert_eq!(
        mvp_authoring_trace_event_kind(&EditorEvent::WorkbenchMenu(MenuAction::SaveProject)),
        Some("save_project")
    );
}
