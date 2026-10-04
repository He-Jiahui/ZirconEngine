use crate::core::framework::scene::{EntityId, Mobility};

use super::super::RenderLayerSet;

/// 可见性规划中的一个稳定渲染实例，携带实体归属、移动性和层掩码。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityRenderableInput {
    pub entity: EntityId,
    /// Stable render-instance identity. Mesh primitives sharing an authoring entity must use
    /// distinct keys so visibility planning never collapses them by owner.
    pub stable_instance_key: u64,
    pub mobility: Mobility,
    pub render_layer_mask: RenderLayerSet,
}
