use std::collections::BTreeSet;

use crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA;
use crate::graphics::pipeline::declarations::RenderPipelineCompileOptions;

impl Default for RenderPipelineCompileOptions {
    fn default() -> Self {
        Self {
            enabled_features: BTreeSet::new(),
            disabled_features: BTreeSet::new(),
            disabled_plugin_features: BTreeSet::new(),
            enabled_capabilities: BTreeSet::new(),
            allow_async_compute: true,
            enable_hzb_occlusion_culling: true,
            enable_half_resolution_transparency: false,
            half_resolution_transparency_depth_sigma: DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
            graph_msaa_sample_count: None,
            shader_quality: Default::default(),
            ambient_occlusion_source: None,
            post_process_stack: None,
            environment_ibl_bake_request: None,
            advanced_lighting_inputs: None,
        }
    }
}
