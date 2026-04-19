use zircon_resource::ResourceId;
use zircon_framework::scene::Mobility;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VisibilityBatchKey {
    pub render_layer_mask: u32,
    pub material_id: ResourceId,
    pub model_id: ResourceId,
    pub mobility: Mobility,
}

