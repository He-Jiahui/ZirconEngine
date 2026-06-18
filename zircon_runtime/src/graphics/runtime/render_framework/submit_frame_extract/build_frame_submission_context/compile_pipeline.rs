use crate::core::framework::render::{RenderFrameExtract, RenderFrameworkError};

use std::sync::Arc;

use crate::graphics::pipeline::{extract_compile_fingerprint, CompiledGraphCacheKey};
use crate::graphics::{CompiledRenderPipeline, RenderPipelineCompileOptions, WgpuRenderFramework};

use super::super::super::capability_validation::validate_compiled_pipeline_capabilities;
use super::viewport_record_state::ViewportRecordState;

pub(super) fn compile_submission_pipeline(
    server: &WgpuRenderFramework,
    state: &ViewportRecordState,
    extract: &RenderFrameExtract,
) -> Result<Arc<CompiledRenderPipeline>, RenderFrameworkError> {
    compile_submission_pipeline_with_options(server, state, extract, state.compile_options())
}

pub(super) fn compile_submission_pipeline_with_options(
    server: &WgpuRenderFramework,
    state: &ViewportRecordState,
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
) -> Result<Arc<CompiledRenderPipeline>, RenderFrameworkError> {
    let key = CompiledGraphCacheKey::from_inputs(
        state.pipeline_asset(),
        extract,
        options,
        state.capabilities(),
        state.shader_quality(),
    );
    let frame_fingerprint = key.frame;
    let mut framework_state = server.lock_state();
    let lookup = framework_state
        .compiled_graph_cache
        .get_or_compile_with_status(key, || {
            let compiled = state
                .pipeline_asset()
                .compile_with_options(extract, options)?;
            validate_compiled_pipeline_capabilities(&compiled, state.capabilities())
                .map_err(|error| error.to_string())?;
            Ok(compiled)
        })
        .map_err(RenderFrameworkError::Backend)?;
    if lookup.status.is_hit() {
        debug_assert_eq!(
            frame_fingerprint,
            extract_compile_fingerprint(extract),
            "compiled graph cache hit used a stale render frame compile fingerprint"
        );
    }
    Ok(lookup.pipeline)
}
