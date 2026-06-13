use crate::core::math::UVec2;

const HZB_VIEW_SIZE_SHIFT: u32 = 1;
const MAX_MIPS_PER_REDUCE_PASS: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HzbBuilder {
    pub view_size: UVec2,
}

impl HzbBuilder {
    pub const fn new(view_size: UVec2) -> Self {
        Self { view_size }
    }

    pub fn build_plan(self) -> HzbBuildPlan {
        let hzb_size = hzb_size_for_view(self.view_size);
        let mip_count = full_mip_chain_level_count(hzb_size);

        HzbBuildPlan {
            view_size: self.view_size,
            hzb_size,
            mip_count,
            reduce_pass_count: mip_count.div_ceil(MAX_MIPS_PER_REDUCE_PASS),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HzbBuildPlan {
    pub view_size: UVec2,
    pub hzb_size: UVec2,
    pub mip_count: u32,
    pub reduce_pass_count: u32,
}

impl HzbBuildPlan {
    pub const fn max_mips_per_reduce_pass() -> u32 {
        MAX_MIPS_PER_REDUCE_PASS
    }

    pub fn mip_size(self, mip_level: u32) -> UVec2 {
        UVec2::new(
            self.hzb_size.x.checked_shr(mip_level).unwrap_or(0).max(1),
            self.hzb_size.y.checked_shr(mip_level).unwrap_or(0).max(1),
        )
    }
}

fn hzb_size_for_view(view_size: UVec2) -> UVec2 {
    UVec2::new(
        hzb_extent_for_view_axis(view_size.x),
        hzb_extent_for_view_axis(view_size.y),
    )
}

fn hzb_extent_for_view_axis(value: u32) -> u32 {
    value
        .max(1)
        .next_power_of_two()
        .checked_shr(HZB_VIEW_SIZE_SHIFT)
        .unwrap_or(0)
        .max(1)
}

fn full_mip_chain_level_count(size: UVec2) -> u32 {
    u32::BITS - size.x.max(size.y).max(1).leading_zeros()
}

#[cfg(test)]
mod tests {
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

        assert!(
            plan.reduce_pass_count * HzbBuildPlan::max_mips_per_reduce_pass() >= plan.mip_count
        );
        assert!(
            (plan.reduce_pass_count - 1) * HzbBuildPlan::max_mips_per_reduce_pass()
                < plan.mip_count
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
}
