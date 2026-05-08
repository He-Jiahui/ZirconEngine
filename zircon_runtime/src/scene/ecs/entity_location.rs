use serde::{Deserialize, Serialize};

use crate::scene::ecs::ArchetypeId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityLocation {
    pub archetype_id: ArchetypeId,
    pub table_row: usize,
}

impl EntityLocation {
    pub const fn new(archetype_id: ArchetypeId, table_row: usize) -> Self {
        Self {
            archetype_id,
            table_row,
        }
    }
}
