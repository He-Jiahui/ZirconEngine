use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ReconnectPolicy {
    pub(crate) base_delay: Duration,
    pub(crate) max_delay: Duration,
    pub(crate) jitter: Duration,
    pub(crate) max_attempts: u32,
}

impl ReconnectPolicy {
    pub(crate) fn new(
        base_delay: Duration,
        max_delay: Duration,
        jitter: Duration,
        max_attempts: u32,
    ) -> Self {
        Self {
            base_delay,
            max_delay,
            jitter,
            max_attempts,
        }
    }

    pub(crate) fn delay_for_attempt(self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }

        let multiplier = 1_u128.checked_shl(attempt).unwrap_or(u128::MAX);
        let base_ms = self.base_delay.as_millis().saturating_mul(multiplier);
        let capped_ms = base_ms.min(self.max_delay.as_millis());
        let jitter_ms = deterministic_jitter_ms(attempt, self.jitter);
        let delayed_ms = capped_ms
            .saturating_add(jitter_ms)
            .min(self.max_delay.as_millis());
        Some(duration_from_millis(delayed_ms))
    }

    pub(crate) fn delays(self) -> impl Iterator<Item = Duration> {
        (0..self.max_attempts).filter_map(move |attempt| self.delay_for_attempt(attempt))
    }
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self::new(
            Duration::from_millis(100),
            Duration::from_secs(5),
            Duration::from_millis(0),
            5,
        )
    }
}

fn deterministic_jitter_ms(attempt: u32, jitter: Duration) -> u128 {
    let jitter_ms = jitter.as_millis();
    if jitter_ms == 0 {
        return 0;
    }

    let sample = (attempt as u128)
        .wrapping_mul(1_103_515_245)
        .wrapping_add(12_345);
    sample % (jitter_ms + 1)
}

fn duration_from_millis(millis: u128) -> Duration {
    Duration::from_millis(millis.min(u64::MAX as u128) as u64)
}
