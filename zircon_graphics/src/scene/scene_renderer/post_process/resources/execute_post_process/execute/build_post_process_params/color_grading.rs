use zircon_framework::render::{RenderColorGradingSettings, RenderFrameExtract};

use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;

pub(super) fn color_grading(
    extract: &RenderFrameExtract,
    features: SceneRuntimeFeatureFlags,
) -> RenderColorGradingSettings {
    if features.color_grading_enabled {
        extract.post_process.color_grading
    } else {
        RenderColorGradingSettings::default()
    }
}
