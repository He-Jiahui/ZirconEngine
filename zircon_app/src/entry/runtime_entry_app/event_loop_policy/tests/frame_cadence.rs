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

#[test]
fn low_power_late_wakes_preserve_periodic_phase_across_visibility_modes() {
    let start = Instant::now();
    for (policy, focused, occluded, interval) in [
        (EventLoopPolicy::Game, false, false, UNFOCUSED_GAME_FRAME_INTERVAL),
        (EventLoopPolicy::Game, true, true, BACKGROUND_FRAME_INTERVAL),
        (EventLoopPolicy::Mobile, true, false, MOBILE_FOREGROUND_FRAME_INTERVAL),
        (EventLoopPolicy::Mobile, false, false, BACKGROUND_FRAME_INTERVAL),
    ] {
        let mut cadence =
            RuntimeFrameCadence::new_at_with_window_state(policy, focused, occluded, start);
        assert!(cadence.take_frame_request(start));
        let first_deadline = start + interval;
        let late = first_deadline + interval / 20;
        assert!(cadence.take_frame_request(late));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(first_deadline + interval)
        );
        assert!(!cadence.take_frame_request(late));
        assert!(cadence.take_frame_request(first_deadline + interval));
        assert_eq!(cadence.report().frame_pumps, 3);
    }
}

#[test]
fn fixed_interval_late_wakes_skip_missed_slots_without_catch_up() {
    let start = Instant::now();
    let interval = HEADLESS_FRAME_INTERVAL;
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Headless, start);
    assert!(cadence.take_frame_request(start));

    let late = start + interval * 3 + interval / 2;
    assert!(cadence.take_frame_request(late));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(start + interval * 4)
    );
    assert!(!cadence.take_frame_request(late));
    assert!(!cadence.take_frame_request(start + interval * 4 - Duration::from_nanos(1)));
    assert!(cadence.take_frame_request(start + interval * 4));

    // A large number of missed slots still produces only one pump, not a catch-up burst.
    let much_later = start + interval * 1_000_000 + interval / 2;
    assert!(cadence.take_frame_request(much_later));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(start + interval * 1_000_001)
    );
    assert!(!cadence.take_frame_request(much_later));
    assert_eq!(cadence.report().frame_pumps, 4);
}

#[test]
fn low_power_early_requested_and_runtime_pumps_keep_the_existing_rebase() {
    let start = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Game, start);
    assert!(cadence.take_frame_request(start));
    assert!(cadence.set_window_focused_at(false, start));

    let requested_at = start + Duration::from_millis(30);
    assert!(cadence.request_frame());
    assert!(cadence.take_frame_request(requested_at));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(requested_at + UNFOCUSED_GAME_FRAME_INTERVAL)
    );
    let delay = Duration::from_millis(5);
    assert!(!cadence.apply_runtime_demand(requested_at, RuntimeFrameDemand::After(delay)));
    let runtime_at = requested_at + delay;
    assert!(cadence.take_frame_request(runtime_at));
    assert!(!cadence.apply_runtime_demand(runtime_at, RuntimeFrameDemand::Idle));
    let deadline = runtime_at + UNFOCUSED_GAME_FRAME_INTERVAL;
    assert_eq!(cadence.control_flow(), ControlFlow::WaitUntil(deadline));

    // Once that rebased periodic deadline is due, lateness must not rebase it again.
    assert!(cadence.take_frame_request(deadline + delay));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(deadline + UNFOCUSED_GAME_FRAME_INTERVAL)
    );
}

#[test]
fn cadence_mode_changes_rebase_but_unchanged_visibility_preserves_phase() {
    let start = Instant::now();
    let mut game = RuntimeFrameCadence::new_at(EventLoopPolicy::Game, start);
    assert!(game.take_frame_request(start));
    let blurred_at = start + Duration::from_millis(20);
    assert!(game.set_window_focused_at(false, blurred_at));
    assert_eq!(
        game.control_flow(),
        ControlFlow::WaitUntil(blurred_at + UNFOCUSED_GAME_FRAME_INTERVAL)
    );
    assert!(!game.set_window_focused_at(false, blurred_at + Duration::from_millis(1)));

    let occluded_at = start + Duration::from_millis(50);
    assert!(game.set_window_occluded_at(true, occluded_at));
    let background_deadline = occluded_at + BACKGROUND_FRAME_INTERVAL;
    assert!(game.set_window_focused_at(true, occluded_at + Duration::from_millis(10)));
    assert_eq!(game.control_flow(), ControlFlow::WaitUntil(background_deadline));
    assert!(game.set_window_occluded_at(false, occluded_at + Duration::from_millis(20)));
    assert_eq!(game.control_flow(), ControlFlow::Poll);

    let mut mobile = RuntimeFrameCadence::new_at(EventLoopPolicy::Mobile, start);
    assert!(mobile.take_frame_request(start));
    assert!(mobile.set_window_focused_at(false, blurred_at));
    assert!(mobile.set_window_occluded_at(true, occluded_at));
    assert_eq!(
        mobile.control_flow(),
        ControlFlow::WaitUntil(blurred_at + BACKGROUND_FRAME_INTERVAL)
    );
    assert!(mobile.set_window_focused_at(true, occluded_at));
    assert_eq!(
        mobile.control_flow(),
        ControlFlow::WaitUntil(blurred_at + BACKGROUND_FRAME_INTERVAL)
    );
    let visible_at = occluded_at + Duration::from_millis(20);
    assert!(mobile.set_window_occluded_at(false, visible_at));
    assert_eq!(
        mobile.control_flow(),
        ControlFlow::WaitUntil(visible_at + MOBILE_FOREGROUND_FRAME_INTERVAL)
    );
}

// Find a valid Instant at the platform's representability boundary using only checked_add.
// Both bounded searches are test-only; production never walks elapsed periodic slots.
fn latest_representable_instant(start: Instant) -> Instant {
    let mut low = 0_u64;
    let mut high = u64::MAX;
    while low < high {
        let middle = low + (high - low) / 2 + 1;
        if start.checked_add(Duration::from_secs(middle)).is_some() {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    assert!(low < u64::MAX, "the native Instant range must have a finite checked boundary");
    let last_second = start.checked_add(Duration::from_secs(low)).unwrap();
    let mut low_nanos = 0_u32;
    let mut high_nanos = 999_999_999_u32;
    while low_nanos < high_nanos {
        let middle = low_nanos + (high_nanos - low_nanos) / 2 + 1;
        if last_second
            .checked_add(Duration::from_nanos(u64::from(middle)))
            .is_some()
        {
            low_nanos = middle;
        } else {
            high_nanos = middle - 1;
        }
    }
    let edge = last_second
        .checked_add(Duration::from_nanos(u64::from(low_nanos)))
        .unwrap();
    assert!(edge.checked_add(Duration::from_nanos(1)).is_none());
    edge
}

#[test]
fn unrepresentable_periodic_deadlines_wait_without_busy_pumping() {
    let start = Instant::now();
    let edge = latest_representable_instant(start);
    for (policy, focused, occluded, interval) in [
        (EventLoopPolicy::Headless, true, false, HEADLESS_FRAME_INTERVAL),
        (EventLoopPolicy::Game, false, false, UNFOCUSED_GAME_FRAME_INTERVAL),
        (EventLoopPolicy::Game, true, true, BACKGROUND_FRAME_INTERVAL),
        (EventLoopPolicy::Mobile, true, false, MOBILE_FOREGROUND_FRAME_INTERVAL),
    ] {
        let mut cadence =
            RuntimeFrameCadence::new_at_with_window_state(policy, focused, occluded, edge);
        assert!(cadence.take_frame_request(edge));
        assert_eq!(cadence.control_flow(), ControlFlow::Wait);
        assert!(!cadence.take_frame_request(edge));

        // Also exercise overflow while advancing an already installed, due deadline.
        cadence.mode = if policy == EventLoopPolicy::Headless {
            RuntimeFrameCadenceMode::FixedInterval {
                interval,
                next_deadline: Some(edge),
            }
        } else {
            RuntimeFrameCadenceMode::LowPower {
                interval,
                next_deadline: Some(edge),
            }
        };
        assert!(cadence.take_frame_request(edge));
        assert_eq!(cadence.control_flow(), ControlFlow::Wait);
        assert!(!cadence.take_frame_request(edge));
    }
}

#[test]
fn low_power_none_deadline_accepts_work_and_recovers_on_mode_rebase() {
    let start = Instant::now();
    let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Mobile, start);
    assert!(cadence.take_frame_request(start));
    // A private, valid state isolates the recovery contract without inventing an Instant.
    cadence.mode = RuntimeFrameCadenceMode::LowPower {
        interval: MOBILE_FOREGROUND_FRAME_INTERVAL,
        next_deadline: None,
    };
    assert_eq!(cadence.control_flow(), ControlFlow::Wait);
    assert!(!cadence.take_frame_request(start));
    assert!(cadence.request_frame());
    assert_eq!(cadence.control_flow(), ControlFlow::Poll);
    assert!(cadence.take_frame_request(start));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(start + MOBILE_FOREGROUND_FRAME_INTERVAL)
    );

    cadence.mode = RuntimeFrameCadenceMode::LowPower {
        interval: MOBILE_FOREGROUND_FRAME_INTERVAL,
        next_deadline: None,
    };
    let delay = Duration::from_millis(5);
    assert!(!cadence.apply_runtime_demand(start, RuntimeFrameDemand::After(delay)));
    assert_eq!(cadence.control_flow(), ControlFlow::WaitUntil(start + delay));
    assert!(cadence.take_frame_request(start + delay));

    cadence.mode = RuntimeFrameCadenceMode::LowPower {
        interval: MOBILE_FOREGROUND_FRAME_INTERVAL,
        next_deadline: None,
    };
    let blurred_at = start + delay;
    assert!(cadence.set_window_focused_at(false, blurred_at));
    assert_eq!(
        cadence.control_flow(),
        ControlFlow::WaitUntil(blurred_at + BACKGROUND_FRAME_INTERVAL)
    );
}
