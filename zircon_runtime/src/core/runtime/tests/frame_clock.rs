use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::super::clock_source::{ClockSource, ManualClockSource};
use super::{FrameClock, FrameClockFirstTickPolicy, FrameClockRebaseCause};

struct ScriptedClockSource(Mutex<VecDeque<Instant>>);

impl ScriptedClockSource {
    fn new(samples: impl IntoIterator<Item = Instant>) -> Self {
        Self(Mutex::new(samples.into_iter().collect()))
    }
}

impl ClockSource for ScriptedClockSource {
    fn monotonic_now(&self) -> Instant {
        self.0
            .lock()
            .expect("scripted clock sample lock")
            .pop_front()
            .expect("scripted clock has a sample")
    }
}

#[test]
fn rebase_issues_a_monotonic_baseline_receipt() {
    let mut clock = FrameClock::default();

    let first = clock.rebase();
    let second = clock.rebase();

    assert_eq!(first.generation(), 1);
    assert_eq!(second.generation(), 2);
    assert_eq!(
        second.first_tick_policy(),
        FrameClockFirstTickPolicy::MeasureFromRebase
    );
    assert_eq!(second.cause(), FrameClockRebaseCause::Manual);
    assert_eq!(clock.tick().rebase(), Some(second));
    assert_eq!(clock.tick().rebase(), None);
}

#[test]
fn injected_clock_source_drives_tick_and_rebase_without_sleeping() {
    let source = Arc::new(ManualClockSource::with_origin(Instant::now()));
    let mut clock = FrameClock::with_clock_source(source.clone());

    source
        .try_advance_by(Duration::from_millis(16))
        .expect("manual source should advance");
    assert_eq!(clock.tick().delta(), Duration::from_millis(16));

    let receipt = clock.rebase();
    source
        .try_advance_by(Duration::from_millis(8))
        .expect("manual source should advance after rebase");
    let rebased = clock.tick();

    assert_eq!(rebased.delta(), Duration::from_millis(8));
    assert_eq!(rebased.rebase(), Some(receipt));
}

#[test]
fn backward_clock_sample_does_not_lower_the_frame_delta_baseline() {
    let origin = Instant::now();
    let source = Arc::new(ScriptedClockSource::new([
        origin,
        origin + Duration::from_millis(10),
        origin + Duration::from_millis(5),
        origin + Duration::from_millis(20),
    ]));
    let mut clock = FrameClock::with_clock_source(source);

    assert_eq!(clock.tick().delta(), Duration::from_millis(10));
    assert_eq!(clock.tick().delta(), Duration::ZERO);
    assert_eq!(clock.tick().delta(), Duration::from_millis(10));
}

#[test]
fn backward_rebase_sample_does_not_lower_the_frame_delta_baseline() {
    let origin = Instant::now();
    let source = Arc::new(ScriptedClockSource::new([
        origin,
        origin + Duration::from_millis(10),
        origin + Duration::from_millis(5),
        origin + Duration::from_millis(20),
    ]));
    let mut clock = FrameClock::with_clock_source(source);
    assert_eq!(clock.tick().delta(), Duration::from_millis(10));

    let receipt = clock.rebase();
    let tick = clock.tick();

    assert_eq!(tick.delta(), Duration::from_millis(10));
    assert_eq!(tick.rebase(), Some(receipt));
}
