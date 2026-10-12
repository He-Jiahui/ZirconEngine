use super::RenderReflectionProbeWorkloadReport;
use crate::core::math::UVec2;

#[test]
fn reflection_probe_workload_derives_full_resolution_visit_upper_bound() {
    let report = RenderReflectionProbeWorkloadReport {
        active_probe_count: 4,
        ..RenderReflectionProbeWorkloadReport::default()
    }
    .with_render_size(UVec2::new(1_920, 1_080));

    assert_eq!(
        report.full_resolution_fragment_probe_visit_upper_bound,
        8_294_400
    );
}

#[test]
fn reflection_probe_workload_visit_upper_bound_saturates() {
    let report = RenderReflectionProbeWorkloadReport {
        active_probe_count: usize::MAX,
        ..RenderReflectionProbeWorkloadReport::default()
    }
    .with_render_size(UVec2::new(u32::MAX, u32::MAX));

    assert_eq!(
        report.full_resolution_fragment_probe_visit_upper_bound,
        u64::MAX
    );
}
