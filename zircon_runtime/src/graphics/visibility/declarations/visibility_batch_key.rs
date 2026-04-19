use crate::core::framework::scene::Mobility;
use crate::core::resource::ResourceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VisibilityBatchKey {
    pub render_layer_mask: u32,
    pub material_id: ResourceId,
    pub model_id: ResourceId,
    pub mobility: Mobility,
}
