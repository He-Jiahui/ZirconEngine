use crate::core::framework::render::{OitBufferPlan, RenderFrameExtract, RenderFrameworkError};

use std::sync::Arc;

use crate::graphics::pipeline::{CompiledGraphCacheKey, RenderGraphCompileCameraTargetFingerprint};
use crate::graphics::{CompiledRenderPipeline, RenderPipelineCompileOptions};

use super::super::super::capability_validation::validate_compiled_pipeline_capabilities;
use super::super::super::wgpu_render_framework::WgpuRenderFrameworkAccess;
use super::viewport_record_state::ViewportRecordState;

pub(super) fn compile_submission_pipeline(
    framework: &dyn WgpuRenderFrameworkAccess,
    state: &ViewportRecordState,
    extract: &RenderFrameExtract,
    camera_target: RenderGraphCompileCameraTargetFingerprint,
) -> Result<Arc<CompiledRenderPipeline>, RenderFrameworkError> {
    compile_submission_pipeline_with_options(
        framework,
        state,
        extract,
        camera_target,
        state.compile_options(),
    )
}

pub(super) fn compile_submission_pipeline_with_options(
    framework: &dyn WgpuRenderFrameworkAccess,
    state: &ViewportRecordState,
    extract: &RenderFrameExtract,
    camera_target: RenderGraphCompileCameraTargetFingerprint,
    options: &RenderPipelineCompileOptions,
) -> Result<Arc<CompiledRenderPipeline>, RenderFrameworkError> {
    let mut options = options.clone().with_shader_quality(state.shader_quality());
    if let Some(settings) = extract.lighting.advanced_lighting.oit {
        let view_size = extract.view.effective_render_size();
        let plan = OitBufferPlan::for_view([view_size.x, view_size.y], settings);
        let max_binding_size = state.capabilities().max_storage_buffer_binding_size;
        if !plan.fits_storage_binding_size_limit(max_binding_size) {
            options = options.with_plugin_feature_disabled("oit");
        }
    }
    let key = CompiledGraphCacheKey::from_inputs(
        state.pipeline_asset(),
        extract,
        camera_target,
        &options,
        state.capabilities(),
        state.shader_quality(),
    );
    let mut framework_state = framework.lock_state();
    let lookup = framework_state
        .compiled_graph_cache
        .get_or_compile_with_status(key, || {
            let compiled = state
                .pipeline_asset()
                .compile_with_options(extract, &options)?;
            validate_compiled_pipeline_capabilities(&compiled, state.capabilities())
                .map_err(|error| error.to_string())?;
            Ok(compiled)
        })
        .map_err(RenderFrameworkError::Backend)?;
    Ok(lookup.pipeline)
}
