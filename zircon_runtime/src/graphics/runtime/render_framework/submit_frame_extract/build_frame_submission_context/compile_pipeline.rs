use crate::core::framework::render::{RenderFrameExtract, RenderFrameworkError};

use crate::{CompiledRenderPipeline, RenderPipelineCompileOptions};

use super::super::super::capability_validation::validate_compiled_pipeline_capabilities;
use super::viewport_record_state::ViewportRecordState;

pub(super) fn compile_submission_pipeline(
    state: &ViewportRecordState,
    extract: &RenderFrameExtract,
) -> Result<CompiledRenderPipeline, RenderFrameworkError> {
    compile_submission_pipeline_with_options(state, extract, state.compile_options())
}

pub(super) fn compile_submission_pipeline_with_options(
    state: &ViewportRecordState,
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
) -> Result<CompiledRenderPipeline, RenderFrameworkError> {
    let compiled = state
        .pipeline_asset()
        .compile_with_options(extract, options)
        .map_err(RenderFrameworkError::Backend)?;
    validate_compiled_pipeline_capabilities(&compiled, state.capabilities())?;
    Ok(compiled)
}
