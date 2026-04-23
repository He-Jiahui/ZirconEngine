use std::collections::BTreeMap;

use crate::asset::ModelAsset;
use crate::core::framework::render::{RenderMeshSnapshot, RenderVirtualGeometryDebugState};
use crate::core::resource::ResourceId;
use crate::graphics::runtime::{
    build_virtual_geometry_automatic_extract_from_meshes_with_debug,
    VirtualGeometryAutomaticExtractOutput,
};
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn synthesize_virtual_geometry_extract(
        &self,
        meshes: &[RenderMeshSnapshot],
        debug_override: Option<RenderVirtualGeometryDebugState>,
    ) -> Option<VirtualGeometryAutomaticExtractOutput> {
        let mut cached_models = BTreeMap::<ResourceId, Option<ModelAsset>>::new();
        build_virtual_geometry_automatic_extract_from_meshes_with_debug(
            meshes,
            debug_override.unwrap_or_default(),
            |model_id| {
                cached_models
                    .entry(model_id)
                    .or_insert_with(|| self.streamer.load_model_asset(model_id))
                    .clone()
            },
        )
    }
}
