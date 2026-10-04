use super::{HzbBuildPlan, HzbBuilder};
use crate::core::math::UVec2;

#[test]
fn hzb_builder_sizes_odd_viewport_to_half_power_of_two_chain() {
    let plan = HzbBuilder::new(UVec2::new(1923, 1081)).build_plan();

    assert_eq!(plan.view_size, UVec2::new(1923, 1081));
    assert_eq!(plan.hzb_size, UVec2::new(1024, 1024));
    assert_eq!(plan.mip_count, 11);
    assert_eq!(plan.reduce_pass_count, 3);
}

#[test]
fn hzb_builder_keeps_one_pixel_viewports_valid() {
    let plan = HzbBuilder::new(UVec2::new(1, 1)).build_plan();

    assert_eq!(plan.hzb_size, UVec2::new(1, 1));
    assert_eq!(plan.mip_count, 1);
    assert_eq!(plan.reduce_pass_count, 1);
}

#[test]
fn hzb_builder_reduce_passes_cover_tail_mips() {
    let plan = HzbBuilder::new(UVec2::new(256, 128)).build_plan();

    assert!(plan.reduce_pass_count * HzbBuildPlan::max_mips_per_reduce_pass() >= plan.mip_count);
    assert!(
        (plan.reduce_pass_count - 1) * HzbBuildPlan::max_mips_per_reduce_pass() < plan.mip_count
    );
}

#[test]
fn hzb_build_plan_reports_each_mip_extent() {
    let plan = HzbBuilder::new(UVec2::new(1923, 1081)).build_plan();

    assert_eq!(plan.mip_size(0), UVec2::new(1024, 1024));
    assert_eq!(plan.mip_size(1), UVec2::new(512, 512));
    assert_eq!(plan.mip_size(10), UVec2::new(1, 1));
    assert_eq!(plan.mip_size(11), UVec2::new(1, 1));
}
