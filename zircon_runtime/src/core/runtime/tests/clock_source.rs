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
