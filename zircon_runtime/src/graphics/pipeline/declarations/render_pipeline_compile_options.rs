use std::collections::BTreeSet;

use crate::graphics::feature::BuiltinRenderFeature;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderPipelineCompileOptions {
    pub enabled_features: BTreeSet<BuiltinRenderFeature>,
    pub disabled_features: BTreeSet<BuiltinRenderFeature>,
    pub allow_async_compute: bool,
}
