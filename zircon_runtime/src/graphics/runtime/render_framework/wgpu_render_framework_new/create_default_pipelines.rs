use std::collections::HashMap;

use crate::core::framework::render::RenderPipelineHandle;

use crate::graphics::RenderFeatureDescriptor;
use crate::RenderPipelineAsset;

pub(super) fn create_default_pipelines(
    render_features: &[RenderFeatureDescriptor],
) -> HashMap<RenderPipelineHandle, RenderPipelineAsset> {
    let mut forward = RenderPipelineAsset::default_forward_plus();
    let mut deferred = RenderPipelineAsset::default_deferred();
    forward.apply_plugin_render_features(render_features.iter().cloned());
    deferred.apply_plugin_render_features(render_features.iter().cloned());
    HashMap::from([(forward.handle, forward), (deferred.handle, deferred)])
}
