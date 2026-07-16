use crate::core::framework::animation::AnimationSkeletonAsset;
use crate::core::resource::ResourceId;

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn load_animation_skeleton_asset(
        &self,
        id: ResourceId,
    ) -> Option<AnimationSkeletonAsset> {
        self.asset_manager()
            .ok()?
            .load_animation_skeleton_asset(id)
            .ok()
    }
}
