use std::collections::BTreeSet;

use zircon_scene::RenderFrameExtract;

use super::super::super::declarations::{VisibilityHistoryEntry, VisibilityHistorySnapshot};

pub(super) fn build_history_snapshot(
    value: &RenderFrameExtract,
    history_entries: Vec<VisibilityHistoryEntry>,
    hybrid_gi_active_probe_ids: Vec<u32>,
    hybrid_gi_requested_probes: Vec<u32>,
    virtual_geometry_visible_cluster_ids: Vec<u32>,
    virtual_geometry_requested_pages: Vec<u32>,
) -> VisibilityHistorySnapshot {
    VisibilityHistorySnapshot {
        instances: history_entries,
        particle_emitters: value
            .particles
            .emitters
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        hybrid_gi_active_probe_ids,
        hybrid_gi_requested_probes,
        virtual_geometry_visible_cluster_ids,
        virtual_geometry_requested_pages,
    }
}
