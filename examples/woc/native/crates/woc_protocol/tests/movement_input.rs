use woc_protocol::{
    EntityRef, MovementFrame, MovementFrameBatch, MovementFrameDisposition, MovementInputError,
    MovementInputFlags, MovementInputRelay, MOVEMENT_INPUT_STALE_AFTER_TICKS,
};

fn actor(id: u64, generation: u32) -> EntityRef {
    EntityRef { id, generation }
}

fn flags(forward: bool, back: bool) -> MovementInputFlags {
    MovementInputFlags {
        forward,
        back,
        ..MovementInputFlags::default()
    }
}

fn frame(
    actor: EntityRef,
    sequence: u32,
    flags: MovementInputFlags,
    facing: Option<f64>,
) -> MovementFrame {
    MovementFrame {
        actor,
        sequence,
        flags,
        facing,
    }
}

#[test]
fn movement_batch_canonicalizes_actor_order_and_rejects_invalid_frames() {
    let batch = MovementFrameBatch::new(vec![
        frame(actor(9, 1), 4, flags(true, false), None),
        frame(actor(3, 2), 7, flags(false, true), Some(0.25)),
    ])
    .expect("valid frames must form a batch");
    assert_eq!(
        batch
            .frames()
            .iter()
            .map(|frame| frame.actor)
            .collect::<Vec<_>>(),
        vec![actor(3, 2), actor(9, 1)]
    );

    assert!(matches!(
        MovementFrameBatch::new(vec![frame(actor(0, 1), 1, flags(false, false), None)]),
        Err(MovementInputError::InvalidActor { .. })
    ));
    assert!(matches!(
        MovementFrameBatch::new(vec![frame(actor(1, 1), 0, flags(false, false), None)]),
        Err(MovementInputError::InvalidSequence { .. })
    ));
    assert!(matches!(
        MovementFrameBatch::new(vec![frame(
            actor(1, 1),
            1,
            flags(false, false),
            Some(f64::NAN)
        )]),
        Err(MovementInputError::InvalidFacing { .. })
    ));
    assert!(matches!(
        MovementFrameBatch::new(vec![
            frame(actor(1, 1), 1, flags(false, false), None),
            frame(actor(1, 1), 2, flags(false, false), None),
        ]),
        Err(MovementInputError::DuplicateActor { .. })
    ));
}

#[test]
fn relay_applies_each_valid_frame_and_keeps_a_monotonic_acknowledgement() {
    let player = actor(41, 3);
    let mut relay = MovementInputRelay::default();
    let initial = MovementFrameBatch::new(vec![frame(player, 4, flags(true, false), Some(0.75))])
        .expect("initial frame is valid");
    assert_eq!(
        relay
            .apply_batch(10, &initial)
            .expect("initial batch applies"),
        vec![MovementFrameDisposition::Applied {
            actor: player,
            acknowledgement: 4,
        }]
    );

    let older_sequence =
        MovementFrameBatch::new(vec![frame(player, 3, flags(false, true), Some(0.25))])
            .expect("older-sequence frame shape is still valid");
    assert_eq!(
        relay
            .apply_batch(11, &older_sequence)
            .expect("older-sequence batch is applied"),
        vec![MovementFrameDisposition::Applied {
            actor: player,
            acknowledgement: 4,
        }]
    );
    let after_older_sequence = relay.input(player).expect("player input is retained");
    assert_eq!(after_older_sequence.flags, flags(false, true));
    assert_eq!(after_older_sequence.facing, Some(0.25));
    assert_eq!(after_older_sequence.accepted_tick, 11);
    assert_eq!(relay.acknowledgement(player), Some(4));

    let newer = MovementFrameBatch::new(vec![frame(player, 5, flags(false, true), None)])
        .expect("newer frame is valid");
    relay.apply_batch(12, &newer).expect("newer batch applies");
    let active = relay.input(player).expect("player input is retained");
    assert_eq!(active.flags, flags(false, true));
    assert_eq!(active.facing, Some(0.25));
    assert_eq!(relay.acknowledgement(player), Some(5));
}

#[test]
fn relay_clears_held_flags_only_after_the_source_stale_window() {
    let player = actor(7, 1);
    let mut relay = MovementInputRelay::default();
    let batch = MovementFrameBatch::new(vec![frame(player, 1, flags(true, false), Some(-0.5))])
        .expect("frame is valid");
    relay.apply_batch(20, &batch).expect("batch applies");

    assert!(relay
        .clear_stale(20 + MOVEMENT_INPUT_STALE_AFTER_TICKS)
        .expect("exact stale threshold is valid")
        .is_empty());
    assert_eq!(
        relay.input(player).expect("input remains").flags,
        flags(true, false)
    );

    assert_eq!(
        relay
            .clear_stale(21 + MOVEMENT_INPUT_STALE_AFTER_TICKS)
            .expect("post-threshold tick is valid"),
        vec![player]
    );
    let cleared = relay
        .input(player)
        .expect("input state remains for acknowledgement");
    assert_eq!(cleared.flags, MovementInputFlags::default());
    assert_eq!(cleared.facing, Some(-0.5));
    assert_eq!(cleared.acknowledgement, 1);
}

#[test]
fn relay_rejects_tick_regression_before_mutating_input_state() {
    let player = actor(19, 1);
    let mut relay = MovementInputRelay::default();
    let first = MovementFrameBatch::new(vec![frame(player, 1, flags(true, false), None)])
        .expect("frame is valid");
    relay.apply_batch(40, &first).expect("first tick applies");

    let later = MovementFrameBatch::new(vec![frame(player, 2, flags(false, true), None)])
        .expect("frame is valid");
    assert!(matches!(
        relay.apply_batch(39, &later),
        Err(MovementInputError::TickRegression {
            previous: 40,
            actual: 39,
        })
    ));
    let retained = relay.input(player).expect("original input remains");
    assert_eq!(retained.acknowledgement, 1);
    assert_eq!(retained.flags, flags(true, false));
}
