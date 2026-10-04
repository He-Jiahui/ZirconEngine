//! 从已编译图提取生效特性名，历史兼容和产品报告应以这份最终集合为准。
use crate::graphics::CompiledRenderPipeline;

pub(in crate::graphics::runtime::render_framework) fn compiled_feature_names(
    pipeline: &CompiledRenderPipeline,
) -> Vec<String> {
    let features = pipeline.enabled_features();
    let mut names = Vec::with_capacity(features.len());
    for feature in features {
        names.push(feature.feature_name());
    }
    names
}

#[cfg(test)]
#[path = "tests/compiled_feature_names_optimization_batch_20260830bz_runtime_tests.rs"]
mod optimization_batch_20260830bz_runtime_tests;
