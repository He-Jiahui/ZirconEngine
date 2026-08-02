use std::time::{Duration, Instant};

const GAMEPAD_DRAIN_MAX_EVENTS_PER_FRAME: usize = 256;
const GAMEPAD_DRAIN_MAX_TIME_PER_FRAME: Duration = Duration::from_millis(2);

pub(super) struct GamepadDrainBudget {
    processed_events: usize,
    deadline: Instant,
}

impl GamepadDrainBudget {
    pub(super) fn begin(now: Instant) -> Self {
        Self {
            processed_events: 0,
            deadline: now
                .checked_add(GAMEPAD_DRAIN_MAX_TIME_PER_FRAME)
                .unwrap_or(now),
        }
    }

    pub(super) fn record_event(&mut self) {
        self.processed_events = self.processed_events.saturating_add(1);
    }

    pub(super) fn needs_continuation(&self, now: Instant) -> bool {
        self.processed_events >= GAMEPAD_DRAIN_MAX_EVENTS_PER_FRAME
            || (self.processed_events > 0 && now >= self.deadline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_budget_stops_at_the_named_per_frame_limit() {
        assert_eq!(GAMEPAD_DRAIN_MAX_EVENTS_PER_FRAME, 256);
        let now = Instant::now();
        let mut budget = GamepadDrainBudget::begin(now);

        for _ in 0..GAMEPAD_DRAIN_MAX_EVENTS_PER_FRAME {
            assert!(!budget.needs_continuation(now));
            budget.record_event();
        }

        assert!(budget.needs_continuation(now));
    }

    #[test]
    fn time_budget_stops_after_at_least_one_event() {
        assert_eq!(GAMEPAD_DRAIN_MAX_TIME_PER_FRAME, Duration::from_millis(2));
        let now = Instant::now();
        let mut budget = GamepadDrainBudget::begin(now);
        let expired = now + GAMEPAD_DRAIN_MAX_TIME_PER_FRAME;

        assert!(!budget.needs_continuation(expired));
        budget.record_event();
        assert!(budget.needs_continuation(expired));
    }

    #[test]
    fn queue_exhaustion_before_either_limit_needs_no_continuation() {
        let now = Instant::now();
        let mut budget = GamepadDrainBudget::begin(now);
        budget.record_event();

        assert!(!budget.needs_continuation(now));
    }

    #[test]
    fn polling_keeps_budget_and_continuation_outside_the_gilrs_borrow() {
        let source = include_str!("../polling.rs");
        let source = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();

        let budget_check = source
            .find("ifdrain_budget.needs_continuation(std::time::Instant::now())")
            .expect("polling checks the named drain budget");
        let next_event = source
            .find("letSome(event)=gamepads.next_event()else")
            .expect("polling consumes the next gilrs event only after the budget check");
        let record_event = source
            .find("drain_budget.record_event();")
            .expect("polling records each consumed event");
        let update_event = source
            .find("gamepads.update(&event);")
            .expect("polling preserves gilrs event update order");
        assert!(
            budget_check < next_event && next_event < record_event && record_event < update_event
        );

        let dispatch_error = source
            .find("ifletErr(error)=result{dispatch_error=Some(error);break;}")
            .expect("polling retains the runtime event dispatch error");
        let gilrs_increment = source
            .find("ifdispatch_error.is_none(){gamepads.inc();}")
            .expect("polling advances gilrs only after successful event handling");
        assert!(source.contains(
            "ifdispatch_error.is_none(){gamepads.inc();}}forgamepad_idindisconnected_gamepads{"
        ));

        let disconnected_cleanup = source
            .find("forgamepad_idindisconnected_gamepads{")
            .expect("polling clears disconnected gamepad rumble state after the gilrs borrow");
        let finished_cleanup = source
            .find("super::rumble::clear_finished_rumble_effects(")
            .expect("polling clears completed rumble effects");
        let report_failure = source
            .find("ifletSome(error)=dispatch_error{self.report_fatal_failure(")
            .expect("polling records a fatal diagnostic before exiting on dispatch errors");
        let exit_or_continue = source
            .find("event_loop.exit();}elseifdrain_budget_exhausted{self.request_runtime_frame();}")
            .expect("polling exits on errors or requests one continuation after budget exhaustion");
        assert!(update_event < dispatch_error && dispatch_error < gilrs_increment);
        assert!(gilrs_increment < disconnected_cleanup);
        assert!(disconnected_cleanup < finished_cleanup && finished_cleanup < report_failure);
        assert!(report_failure < exit_or_continue);
    }
}
