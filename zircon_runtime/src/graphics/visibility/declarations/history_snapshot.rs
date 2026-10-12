use crate::core::framework::scene::EntityId;

use super::visibility_history_entry::VisibilityHistoryEntry;

/// 跨帧保留的可见性与资源请求边界；构建下一帧时用它区分首次、变化与延续。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibilityHistorySnapshot {
    pub instances: Vec<VisibilityHistoryEntry>,
    pub particle_emitters: Vec<EntityId>,
    pub hybrid_gi_active_probe_ids: Vec<u32>,
    pub hybrid_gi_requested_probes: Vec<u32>,
    pub virtual_geometry_visible_cluster_ids: Vec<u32>,
    pub virtual_geometry_requested_pages: Vec<u32>,
}
