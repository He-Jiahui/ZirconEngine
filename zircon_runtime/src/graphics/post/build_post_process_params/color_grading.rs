use crate::core::framework::render::{PostProcessExtract, RenderColorGradingSettings};

use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;

/// 编译后的特性开关控制全屏调色入口，停用时返回中性设置供共享参数布局继续使用。
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
