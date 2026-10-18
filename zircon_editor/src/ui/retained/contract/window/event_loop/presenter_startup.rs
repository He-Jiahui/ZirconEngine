use std::time::{Duration, Instant};

const READINESS_POLL_INTERVAL: Duration = Duration::from_millis(50);
const READINESS_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PresenterStartupAction {
    Poll,
    Wait,
    TimedOut,
    Selected { runtime: bool },
}

/// Select once before creating a native surface. A standalone choice lasts for this window.
#[derive(Default)]
pub(super) struct PresenterStartup {
    started: Option<Instant>,
    next_poll: Option<Instant>,
    runtime: Option<bool>,
}

impl PresenterStartup {
    pub(super) fn action(&mut self, now: Instant) -> PresenterStartupAction {
        if let Some(runtime) = self.runtime {
            return PresenterStartupAction::Selected { runtime };
        }
        let started = *self.started.get_or_insert(now);
        let timeout = started + READINESS_TIMEOUT;
        if now >= timeout {
            self.select(false);
            return PresenterStartupAction::TimedOut;
        }
        if self.next_poll.is_some_and(|deadline| now < deadline) {
            return PresenterStartupAction::Wait;
        }
        self.next_poll = Some((now + READINESS_POLL_INTERVAL).min(timeout));
        PresenterStartupAction::Poll
    }

    pub(super) fn select(&mut self, runtime: bool) {
        self.runtime.get_or_insert(runtime);
        self.next_poll = None;
    }

    pub(super) fn wake_deadline(&self) -> Option<Instant> {
        self.next_poll
    }
}

#[cfg(test)]
#[path = "tests/startup.rs"]
mod tests;
