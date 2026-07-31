use woc_runtime::{
    PresentationBlendMode, PresentationCadence, PresentationSnapshot, PresentationTimeline,
    PresentationTimelineError, PresentationTimelinePush,
};

#[derive(Clone, Debug, PartialEq)]
struct Projection {
    player_x: f32,
}

fn snapshot(
    generation: u64,
    tick: u64,
    digest: u32,
    received_at_ns: u64,
    player_x: f32,
) -> PresentationSnapshot<Projection> {
    PresentationSnapshot::new(
        generation,
        tick,
        digest,
        digest.rotate_left(7),
        digest.rotate_left(13),
        received_at_ns,
        Projection { player_x },
    )
}

#[test]
fn default_cadence_is_twenty_authoritative_and_sixty_presentation_hz() {
    let cadence = PresentationCadence::woc_default();

    assert_eq!(cadence.simulation_hz(), 20);
    assert_eq!(cadence.presentation_hz(), 60);
    assert_eq!(cadence.simulation_step_ns(), 50_000_000);
    assert_eq!(cadence.presentation_subframes_per_tick(), 3);
}

#[test]
fn sixty_hz_samples_interpolate_only_between_committed_bulk_snapshots() {
    let mut timeline = PresentationTimeline::new(PresentationCadence::woc_default());
    assert_eq!(
        timeline
            .push(snapshot(4, 10, 0x10, 0, 0.0))
            .expect("first snapshot"),
        PresentationTimelinePush::Reset
    );
    let initial = timeline.sample(16_666_667).expect("initial hold");
    assert_eq!(initial.mode, PresentationBlendMode::HoldCurrent);
    assert_eq!(initial.from, &Projection { player_x: 0.0 });
    assert_eq!(initial.to, initial.from);

    assert_eq!(
        timeline
            .push(snapshot(4, 11, 0x11, 50_000_000, 3.0))
            .expect("next snapshot"),
        PresentationTimelinePush::Advanced
    );

    let at_tick = timeline.sample(50_000_000).expect("tick boundary");
    assert_eq!(at_tick.mode, PresentationBlendMode::Interpolate);
    assert_eq!(at_tick.from, &Projection { player_x: 0.0 });
    assert_eq!(at_tick.to, &Projection { player_x: 3.0 });
    assert_eq!(at_tick.alpha, 0.0);

    let one_third = timeline.sample(66_666_667).expect("one third");
    assert!((one_third.alpha - (1.0 / 3.0)).abs() < 0.000_001);
    let two_thirds = timeline.sample(83_333_333).expect("two thirds");
    assert!((two_thirds.alpha - (2.0 / 3.0)).abs() < 0.000_001);

    let complete = timeline.sample(100_000_000).expect("completed blend");
    assert_eq!(complete.mode, PresentationBlendMode::HoldCurrent);
    assert_eq!(complete.from, &Projection { player_x: 3.0 });
    assert_eq!(complete.to, complete.from);
}

#[test]
fn duplicates_are_idempotent_but_conflicts_and_regressions_are_rejected() {
    let mut timeline = PresentationTimeline::new(PresentationCadence::woc_default());
    timeline
        .push(snapshot(7, 20, 0x20, 100, 1.0))
        .expect("initial snapshot");
    assert_eq!(
        timeline
            .push(snapshot(7, 20, 0x20, 200, 999.0))
            .expect("same committed identity is idempotent"),
        PresentationTimelinePush::Duplicate
    );
    assert_eq!(
        &timeline.current().expect("current").projection,
        &Projection { player_x: 1.0 }
    );

    let mut projection_conflict = snapshot(7, 20, 0x20, 200, 2.0);
    projection_conflict.presentation_digest ^= 1;
    assert_eq!(
        timeline
            .push(projection_conflict)
            .expect_err("same authority with another projection must fail"),
        PresentationTimelineError::ConflictingSnapshot {
            generation: 7,
            tick: 20,
        }
    );

    assert_eq!(
        timeline
            .push(snapshot(7, 20, 0x21, 200, 2.0))
            .expect_err("same tick with another digest must fail"),
        PresentationTimelineError::ConflictingSnapshot {
            generation: 7,
            tick: 20,
        }
    );
    assert_eq!(
        timeline
            .push(snapshot(7, 19, 0x19, 200, -1.0))
            .expect_err("tick regression must fail"),
        PresentationTimelineError::TickRegressed {
            generation: 7,
            actual: 19,
            current: 20,
        }
    );
    assert_eq!(
        timeline
            .push(snapshot(6, 21, 0x21, 200, 2.0))
            .expect_err("generation regression must fail"),
        PresentationTimelineError::GenerationRegressed {
            actual: 6,
            current: 7,
        }
    );
    assert_eq!(
        timeline
            .push(snapshot(7, 21, 0x21, 99, 2.0))
            .expect_err("presentation receipt time must be monotonic"),
        PresentationTimelineError::ReceiptTimeRegressed {
            actual_ns: 99,
            current_ns: 100,
        }
    );
}

#[test]
fn a_new_vm_generation_resets_interpolation_history() {
    let mut timeline = PresentationTimeline::new(PresentationCadence::woc_default());
    timeline
        .push(snapshot(1, 40, 0x40, 1_000, 4.0))
        .expect("old generation");
    timeline
        .push(snapshot(1, 41, 0x41, 50_001_000, 5.0))
        .expect("old next tick");

    assert_eq!(
        timeline
            .push(snapshot(2, 41, 0x51, 50_002_000, 8.0))
            .expect("replacement generation"),
        PresentationTimelinePush::Reset
    );
    assert!(timeline.previous().is_none());
    let sample = timeline.sample(75_000_000).expect("replacement hold");
    assert_eq!(sample.mode, PresentationBlendMode::HoldCurrent);
    assert_eq!(sample.from, &Projection { player_x: 8.0 });
}
