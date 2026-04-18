use zircon_framework::render::RenderFrameworkError;
use zircon_scene::RenderFrameExtract;

use crate::CompiledRenderPipeline;

use super::viewport_record_state::ViewportRecordState;

pub(super) fn compile_submission_pipeline(
    state: &ViewportRecordState,
    extract: &RenderFrameExtract,
) -> Result<CompiledRenderPipeline, RenderFrameworkError> {
    state
        .pipeline_asset
        .compile_with_options(extract, &state.compile_options)
        .map_err(RenderFrameworkError::Backend)
}
