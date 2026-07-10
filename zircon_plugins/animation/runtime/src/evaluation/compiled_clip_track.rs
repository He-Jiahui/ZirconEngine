use zircon_runtime::asset::AnimationChannelAsset;

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
