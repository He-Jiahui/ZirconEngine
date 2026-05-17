use serde::{Deserialize, Serialize};

use crate::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneEditorHierarchyRow {
    pub entity: EntityId,
    pub parent: Option<EntityId>,
    pub depth: u32,
    pub display_name: String,
    pub kind: String,
    pub selected: bool,
    pub active_in_hierarchy: bool,
    pub has_children: bool,
}
