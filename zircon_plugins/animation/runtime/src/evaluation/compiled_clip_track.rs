//! 编译后轨道同时保留通道数据和骨架目标槽，帧采样不再解析源字符串。
use zircon_runtime::core::framework::animation::AnimationChannelAsset;

use super::TargetSlot;

/// Track payload whose scene/skeleton target was resolved before evaluation.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledClipTrack {
    pub(super) target: TargetSlot,
    pub(super) translation: AnimationChannelAsset,
    pub(super) rotation: AnimationChannelAsset,
    pub(super) scale: AnimationChannelAsset,
}

impl CompiledClipTrack {
    pub fn translation(&self) -> &AnimationChannelAsset {
        &self.translation
    }

    pub fn rotation(&self) -> &AnimationChannelAsset {
        &self.rotation
    }

    pub fn scale(&self) -> &AnimationChannelAsset {
        &self.scale
    }
}
