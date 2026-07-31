use super::super::source_assertions::assert_source_order;
use super::sources::runtime_application_handler_source;

#[test]
fn runtime_entry_ticks_dynamic_runtime_time_before_redraw_request() {
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_frame_loop_source = include_str!("../../runtime_entry_app/frame_loop.rs");
    let runtime_session_source = include_str!("../../runtime_library/runtime_session.rs");
    let runtime_app_source = include_str!("../../runtime_entry_app/mod.rs");

    assert!(
        runtime_session_source.contains("pub(crate) fn tick_frame"),
        "runtime session should expose the required V3 dynamic runtime tick wrapper"
    );
    assert!(
        runtime_app_source.contains("frame_cadence: RuntimeFrameCadence"),
        "runtime entry app should store the profile-aware frame cadence state"
    );
    assert_source_order(
        runtime_handler_source,
        &["fn about_to_wait", "self.pump_frame_loop(event_loop);"],
        "runtime entry ApplicationHandler should delegate about-to-wait frame pumping",
    );
    assert!(
        runtime_app_source.contains("mod frame_loop;"),
        "runtime entry app should keep the frame pump in a child module"
    );
    assert_source_order(
        runtime_frame_loop_source,
        &[
            "fn pump_frame_loop",
            "let should_pump = self.frame_cadence.take_frame_request(now);",
            "self.apply_event_loop_policy(event_loop);",
            "if !should_pump {",
            "return;",
            "self.session.tick_frame()",
            "apply_runtime_demand",
            "self.session.wake_host();",
            "self.apply_runtime_host_requests(event_loop)",
            "window.request_redraw();",
        ],
        "runtime entry should suppress idle reactive frames, consume V3 demand, wake when required, then apply host requests before requesting redraw",
    );
}
