use crate::core::framework::render::{PostProcessExtract, RenderColorGradingSettings};

use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;

pub(super) fn color_grading(
    post_process: &PostProcessExtract,
    features: SceneRuntimeFeatureFlags,
) -> RenderColorGradingSettings {
    if features.color_grading_enabled {
        post_process.color_grading
    } else {
        RenderColorGradingSettings::default()
    }
}
