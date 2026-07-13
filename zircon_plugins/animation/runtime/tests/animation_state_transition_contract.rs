use zircon_plugin_animation_runtime::{
    InterruptionPolicy, TransitionDesc, TransitionRequest, TransitionRuntime, TransitionState,
};

#[test]
fn interruption_policy_matrix_contract() {
    let current = TransitionState::new(1);
    let next = TransitionState::new(2);
    let other = TransitionState::new(3);

    let cases = [
        (InterruptionPolicy::None, current, false),
        (InterruptionPolicy::None, next, false),
        (InterruptionPolicy::CurrentToNext, current, true),
        (InterruptionPolicy::CurrentToNext, next, false),
        (InterruptionPolicy::NextToNext, current, false),
        (InterruptionPolicy::NextToNext, next, true),
        (InterruptionPolicy::Both, current, true),
        (InterruptionPolicy::Both, next, true),
        (InterruptionPolicy::Both, other, false),
    ];

    for (policy, requested_from, expected) in cases {
        let active = TransitionRuntime::begin(
            TransitionRequest::new(
                current,
                next,
                TransitionDesc::new(0.4).with_interruption(policy),
            ),
            0.0,
        );
        assert_eq!(active.can_interrupt_from(requested_from), expected);
    }
}

#[test]
fn transition_crossfade_pose_continuity() {
    let request = TransitionRequest::new(
        TransitionState::new(1),
        TransitionState::new(2),
        TransitionDesc::new(1.0),
    );
    let mut transition = TransitionRuntime::begin(request, 0.0);
    let mut previous_target = 0.0;

    for expected_target in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let weights = transition.crossfade_weights();
        assert!((weights.source + weights.target - 1.0).abs() <= f32::EPSILON);
        assert!((weights.target - expected_target).abs() <= f32::EPSILON);
        assert!(weights.target >= previous_target);
        previous_target = weights.target;
        transition.advance(0.25);
    }
    assert!(transition.is_complete());
}

#[test]
fn transition_exit_time_and_invalid_time_are_bounded() {
    let desc = TransitionDesc::new(f32::NAN).with_exit_time(0.75);
    assert!(!desc.exit_ready(0.5));
    assert!(desc.exit_ready(0.75));
    assert!(!desc.exit_ready(f32::NAN));

    let transition = TransitionRuntime::begin(
        TransitionRequest::new(TransitionState::new(1), TransitionState::new(2), desc),
        f32::NAN,
    );
    assert_eq!(transition.duration_seconds(), 0.0);
    assert_eq!(transition.elapsed_seconds(), 0.0);
    assert!(transition.is_complete());
    assert_eq!(transition.crossfade_weights().target, 1.0);
}
