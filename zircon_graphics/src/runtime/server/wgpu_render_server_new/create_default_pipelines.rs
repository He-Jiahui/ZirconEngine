use std::collections::HashMap;

use zircon_render_server::RenderPipelineHandle;

use crate::RenderPipelineAsset;

pub(super) fn create_default_pipelines() -> HashMap<RenderPipelineHandle, RenderPipelineAsset> {
    HashMap::from([
        (
            RenderPipelineAsset::default_forward_plus().handle,
            RenderPipelineAsset::default_forward_plus(),
        ),
        (
            RenderPipelineAsset::default_deferred().handle,
            RenderPipelineAsset::default_deferred(),
        ),
    ])
}
