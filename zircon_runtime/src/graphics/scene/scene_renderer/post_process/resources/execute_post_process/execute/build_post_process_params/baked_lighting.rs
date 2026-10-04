use crate::core::framework::render::{RenderBakedLightingExtract, RenderFrameExtract};

use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;

/// 全屏组合不叠加旧烘焙环境项；逐表面的 lightmap 链已负责烘焙照明，避免重复累计。
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
