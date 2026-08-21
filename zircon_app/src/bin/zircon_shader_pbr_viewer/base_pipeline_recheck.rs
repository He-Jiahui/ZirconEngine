use std::time::{Duration, Instant};

// Keeps the first pending recheck responsive while avoiding continuous full-frame presents.
pub(super) const BASE_PIPELINE_RECHECK_INTERVAL: Duration = Duration::from_millis(16);
pub(super) const BASE_PIPELINE_RECHECK_MAX_INTERVAL: Duration = Duration::from_millis(250);
pub(super) const ONE_SHOT_BASE_PIPELINE_WAIT_TIMEOUT: Duration = Duration::from_secs(45);

pub(super) fn base_pipeline_recheck_deadline_with_cap(
    now: Instant,
    retry_attempt: u32,
    deadline_cap: Option<Instant>,
) -> Instant {
    let deadline = now + base_pipeline_recheck_interval(retry_attempt);
    deadline_cap.map_or(deadline, |deadline_cap| deadline.min(deadline_cap))
}

pub(super) fn base_pipeline_recheck_is_due(deadline: Instant, now: Instant) -> bool {
    now >= deadline
}

pub(super) fn one_shot_base_pipeline_wait_deadline(started_at: Instant) -> Instant {
    started_at + ONE_SHOT_BASE_PIPELINE_WAIT_TIMEOUT
}

pub(super) fn one_shot_base_pipeline_wait_is_expired(started_at: Instant, now: Instant) -> bool {
    now >= one_shot_base_pipeline_wait_deadline(started_at)
}

fn base_pipeline_recheck_interval(retry_attempt: u32) -> Duration {
    let multiplier = 1_u32 << retry_attempt.min(4);
    BASE_PIPELINE_RECHECK_INTERVAL
        .saturating_mul(multiplier)
        .min(BASE_PIPELINE_RECHECK_MAX_INTERVAL)
}

#[cfg(test)]
mod tests {
    use super::{
        base_pipeline_recheck_deadline_with_cap, base_pipeline_recheck_interval,
        base_pipeline_recheck_is_due, one_shot_base_pipeline_wait_deadline,
        one_shot_base_pipeline_wait_is_expired, BASE_PIPELINE_RECHECK_INTERVAL,
        BASE_PIPELINE_RECHECK_MAX_INTERVAL, ONE_SHOT_BASE_PIPELINE_WAIT_TIMEOUT,
    };
    use std::time::Duration;

    #[test]
    fn pending_base_pipeline_rechecks_on_a_bounded_deadline() {
        let started = std::time::Instant::now();
        let deadline = base_pipeline_recheck_deadline_with_cap(started, 0, None);

        assert_eq!(deadline, started + BASE_PIPELINE_RECHECK_INTERVAL);
        assert!(!base_pipeline_recheck_is_due(deadline, started));
        assert!(!base_pipeline_recheck_is_due(
            deadline,
            deadline - Duration::from_nanos(1)
        ));
        assert!(base_pipeline_recheck_is_due(deadline, deadline));
    }

    #[test]
    fn one_shot_base_pipeline_recheck_never_sleeps_past_its_terminal_deadline() {
        let started = std::time::Instant::now();
        let terminal_deadline = one_shot_base_pipeline_wait_deadline(started);
        let near_deadline = terminal_deadline - Duration::from_millis(1);

        assert_eq!(
            base_pipeline_recheck_deadline_with_cap(near_deadline, 4, Some(terminal_deadline)),
            terminal_deadline,
            "the final backoff must wake at the one-shot timeout rather than up to 250 ms later"
        );
        assert_eq!(
            base_pipeline_recheck_deadline_with_cap(near_deadline, 4, None),
            near_deadline + BASE_PIPELINE_RECHECK_MAX_INTERVAL,
            "interactive rechecks retain their capped backoff without a one-shot deadline"
        );
    }

    #[test]
    fn pending_base_pipeline_rechecks_back_off_before_the_bounded_ceiling() {
        assert_eq!(
            base_pipeline_recheck_interval(0),
            BASE_PIPELINE_RECHECK_INTERVAL
        );
        assert_eq!(base_pipeline_recheck_interval(1), Duration::from_millis(32));
        assert_eq!(base_pipeline_recheck_interval(2), Duration::from_millis(64));
        assert_eq!(
            base_pipeline_recheck_interval(3),
            Duration::from_millis(128)
        );
        assert_eq!(
            base_pipeline_recheck_interval(4),
            Duration::from_millis(250)
        );
        assert_eq!(
            base_pipeline_recheck_interval(u32::MAX),
            Duration::from_millis(250)
        );
    }

    #[test]
    fn one_shot_base_pipeline_wait_has_a_shared_bounded_deadline() {
        let started = std::time::Instant::now();
        let deadline = one_shot_base_pipeline_wait_deadline(started);

        assert_eq!(ONE_SHOT_BASE_PIPELINE_WAIT_TIMEOUT, Duration::from_secs(45));
        assert_eq!(deadline, started + ONE_SHOT_BASE_PIPELINE_WAIT_TIMEOUT);
        assert!(!one_shot_base_pipeline_wait_is_expired(
            started,
            deadline - Duration::from_nanos(1)
        ));
        assert!(one_shot_base_pipeline_wait_is_expired(started, deadline));
    }
}
