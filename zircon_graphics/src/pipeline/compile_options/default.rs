use std::collections::BTreeSet;

use crate::pipeline::declarations::RenderPipelineCompileOptions;

impl Default for RenderPipelineCompileOptions {
    fn default() -> Self {
        Self {
            enabled_features: BTreeSet::new(),
            disabled_features: BTreeSet::new(),
            allow_async_compute: true,
        }
    }
}
