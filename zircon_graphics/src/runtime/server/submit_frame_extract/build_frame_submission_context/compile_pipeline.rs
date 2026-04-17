use zircon_render_server::RenderServerError;
use zircon_scene::RenderFrameExtract;

use crate::CompiledRenderPipeline;

use super::viewport_record_state::ViewportRecordState;

pub(super) fn compile_submission_pipeline(
    state: &ViewportRecordState,
    extract: &RenderFrameExtract,
) -> Result<CompiledRenderPipeline, RenderServerError> {
    state
        .pipeline_asset
        .compile_with_options(extract, &state.compile_options)
        .map_err(RenderServerError::Backend)
}
