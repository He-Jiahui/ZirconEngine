use crate::core::framework::scene::{EntityId, Mobility};

use super::super::RenderLayerSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityRenderableInput {
    pub entity: EntityId,
    /// Stable render-instance identity. Mesh primitives sharing an authoring entity must use
    /// distinct keys so visibility planning never collapses them by owner.
    pub stable_instance_key: u64,
    pub mobility: Mobility,
    pub render_layer_mask: RenderLayerSet,
}
