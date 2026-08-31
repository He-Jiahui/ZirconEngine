use std::fmt;

pub(crate) const MIP_GEN_MIPS_PER_DISPATCH: u32 = 4;
pub(crate) const MIP_GEN_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MipGenDispatch {
    source_mip_level: u32,
    first_target_mip_level: u32,
    generated_mip_count: u32,
    target_extent: [u32; 2],
    workgroup_count: [u32; 3],
}

impl MipGenDispatch {
    const fn new(
        source_mip_level: u32,
        first_target_mip_level: u32,
        generated_mip_count: u32,
        target_extent: [u32; 2],
        array_layer_count: u32,
    ) -> Self {
        Self {
            source_mip_level,
            first_target_mip_level,
            generated_mip_count,
            target_extent,
            workgroup_count: [
                target_extent[0].div_ceil(MIP_GEN_WORKGROUP_SIZE[0]),
                target_extent[1].div_ceil(MIP_GEN_WORKGROUP_SIZE[1]),
                array_layer_count,
            ],
        }
    }

    pub(crate) const fn source_mip_level(&self) -> u32 {
        self.source_mip_level
    }

    pub(crate) const fn first_target_mip_level(&self) -> u32 {
        self.first_target_mip_level
    }

    pub(crate) const fn generated_mip_count(&self) -> u32 {
        self.generated_mip_count
    }

    pub(crate) const fn target_extent(&self) -> [u32; 2] {
        self.target_extent
    }

    pub(crate) const fn workgroup_count(&self) -> [u32; 3] {
        self.workgroup_count
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MipGenDispatchPlan {
    texture_extent: [u32; 2],
    array_layer_count: u32,
    mip_level_count: u32,
    dispatches: Vec<MipGenDispatch>,
}

impl MipGenDispatchPlan {
    pub(crate) fn new(
        width: u32,
        height: u32,
        array_layer_count: u32,
        mip_level_count: u32,
    ) -> Result<Self, MipGenPlanError> {
        if width == 0 || height == 0 {
            return Err(MipGenPlanError::ZeroExtent { width, height });
        }
        if array_layer_count == 0 {
            return Err(MipGenPlanError::ZeroArrayLayers);
        }
        if mip_level_count == 0 {
            return Err(MipGenPlanError::ZeroMipLevels);
        }

        let max_mip_level_count = full_mip_level_count(width, height);
        if mip_level_count > max_mip_level_count {
            return Err(MipGenPlanError::MipLevelsExceedExtent {
                requested: mip_level_count,
                maximum: max_mip_level_count,
            });
        }

        let mut dispatches = Vec::with_capacity(
            ((mip_level_count.saturating_sub(1)).div_ceil(MIP_GEN_MIPS_PER_DISPATCH)) as usize,
        );
        let mut first_target_mip_level = 1;
        while first_target_mip_level < mip_level_count {
            let generated_mip_count =
                (mip_level_count - first_target_mip_level).min(MIP_GEN_MIPS_PER_DISPATCH);
            let target_extent = [
                mip_extent(width, first_target_mip_level),
                mip_extent(height, first_target_mip_level),
            ];
            dispatches.push(MipGenDispatch::new(
                first_target_mip_level - 1,
                first_target_mip_level,
                generated_mip_count,
                target_extent,
                array_layer_count,
            ));
            first_target_mip_level += generated_mip_count;
        }

        Ok(Self {
            texture_extent: [width, height],
            array_layer_count,
            mip_level_count,
            dispatches,
        })
    }

    pub(crate) const fn texture_extent(&self) -> [u32; 2] {
        self.texture_extent
    }

    pub(crate) const fn array_layer_count(&self) -> u32 {
        self.array_layer_count
    }

    pub(crate) const fn mip_level_count(&self) -> u32 {
        self.mip_level_count
    }

    pub(crate) fn dispatches(&self) -> &[MipGenDispatch] {
        &self.dispatches
    }

    pub(crate) fn dispatch_count(&self) -> u32 {
        self.dispatches.len() as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MipGenPlanError {
    ZeroExtent { width: u32, height: u32 },
    ZeroArrayLayers,
    ZeroMipLevels,
    MipLevelsExceedExtent { requested: u32, maximum: u32 },
}

impl fmt::Display for MipGenPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroExtent { width, height } => {
                write!(
                    formatter,
                    "mip generation extent must be non-zero, got {width}x{height}"
                )
            }
            Self::ZeroArrayLayers => {
                formatter.write_str("mip generation requires at least one array layer")
            }
            Self::ZeroMipLevels => {
                formatter.write_str("mip generation requires at least one mip level")
            }
            Self::MipLevelsExceedExtent { requested, maximum } => write!(
                formatter,
                "mip generation requested {requested} levels, but the texture extent supports at most {maximum}"
            ),
        }
    }
}

impl std::error::Error for MipGenPlanError {}

fn full_mip_level_count(mut width: u32, mut height: u32) -> u32 {
    let mut count = 1;
    while width > 1 || height > 1 {
        width = (width / 2).max(1);
        height = (height / 2).max(1);
        count += 1;
    }
    count
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    if level >= u32::BITS {
        1
    } else {
        let shifted = value >> level;
        if shifted == 0 { 1 } else { shifted }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_mipgen_pass_four_mips_per_dispatch() {
        let plan = MipGenDispatchPlan::new(2_048, 2_048, 1, 12)
            .expect("12 levels fit a 2048 square texture");

        assert_eq!(plan.dispatch_count(), 3);
        assert_eq!(
            plan.dispatches()
                .iter()
                .map(MipGenDispatch::source_mip_level)
                .collect::<Vec<_>>(),
            vec![0, 4, 8]
        );
        assert_eq!(
            plan.dispatches()
                .iter()
                .map(MipGenDispatch::first_target_mip_level)
                .collect::<Vec<_>>(),
            vec![1, 5, 9]
        );
        assert_eq!(
            plan.dispatches()
                .iter()
                .map(MipGenDispatch::generated_mip_count)
                .collect::<Vec<_>>(),
            vec![4, 4, 3]
        );
        assert_eq!(plan.dispatches()[0].target_extent(), [1_024, 1_024]);
        assert_eq!(plan.dispatches()[1].target_extent(), [64, 64]);
        assert_eq!(plan.dispatches()[2].target_extent(), [4, 4]);
    }

    #[test]
    fn mipgen_dispatch_plan_preserves_array_layers_in_workgroup_depth() {
        let plan = MipGenDispatchPlan::new(64, 32, 6, 7)
            .expect("six-layer texture has a valid full mip chain");

        assert_eq!(plan.texture_extent(), [64, 32]);
        assert_eq!(plan.array_layer_count(), 6);
        assert_eq!(plan.mip_level_count(), 7);
        assert_eq!(plan.dispatch_count(), 2);
        assert_eq!(plan.dispatches()[0].workgroup_count(), [4, 2, 6]);
        assert_eq!(plan.dispatches()[1].workgroup_count(), [1, 1, 6]);
    }

    #[test]
    fn mipgen_dispatch_plan_rejects_impossible_texture_descriptions() {
        assert_eq!(
            MipGenDispatchPlan::new(0, 1, 1, 1),
            Err(MipGenPlanError::ZeroExtent {
                width: 0,
                height: 1
            })
        );
        assert_eq!(
            MipGenDispatchPlan::new(1, 1, 0, 1),
            Err(MipGenPlanError::ZeroArrayLayers)
        );
        assert_eq!(
            MipGenDispatchPlan::new(1, 1, 1, 0),
            Err(MipGenPlanError::ZeroMipLevels)
        );
        assert_eq!(
            MipGenDispatchPlan::new(4, 4, 1, 4),
            Err(MipGenPlanError::MipLevelsExceedExtent {
                requested: 4,
                maximum: 3
            })
        );
    }
}
