use super::super::source_assertions::assert_source_order;
use super::sources::{entry_root, runtime_event_loop_policy_source};

#[test]
fn runtime_entry_maps_platform_event_loop_policy_to_winit_control_flow() {
    let event_loop_policy_root_source =
        include_str!("../../runtime_entry_app/event_loop_policy/mod.rs");
    let event_loop_policy_control_flow_source =
        include_str!("../../runtime_entry_app/event_loop_policy/control_flow.rs");
    let frame_cadence_source =
        include_str!("../../runtime_entry_app/event_loop_policy/frame_cadence.rs");
    let event_loop_policy_source = format!(
        "{}\n{}",
        runtime_event_loop_policy_source(),
        frame_cadence_source
    );
    let runtime_app_source = include_str!("../../runtime_entry_app/mod.rs");
    let root = entry_root();

    for required in [
        "EventLoopPolicy::Game",
        "EventLoopPolicy::Continuous",
        "EventLoopPolicy::DesktopApp",
        "EventLoopPolicy::Mobile",
        "EventLoopPolicy::Headless",
        "ControlFlow::Poll",
        "ControlFlow::Wait,",
        "ControlFlow::WaitUntil",
        "event_loop.set_control_flow",
        "pub(in crate::entry::runtime_entry_app) fn apply_event_loop_policy",
    ] {
        assert!(
            event_loop_policy_source.contains(required),
            "runtime event-loop policy helper should preserve `{required}`"
        );
    }
    assert!(
        runtime_app_source.contains("mod event_loop_policy;"),
        "runtime entry app should keep event-loop policy mapping in a child module"
    );
    assert!(
        event_loop_policy_root_source.contains("mod control_flow;")
            && event_loop_policy_root_source.contains("mod frame_cadence;")
            && event_loop_policy_root_source.contains(
                "pub(in crate::entry::runtime_entry_app) use frame_cadence::RuntimeFrameCadence;"
            ),
        "runtime event-loop policy root should delegate control-flow and cadence behavior"
    );
    assert!(
        !root.join("runtime_entry_app/event_loop_policy.rs").exists(),
        "runtime event-loop policy should stay folder-backed instead of returning to an umbrella event_loop_policy.rs file"
    );
    assert_source_order(
        event_loop_policy_control_flow_source,
        &[
            "fn apply_event_loop_policy",
            "self.frame_cadence.control_flow",
            "event_loop.set_control_flow",
        ],
        "event-loop policy control-flow helper should delegate to the cadence owner",
    );
    for required in [
        "HEADLESS_FRAME_INTERVAL",
        "UNFOCUSED_GAME_FRAME_INTERVAL",
        "MOBILE_FOREGROUND_FRAME_INTERVAL",
        "BACKGROUND_FRAME_INTERVAL",
        "RuntimeFrameCadenceMode::Continuous",
        "RuntimeFrameCadenceMode::Reactive",
        "RuntimeFrameCadenceMode::LowPower",
        "RuntimeFrameCadenceMode::FixedInterval",
        "fn request_frame",
        "fn apply_runtime_demand",
        "fn take_frame_request",
        "fn control_flow",
        "fn set_window_focused",
        "fn set_window_occluded",
        "frame_requests_accepted",
        "frame_requests_coalesced",
        "frame_requests_ignored",
        "reactive_cadence_coalesces_requests_and_suppresses_idle_frames",
        "foreground_game_and_explicit_continuous_cadence_never_suppress_frame_pumps",
        "game_cadence_throttles_unfocused_and_occluded_windows",
        "mobile_cadence_has_explicit_foreground_and_background_limits",
        "explicit_continuous_profile_ignores_visibility_throttling",
        "low_power_cadence_consumes_runtime_immediate_and_after_demand",
        "headless_cadence_uses_fixed_wait_deadlines",
        "headless_early_wake_does_not_pump_or_move_fixed_deadline",
        "reactive_runtime_immediate_demand_coalesces_one_host_wake",
        "reactive_runtime_after_replaces_and_idle_cancels_previous_deadline",
        "continuous_cadence_does_not_schedule_extra_wakes_from_runtime_demand",
    ] {
        assert!(
            frame_cadence_source.contains(required),
            "runtime frame-cadence owner should preserve `{required}`"
        );
    }
}
