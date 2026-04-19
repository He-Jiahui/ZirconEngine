use crate::CompiledRenderPipeline;

pub(in crate::graphics::runtime::render_framework) fn compiled_feature_names(
    pipeline: &CompiledRenderPipeline,
) -> Vec<String> {
    pipeline
        .enabled_features
        .iter()
        .map(|feature| feature.feature.descriptor().name)
        .collect()
}
