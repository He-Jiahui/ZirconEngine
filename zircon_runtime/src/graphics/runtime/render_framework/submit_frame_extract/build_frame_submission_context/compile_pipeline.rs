use crate::core::framework::render::{RenderFrameExtract, RenderFrameworkError};

use crate::CompiledRenderPipeline;

use super::super::super::capability_validation::validate_compiled_pipeline_capabilities;
use super::viewport_record_state::ViewportRecordState;

pub(super) fn compile_submission_pipeline(
    state: &ViewportRecordState,
    extract: &RenderFrameExtract,
) -> Result<CompiledRenderPipeline, RenderFrameworkError> {
    let compiled = state
        .pipeline_asset()
        .compile_with_options(extract, state.compile_options())
        .map_err(RenderFrameworkError::Backend)?;
    validate_compiled_pipeline_capabilities(&compiled, state.capabilities())?;
    Ok(compiled)
}
