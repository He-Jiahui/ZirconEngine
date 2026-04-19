use crate::core::framework::render::{RenderFrameExtract, VisibilityRenderableInput};

pub(crate) fn visibility_entries(extract: &RenderFrameExtract) -> Vec<VisibilityRenderableInput> {
    if !extract.visibility.renderables.is_empty() {
        return extract.visibility.renderables.clone();
    }

    extract
        .geometry
        .meshes
        .iter()
        .map(|mesh| VisibilityRenderableInput {
            entity: mesh.node_id,
            mobility: mesh.mobility,
            render_layer_mask: mesh.render_layer_mask,
        })
        .collect()
}
