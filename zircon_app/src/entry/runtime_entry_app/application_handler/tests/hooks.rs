#[test]
fn surface_ownership_is_confirmed_before_input_probe_controls_initial_frame_scheduling() {
    let source = include_str!("../../application_lifecycle/events.rs");
    let surface_created = source
        .find("if self.create_primary_window_surface(event_loop) {")
        .expect("surface availability should create the primary window only after winit admission");
    let ownership_confirmed = source
        .find("self.application_lifecycle.confirm_surface_created();")
        .expect("successful native creation should immediately update lifecycle ownership");
    let input_probe = source
        .find("if self.submit_mvp_input_probe_if_requested(event_loop) {")
        .expect("input probe should continue to gate the initial frame");
    let frame_requested = source
        .find("self.request_runtime_frame();")
        .expect("input probe success should schedule the initial frame");

    assert!(surface_created < ownership_confirmed);
    assert!(ownership_confirmed < input_probe);
    assert!(input_probe < frame_requested);
}
