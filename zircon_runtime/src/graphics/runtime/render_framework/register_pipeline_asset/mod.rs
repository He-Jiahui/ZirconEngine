mod register_pipeline_asset;

use crate::core::framework::render::{RenderFrameworkError, RenderPipelineHandle};
use crate::graphics::RenderPipelineAsset;

use super::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) use register_pipeline_asset::{
    compile_pipeline_for_validation, register_pipeline_asset,
};

impl WgpuRenderFramework {
    /// Registers a graphics-owned pipeline asset with the concrete WGPU runtime.
    ///
    /// Pipeline authoring stays above the neutral `RenderFramework` contract; cross-domain users
    /// select or reload already registered pipelines through stable handles.
    pub fn register_pipeline_asset(
        &self,
        pipeline: RenderPipelineAsset,
    ) -> Result<RenderPipelineHandle, RenderFrameworkError> {
        register_pipeline_asset(self, pipeline)
    }
}
