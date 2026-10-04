use super::*;

#[test]
fn reactive_cadence_coalesces_requests_and_suppresses_idle_frames() {
    let mut cadence = RuntimeFrameCadence::new(EventLoopPolicy::DesktopApp);
    let now = Instant::now();

    assert_eq!(cadence.control_flow(), ControlFlow::Poll);
    assert!(cadence.take_frame_request(now));
    assert_eq!(cadence.control_flow(), ControlFlow::Wait);
    assert!(!cadence.take_frame_request(now));
    cadence.request_frame();
    cadence.request_frame();
    assert!(cadence.take_frame_request(now));
    assert!(!cadence.take_frame_request(now));

    assert_eq!(cadence.report().frame_requests, 3);
    assert_eq!(cadence.report().frame_requests_accepted, 2);
    assert_eq!(cadence.report().frame_requests_coalesced, 1);
    assert_eq!(cadence.report().frame_requests_ignored, 0);
    assert_eq!(cadence.report().frame_pumps, 2);
    assert_eq!(cadence.report().idle_pumps_suppressed, 2);
}

#[test]
fn reactive_pending_request_survives_runtime_idle_until_next_pump() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::DesktopApp, now);
    assert!(cadence.take_frame_request(now));

    assert!(cadence.request_frame());
    assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::Idle));
    assert_eq!(cadence.control_flow(), ControlFlow::Poll);
    assert!(cadence.take_frame_request(now));
    assert_eq!(cadence.control_flow(), ControlFlow::Wait);
}

#[test]
fn reactive_pending_request_still_polls_when_runtime_immediate_coalesces() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::DesktopApp, now);
    assert!(cadence.take_frame_request(now));

    assert!(cadence.request_frame());
    assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
    assert_eq!(cadence.control_flow(), ControlFlow::Poll);
    assert!(cadence.take_frame_request(now));
    assert_eq!(cadence.control_flow(), ControlFlow::Wait);
    assert_eq!(cadence.report().frame_requests_coalesced, 1);
}

#[test]
fn reactive_runtime_immediate_demand_coalesces_one_host_wake() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::DesktopApp, now);
    assert!(cadence.take_frame_request(now));

    assert!(cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
    assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
    assert!(cadence.take_frame_request(now));
    assert!(!cadence.take_frame_request(now));
}

#[test]
fn reactive_runtime_after_replaces_and_idle_cancels_previous_deadline() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::DesktopApp, now);
    assert!(cadence.take_frame_request(now));

    assert!(
        !cadence.apply_runtime_demand(now, RuntimeFrameDemand::After(Duration::from_millis(40)),)
    );
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(now + Duration::from_millis(40))
    );

    assert!(
        !cadence.apply_runtime_demand(now, RuntimeFrameDemand::After(Duration::from_millis(80)),)
    );
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(now + Duration::from_millis(80)),
        "a new runtime snapshot must replace, not merge with, the prior deadline"
    );
    assert!(!cadence.take_frame_request(now + Duration::from_millis(79)));
    assert!(cadence.take_frame_request(now + Duration::from_millis(80)));

    assert!(!cadence.apply_runtime_demand(
        now + Duration::from_millis(80),
        RuntimeFrameDemand::After(Duration::from_millis(20)),
    ));
    assert!(
        !cadence.apply_runtime_demand(now + Duration::from_millis(80), RuntimeFrameDemand::Idle,)
    );
    assert_eq!(cadence.control_flow(), ControlFlow::Wait);
}

#[test]
fn continuous_cadence_does_not_schedule_extra_wakes_from_runtime_demand() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Game, now);

    assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
    assert!(
        !cadence.apply_runtime_demand(now, RuntimeFrameDemand::After(Duration::from_millis(40)),)
    );
    assert_eq!(cadence.control_flow(), ControlFlow::Poll);
}

#[test]
fn foreground_game_and_explicit_continuous_cadence_never_suppress_frame_pumps() {
    for policy in [EventLoopPolicy::Game, EventLoopPolicy::Continuous] {
        let mut cadence = RuntimeFrameCadence::new(policy);
        let now = Instant::now();
        assert!(cadence.take_frame_request(now));
        assert!(cadence.take_frame_request(now));
        assert_eq!(cadence.control_flow(), ControlFlow::Poll);
        assert_eq!(cadence.report().idle_pumps_suppressed, 0);
    }
}

#[test]
fn game_cadence_throttles_unfocused_and_occluded_windows() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Game, now);
    assert!(cadence.take_frame_request(now));
    assert_eq!(cadence.control_flow(), ControlFlow::Poll);

    assert!(cadence.set_window_focused_at(false, now));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(now + UNFOCUSED_GAME_FRAME_INTERVAL)
    );
    assert!(!cadence.take_frame_request(now + UNFOCUSED_GAME_FRAME_INTERVAL / 2));
    let unchanged_deadline = cadence.control_flow();
    assert!(!cadence.set_window_focused_at(false, now + Duration::from_millis(1)));
    assert_eq!(cadence.control_flow(), unchanged_deadline);
    cadence.request_frame();
    assert!(cadence.take_frame_request(now + UNFOCUSED_GAME_FRAME_INTERVAL / 2));

    let occluded_at = now + UNFOCUSED_GAME_FRAME_INTERVAL;
    assert!(cadence.set_window_occluded_at(true, occluded_at));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(occluded_at + BACKGROUND_FRAME_INTERVAL)
    );
    assert!(cadence.set_window_focused_at(true, occluded_at));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(occluded_at + BACKGROUND_FRAME_INTERVAL),
        "occlusion remains authoritative even when focus returns"
    );
    assert!(cadence.set_window_occluded_at(false, occluded_at));
    assert_eq!(cadence.control_flow(), ControlFlow::Poll);

    assert_eq!(cadence.report().focus_transitions, 2);
    assert_eq!(cadence.report().occlusion_transitions, 2);
    assert_eq!(cadence.report().low_power_pumps, 1);
    assert_eq!(cadence.report().low_power_pumps_suppressed, 1);
}

#[test]
fn unfocused_game_cadence_caps_default_wake_rate_at_ten_hz() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Game, now);
    assert!(cadence.take_frame_request(now));
    assert!(cadence.set_window_focused_at(false, now));

    assert_eq!(
        UNFOCUSED_GAME_FRAME_INTERVAL,
        Duration::from_millis(100),
        "the default unfocused policy must cap timer-driven pumps at 10 Hz"
    );
    assert!(!cadence.take_frame_request(now + Duration::from_millis(99)));
    assert!(cadence.take_frame_request(now + Duration::from_millis(100)));
}

#[test]
fn mobile_cadence_has_explicit_foreground_and_background_limits() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Mobile, now);

    assert_eq!(cadence.control_flow(), ControlFlow::Poll);
    assert!(cadence.take_frame_request(now));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(now + MOBILE_FOREGROUND_FRAME_INTERVAL)
    );

    assert!(cadence.set_window_focused_at(false, now));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(now + BACKGROUND_FRAME_INTERVAL)
    );
}

#[test]
fn explicit_continuous_profile_ignores_visibility_throttling() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Continuous, now);

    assert!(cadence.set_window_focused_at(false, now));
    assert!(cadence.set_window_occluded_at(true, now));

    assert!(cadence.take_frame_request(now));
    assert_eq!(cadence.control_flow(), ControlFlow::Poll);
}

#[test]
fn low_power_cadence_consumes_runtime_immediate_and_after_demand() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Game, now);
    assert!(cadence.take_frame_request(now));
    assert!(cadence.set_window_focused_at(false, now));
    cadence.request_frame();
    assert!(cadence.take_frame_request(now));

    assert!(cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
    assert!(cadence.take_frame_request(now));

    let runtime_delay = Duration::from_millis(5);
    assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::After(runtime_delay),));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(now + runtime_delay)
    );
    assert!(cadence.take_frame_request(now + runtime_delay));
    assert!(!cadence.apply_runtime_demand(now + runtime_delay, RuntimeFrameDemand::Idle,));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(now + runtime_delay + UNFOCUSED_GAME_FRAME_INTERVAL)
    );
}

#[test]
fn headless_cadence_uses_fixed_wait_deadlines() {
    let now = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Headless, now);

    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(now + HEADLESS_FRAME_INTERVAL)
    );
    assert!(cadence.take_frame_request(now));
}

#[test]
fn headless_early_wake_does_not_pump_or_move_fixed_deadline() {
    let start = Instant::now();
    let deadline = start + HEADLESS_FRAME_INTERVAL;
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Headless, start);

    assert!(cadence.take_frame_request(start));
    cadence.request_frame();
    assert!(!cadence.take_frame_request(start + HEADLESS_FRAME_INTERVAL / 2));
    assert_eq!(cadence.control_flow(), ControlFlow::WaitUntil(deadline));

    assert!(cadence.take_frame_request(deadline));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(deadline + HEADLESS_FRAME_INTERVAL)
    );
}
