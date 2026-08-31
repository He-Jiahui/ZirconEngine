#[test]
fn preview_transport_stays_in_its_dedicated_owner() {
    let controller = include_str!("../controller.rs");
    let preview_routing = include_str!("preview_routing.rs");

    assert!(controller.contains("mod preview_routing;"));
    for method in [
        "pub fn capture_preview_frame(",
        "pub fn route_preview_input(",
        "pub fn route_simulate_camera(",
    ] {
        assert!(
            !controller.contains(method),
            "{method} drifted into controller"
        );
        assert!(
            preview_routing.contains(method),
            "{method} is missing from preview routing"
        );
    }
    assert_eq!(
        preview_routing.matches("handle_event_at_identity").count(),
        2,
        "Play input and SIE camera must both bind dispatch to the sampled gateway identity"
    );
}

#[test]
fn runtime_ownership_stays_in_its_dedicated_owner() {
    let controller = include_str!("../controller.rs");
    let runtime_ownership = include_str!("runtime_ownership.rs");

    assert!(controller.contains("mod runtime_ownership;"));
    for method in [
        "pub(crate) fn detach_terminal_play_gateway<",
        "pub fn attached_world_domain(",
        "pub fn terminal_backend_retirement_pending(",
        "pub fn play_gateway(",
        "pub fn retire_terminal_backend(",
    ] {
        assert!(
            !controller.contains(method),
            "{method} drifted into controller"
        );
        assert!(
            runtime_ownership.contains(method),
            "{method} is missing from runtime ownership"
        );
    }
    assert!(!runtime_ownership.contains("attach_play_gateway"));
}

#[test]
fn inactive_backend_poll_reads_the_mode_once() {
    let source = include_str!("../controller.rs");
    let body = source
        .split("pub fn poll_backend")
        .nth(1)
        .and_then(|body| body.split("pub fn route_edit").next())
        .expect("poll backend body should remain available");

    assert_eq!(body.matches("self.mode()").count(), 1);
}
