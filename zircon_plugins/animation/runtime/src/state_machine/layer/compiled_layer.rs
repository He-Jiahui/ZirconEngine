//! 已编译层的名称、子机引用、权重、掩码与混合模式；帧管线按顺序采样。
use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::math::Real;

use crate::{MaskWeights, PoseLayerBlendMode};

#[derive(Clone, Debug)]
pub struct CompiledStateMachineLayer {
    pub(super) name: String,
    pub(super) machine: AssetReference,
    pub(super) weight: Real,
    pub(super) blend_mode: PoseLayerBlendMode,
    pub(super) mask: Option<MaskWeights>,
}

impl CompiledStateMachineLayer {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn weight(&self) -> Real {
        self.weight
    }

    pub fn machine(&self) -> &AssetReference {
        &self.machine
    }

    pub fn blend_mode(&self) -> PoseLayerBlendMode {
        self.blend_mode
    }

    pub fn mask(&self) -> Option<&MaskWeights> {
        self.mask.as_ref()
    }
}
