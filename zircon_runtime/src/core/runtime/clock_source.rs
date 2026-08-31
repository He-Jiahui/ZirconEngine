//! Injection boundary for samples that advance the runtime frame clock.

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use thiserror::Error;

/// Supplies the monotonic samples consumed by [`super::FrameClock`].
///
/// This boundary intentionally covers frame-delta measurement only. OS
/// deadlines, file watchers, telemetry, and profiling retain their own time
/// sources because they must continue to progress while a runtime clock is
/// paused, replayed, or otherwise controlled.
pub trait ClockSource: Send + Sync + 'static {
    /// Returns the next monotonic sample for a runtime outer-frame tick.
    fn monotonic_now(&self) -> Instant;
}

/// A caller-driven monotonic source for tests, replay, and external authorities.
///
/// The source stores an opaque `Instant` anchor together with a non-decreasing
/// elapsed duration. Replay seek must construct a new source and rebase the
/// frame clock instead of producing a negative outer-frame delta.
#[derive(Clone, Debug)]
pub struct ManualClockSource {
    state: Arc<Mutex<ManualClockState>>,
}

#[derive(Clone, Copy, Debug)]
struct ManualClockState {
    now: Instant,
    elapsed: Duration,
}

impl ManualClockSource {
    /// Creates a source at a caller-owned monotonic origin.
    pub fn with_origin(origin: Instant) -> Self {
        Self {
            state: Arc::new(Mutex::new(ManualClockState {
                now: origin,
                elapsed: Duration::ZERO,
            })),
        }
    }

    /// Returns elapsed time accepted by this source since its origin.
    pub fn elapsed(&self) -> Duration {
        self.lock_state().elapsed
    }

    /// Advances the source by a non-negative delta.
    pub fn try_advance_by(&self, delta: Duration) -> Result<(), ManualClockSourceError> {
        let mut state = self.lock_state();
        let requested =
            state
                .elapsed
                .checked_add(delta)
                .ok_or(ManualClockSourceError::OutOfRange {
                    current: state.elapsed,
                    delta,
                })?;
        advance_state_to(&mut state, requested)
    }

    /// Advances the source to an absolute elapsed position from its origin.
    pub fn try_advance_to(&self, requested: Duration) -> Result<(), ManualClockSourceError> {
        let mut state = self.lock_state();
        advance_state_to(&mut state, requested)
    }

    fn lock_state(&self) -> MutexGuard<'_, ManualClockState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl ClockSource for ManualClockSource {
    fn monotonic_now(&self) -> Instant {
        self.lock_state().now
    }
}

fn advance_state_to(
    state: &mut ManualClockState,
    requested: Duration,
) -> Result<(), ManualClockSourceError> {
    if requested < state.elapsed {
        return Err(ManualClockSourceError::NonMonotonicAdvance {
            current: state.elapsed,
            requested,
        });
    }
    let delta = requested.saturating_sub(state.elapsed);
    let Some(now) = state.now.checked_add(delta) else {
        return Err(ManualClockSourceError::OutOfRange {
            current: state.elapsed,
            delta,
        });
    };
    state.now = now;
    state.elapsed = requested;
    Ok(())
}

/// Rejection from a caller-driven monotonic clock update.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ManualClockSourceError {
    #[error("manual clock cannot move backward from {current:?} to {requested:?}")]
    NonMonotonicAdvance {
        current: Duration,
        requested: Duration,
    },
    #[error("manual clock cannot represent a {delta:?} advance from {current:?}")]
    OutOfRange { current: Duration, delta: Duration },
}

#[derive(Clone)]
pub(crate) enum FrameClockSource {
    SystemMonotonic,
    Injected(Arc<dyn ClockSource>),
}

impl FrameClockSource {
    pub(crate) const fn system_monotonic() -> Self {
        Self::SystemMonotonic
    }

    pub(crate) fn injected(source: Arc<dyn ClockSource>) -> Self {
        Self::Injected(source)
    }

    pub(crate) fn monotonic_now(&self) -> Instant {
        match self {
            Self::SystemMonotonic => Instant::now(),
            Self::Injected(source) => source.monotonic_now(),
        }
    }
}

impl fmt::Debug for FrameClockSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemMonotonic => formatter.write_str("FrameClockSource::SystemMonotonic"),
            Self::Injected(_) => formatter.write_str("FrameClockSource::Injected(..)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::{ManualClockSource, ManualClockSourceError};
    use crate::core::CoreRuntime;

    #[test]
    fn manual_clock_source_rejects_a_replay_sample_before_its_current_position() {
        let source = ManualClockSource::with_origin(Instant::now());

        source
            .try_advance_to(Duration::from_millis(20))
            .expect("forward replay sample should be accepted");

        assert_eq!(
            source.try_advance_to(Duration::from_millis(10)),
            Err(ManualClockSourceError::NonMonotonicAdvance {
                current: Duration::from_millis(20),
                requested: Duration::from_millis(10),
            })
        );
        assert_eq!(source.elapsed(), Duration::from_millis(20));
    }

    #[test]
    fn manual_clock_source_drives_core_runtime_without_wall_clock_waiting() {
        let source = Arc::new(ManualClockSource::with_origin(Instant::now()));
        let runtime = CoreRuntime::with_clock_source(source.clone());

        source
            .try_advance_by(Duration::from_millis(16))
            .expect("manual source should advance");
        let first = runtime.tick_time(8);
        source
            .try_advance_by(Duration::from_millis(8))
            .expect("manual source should advance again");
        let second = runtime.tick_time(8);

        assert_eq!(first.raw_real_delta(), Duration::from_millis(16));
        assert_eq!(second.raw_real_delta(), Duration::from_millis(8));
        assert_eq!(second.outer_frame_index(), 2);
    }
}
