use super::*;

#[test]
fn stage_defaults_select_the_canonical_clock_domain_and_pause_contract() {
    let update = SceneSystemTickPolicy::for_stage(SystemStage::Update);
    let fixed = SceneSystemTickPolicy::for_stage(SystemStage::FixedUpdate);

    assert_eq!(update.clock_domain(), SceneSystemClockDomain::Virtual);
    assert_eq!(
        update.pause_behavior(),
        SceneSystemPauseBehavior::SkipWhenVirtualPaused
    );
    assert_eq!(fixed.clock_domain(), SceneSystemClockDomain::Fixed);
    assert!(update.is_valid_for_stage(SystemStage::Update));
    assert!(fixed.is_valid_for_stage(SystemStage::FixedUpdate));
}

#[test]
fn invalid_fixed_policy_combinations_are_rejected_before_schedule_execution() {
    assert!(!SceneSystemTickPolicy::monotonic_real().is_valid_for_stage(SystemStage::FixedUpdate));
    assert!(!SceneSystemTickPolicy::fixed().is_valid_for_stage(SystemStage::Update));
    assert!(!SceneSystemTickPolicy::new(
        SceneSystemClockDomain::Fixed,
        SceneSystemPauseBehavior::RunWhenVirtualPaused,
    )
    .is_valid_for_stage(SystemStage::FixedUpdate));
}
