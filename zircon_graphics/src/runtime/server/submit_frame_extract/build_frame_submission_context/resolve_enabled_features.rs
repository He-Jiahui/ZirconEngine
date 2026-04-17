use crate::{BuiltinRenderFeature, CompiledRenderPipeline};

pub(super) fn resolve_enabled_features(compiled_pipeline: &CompiledRenderPipeline) -> (bool, bool) {
    let hybrid_gi_enabled = compiled_pipeline
        .enabled_features
        .iter()
        .any(|feature| feature.feature == BuiltinRenderFeature::GlobalIllumination);
    let virtual_geometry_enabled = compiled_pipeline
        .enabled_features
        .iter()
        .any(|feature| feature.feature == BuiltinRenderFeature::VirtualGeometry);

    (hybrid_gi_enabled, virtual_geometry_enabled)
}
