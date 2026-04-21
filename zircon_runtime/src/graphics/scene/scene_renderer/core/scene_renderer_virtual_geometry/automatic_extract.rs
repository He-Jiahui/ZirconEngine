use std::collections::BTreeMap;

use crate::asset::ModelAsset;
use crate::core::framework::render::RenderMeshSnapshot;
use crate::core::resource::ResourceId;
use crate::graphics::runtime::{
    build_virtual_geometry_automatic_extract_from_meshes, VirtualGeometryAutomaticExtractOutput,
};
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn synthesize_virtual_geometry_extract(
        &self,
        meshes: &[RenderMeshSnapshot],
    ) -> Option<VirtualGeometryAutomaticExtractOutput> {
        let mut cached_models = BTreeMap::<ResourceId, Option<ModelAsset>>::new();
        build_virtual_geometry_automatic_extract_from_meshes(meshes, |model_id| {
            cached_models
                .entry(model_id)
                .or_insert_with(|| self.streamer.load_model_asset(model_id))
                .clone()
        })
    }
}
