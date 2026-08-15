use std::path::Path;

use crate::core::editor_message::DocumentId;

use super::DocumentLifecycleState;

/// Read-only retention and probe data for a document lifecycle authority.
///
/// Path byte counts are sampled from existing `PathBuf` or `String` owners. Reading this type
/// does not clone an identity or reset the cumulative probe counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentLifecycleRetentionSnapshot {
    /// Number of retained project-root identities.
    pub root_identity_count: usize,
    /// Number of retained scene identities.
    pub scene_identity_count: usize,
    /// Logical byte count held by retained project-root identities.
    pub root_path_bytes: usize,
    /// Logical byte count of project roots held by retained scene identities.
    pub scene_project_root_path_bytes: usize,
    /// Logical byte count held by retained scene URIs.
    pub scene_uri_bytes: usize,
    /// Number of active project-session root owners.
    pub active_project_session_count: usize,
    /// Logical byte count held by the active project-session root owner.
    pub active_project_session_root_path_bytes: usize,
    /// Number of active scene identity owners.
    pub active_scene_identity_count: usize,
    /// Number of active document ids.
    pub active_document_count: usize,
    /// The active document id when one is selected.
    pub active_document_id: Option<DocumentId>,
    /// Cumulative document-id occupancy index probes.
    pub document_id_occupancy_probe_count: u64,
    /// Cumulative retained-root entries inspected while finding an eviction candidate.
    pub root_eviction_scan_entry_count: u64,
    /// Cumulative retained-root identity evictions.
    pub root_eviction_count: u64,
    /// Cumulative retained-scene entries inspected while finding an eviction candidate.
    pub scene_eviction_scan_entry_count: u64,
    /// Cumulative retained-scene identity evictions.
    pub scene_eviction_count: u64,
}

impl DocumentLifecycleRetentionSnapshot {
    pub(super) fn from_state(state: &DocumentLifecycleState) -> Self {
        let active_project_session_count = usize::from(state.active_project_session.is_some());
        let active_project_session_root_path_bytes = state
            .active_project_session
            .as_ref()
            .map_or(0, |session| path_byte_len(&session.root));
        Self {
            root_identity_count: state.ids_by_root.len(),
            scene_identity_count: state.ids_by_scene_key.len(),
            root_path_bytes: state
                .ids_by_root
                .keys()
                .fold(0, |bytes, root| bytes.saturating_add(path_byte_len(root))),
            scene_project_root_path_bytes: state.ids_by_scene_key.keys().fold(0, |bytes, key| {
                bytes.saturating_add(path_byte_len(&key.project_root))
            }),
            scene_uri_bytes: state
                .ids_by_scene_key
                .keys()
                .fold(0, |bytes, key| bytes.saturating_add(key.scene_uri.len())),
            active_project_session_count,
            active_project_session_root_path_bytes,
            active_scene_identity_count: usize::from(state.active_scene_key.is_some()),
            active_document_count: usize::from(state.active_document.is_some()),
            active_document_id: state.active_document,
            document_id_occupancy_probe_count: state
                .probe_counters
                .document_id_occupancy_probe_count,
            root_eviction_scan_entry_count: state.probe_counters.root_eviction_scan_entry_count,
            root_eviction_count: state.probe_counters.root_eviction_count,
            scene_eviction_scan_entry_count: state.probe_counters.scene_eviction_scan_entry_count,
            scene_eviction_count: state.probe_counters.scene_eviction_count,
        }
    }
}

#[derive(Default)]
pub(super) struct DocumentLifecycleProbeCounters {
    document_id_occupancy_probe_count: u64,
    root_eviction_scan_entry_count: u64,
    root_eviction_count: u64,
    scene_eviction_scan_entry_count: u64,
    scene_eviction_count: u64,
}

impl DocumentLifecycleProbeCounters {
    pub(super) fn record_document_id_occupancy_probe(&mut self) {
        self.document_id_occupancy_probe_count =
            self.document_id_occupancy_probe_count.saturating_add(1);
    }

    pub(super) fn record_root_eviction_scan(&mut self, scanned_entries: usize) {
        self.root_eviction_scan_entry_count = self
            .root_eviction_scan_entry_count
            .saturating_add(scanned_entries as u64);
    }

    pub(super) fn record_root_eviction(&mut self) {
        self.root_eviction_count = self.root_eviction_count.saturating_add(1);
    }

    pub(super) fn record_scene_eviction_scan(&mut self, scanned_entries: usize) {
        self.scene_eviction_scan_entry_count = self
            .scene_eviction_scan_entry_count
            .saturating_add(scanned_entries as u64);
    }

    pub(super) fn record_scene_eviction(&mut self) {
        self.scene_eviction_count = self.scene_eviction_count.saturating_add(1);
    }
}

fn path_byte_len(path: &Path) -> usize {
    path.as_os_str().len()
}
