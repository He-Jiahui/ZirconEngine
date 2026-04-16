use zircon_scene::EntityId;

use super::visibility_history_entry::VisibilityHistoryEntry;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibilityHistorySnapshot {
    pub instances: Vec<VisibilityHistoryEntry>,
    pub particle_emitters: Vec<EntityId>,
    pub hybrid_gi_requested_probes: Vec<u32>,
    pub virtual_geometry_requested_pages: Vec<u32>,
}
