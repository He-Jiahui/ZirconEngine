use super::*;

#[test]
fn hzb_occlusion_dispatch_groups_cover_indirect_args() {
    assert_eq!(dispatch_group_count(0), 0);
    assert_eq!(dispatch_group_count(1), 1);
    assert_eq!(dispatch_group_count(64), 1);
    assert_eq!(dispatch_group_count(65), 2);
}

#[test]
fn hzb_occlusion_dispatch_groups_sum_phase_local_workloads() {
    assert_eq!(dispatch_group_count(3), 1);
    assert_eq!(dispatch_group_count_for_phase_arg_counts([1, 1, 1]), 3);
    assert_eq!(dispatch_group_count_for_phase_arg_counts([64, 65, 0]), 3);
    assert_eq!(
        dispatch_group_count_for_phase_arg_counts([u32::MAX]),
        u32::MAX.div_ceil(64)
    );
}

#[test]
fn hzb_occlusion_dispatch_summary_saturates_phase_and_group_counts() {
    let mut summary = HzbOcclusionPhaseDispatchSummary::default();

    summary.record_dispatch_group_count(u32::MAX);
    summary.record_dispatch_group_count(1);

    assert_eq!(summary.dispatched_phase_count(), 2);
    assert_eq!(summary.dispatch_group_count(), u32::MAX);
}
