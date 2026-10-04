use super::StagedMaterialPipelineAdmissionCycle;

#[test]
fn viewport_cycle_requires_an_observed_all_ready_candidate() {
    let mut cycle = StagedMaterialPipelineAdmissionCycle::default();

    assert_eq!(cycle.finish(), None);
    cycle.record(false);
    cycle.record(false);
    assert_eq!(cycle.finish(), Some(true));
    assert_eq!(cycle.finish(), None);
}

#[test]
fn one_deferred_camera_blocks_the_whole_viewport_cycle() {
    let mut cycle = StagedMaterialPipelineAdmissionCycle::default();

    cycle.record(false);
    cycle.record(true);
    cycle.record(false);
    assert_eq!(cycle.finish(), Some(false));
}
