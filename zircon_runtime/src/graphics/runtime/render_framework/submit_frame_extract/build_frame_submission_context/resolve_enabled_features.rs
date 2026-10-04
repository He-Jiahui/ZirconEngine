//! 最终启用特性以已编译图和高级运行时计划交集为准，原始场景请求不能单独启动 provider。
use crate::core::framework::render::AdvancedProfileRuntimePlan;
use crate::graphics::CompiledRenderPipeline;

pub(super) fn resolve_enabled_features(
    compiled_pipeline: &CompiledRenderPipeline,
    advanced_runtime_plan: &AdvancedProfileRuntimePlan,
) -> (bool, bool) {
    let runtime_features = compiled_pipeline.runtime_feature_flags();

    (
        runtime_features.hybrid_global_illumination_enabled
            && advanced_runtime_plan.hybrid_global_illumination_enabled(),
        runtime_features.virtual_geometry_enabled
            && advanced_runtime_plan.virtual_geometry_enabled(),
    )
}

#[cfg(test)]
#[path = "tests/resolve_enabled_features.rs"]
mod tests;
