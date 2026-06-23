use crate::core::framework::render::RenderLayerSet;
use crate::core::framework::scene::Mobility;
use crate::core::resource::ResourceId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VisibilityBatchKey {
    pub render_layer_mask: RenderLayerSet,
    pub material_id: ResourceId,
    pub model_id: ResourceId,
    pub mobility: Mobility,
}
