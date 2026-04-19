use zircon_framework::render::{RenderBakedLightingExtract, RenderFrameExtract};

use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;

pub(super) fn baked_lighting(
    extract: &RenderFrameExtract,
    features: SceneRuntimeFeatureFlags,
) -> RenderBakedLightingExtract {
    if features.baked_lighting_enabled {
        extract.lighting.baked_lighting.unwrap_or_default()
    } else {
        RenderBakedLightingExtract::default()
    }
}
