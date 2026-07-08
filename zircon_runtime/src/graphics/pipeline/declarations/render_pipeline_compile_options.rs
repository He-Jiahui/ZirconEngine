use std::collections::BTreeSet;

use crate::core::framework::render::{IblBakeArtifactRequest, PostProcessStackDescriptor};
use crate::graphics::feature::{BuiltinRenderFeature, RenderFeatureCapabilityRequirement};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RenderPipelineCompileOptions {
    pub enabled_features: BTreeSet<BuiltinRenderFeature>,
    pub disabled_features: BTreeSet<BuiltinRenderFeature>,
    pub disabled_plugin_features: BTreeSet<String>,
    pub enabled_capabilities: BTreeSet<RenderFeatureCapabilityRequirement>,
    pub allow_async_compute: bool,
    pub enable_hzb_occlusion_culling: bool,
    pub graph_msaa_sample_count: Option<u32>,
    pub post_process_stack: Option<PostProcessStackDescriptor>,
    pub environment_ibl_bake_request: Option<IblBakeArtifactRequest>,
}
