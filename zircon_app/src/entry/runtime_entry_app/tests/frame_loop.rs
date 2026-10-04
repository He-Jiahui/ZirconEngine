#[test]
fn frame_pump_publishes_only_the_final_control_flow() {
    let source = include_str!("../frame_loop.rs");
    let pump_start = source
        .find("pub(super) fn pump_frame_loop")
        .expect("frame pump owner");
    let request_start = source[pump_start..]
        .find("pub(super) fn request_runtime_frame")
        .map(|offset| pump_start + offset)
        .expect("frame request owner after pump");
    let pump_body = &source[pump_start..request_start];

    assert_eq!(
        pump_body
            .matches("self.apply_event_loop_policy(event_loop);")
            .count(),
        1,
        "each frame pump must publish only its final control flow",
    );
}

#[test]
fn frame_pump_keeps_the_p1_cadence_measurement_points() {
    let source = include_str!("../frame_loop.rs");

    for name in [
        "runtime_entry.frame_pump",
        "runtime_entry.frame_pump_suppressed",
        "runtime_entry.runtime_tick",
        "runtime_entry.redraw_request",
    ] {
        assert!(
            source.contains(name),
            "P1 cadence reporting must retain the `{name}` counter"
        );
    }
}
