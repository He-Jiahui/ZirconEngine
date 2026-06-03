use std::collections::BTreeSet;

use crate::graphics::feature::{BuiltinRenderFeature, RenderFeatureCapabilityRequirement};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderPipelineCompileOptions {
    pub enabled_features: BTreeSet<BuiltinRenderFeature>,
    pub disabled_features: BTreeSet<BuiltinRenderFeature>,
    pub disabled_plugin_features: BTreeSet<String>,
    pub enabled_capabilities: BTreeSet<RenderFeatureCapabilityRequirement>,
    pub allow_async_compute: bool,
    pub graph_msaa_sample_count: Option<u32>,
}
