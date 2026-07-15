use crate::core::framework::render::{RenderBakedLightingExtract, RenderFrameExtract};

use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;

pub(super) fn baked_lighting(
    extract: &RenderFrameExtract,
    features: SceneRuntimeFeatureFlags,
) -> RenderBakedLightingExtract {
    if features.baked_lighting_enabled {
        // Lightmaps are sampled per surface; the retired full-screen ambient term stays neutral.
        let _baked_contract = extract.environment.baked_lighting();
        RenderBakedLightingExtract::default()
    } else {
        RenderBakedLightingExtract::default()
    }
}
