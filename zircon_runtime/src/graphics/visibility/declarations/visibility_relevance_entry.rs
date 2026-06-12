use crate::core::framework::render::PrimitiveRelevance;
use crate::core::framework::scene::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityRelevanceEntry {
    pub entity: EntityId,
    pub relevance: PrimitiveRelevance,
}
