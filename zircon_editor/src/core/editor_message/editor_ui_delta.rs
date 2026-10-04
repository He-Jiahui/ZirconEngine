use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::event_ui::{UiNodePath, UiReflectionNodePatch};

use crate::core::editor_event::{EditorEventSequence, ViewInstanceId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorUiDeltaBarrierKind {
    Press,
    Release,
    Scroll,
    Focus,
    Geometry,
    Commit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorUiNodeDelta {
    view: ViewInstanceId,
    patch: UiReflectionNodePatch,
}

impl EditorUiNodeDelta {
    pub fn new(view: ViewInstanceId, patch: UiReflectionNodePatch) -> Self {
        Self { view, patch }
    }

    pub fn view(&self) -> &ViewInstanceId {
        &self.view
    }

    pub fn patch(&self) -> &UiReflectionNodePatch {
        &self.patch
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorUiDeltaEntry {
    Nodes(Vec<EditorUiNodeDelta>),
    Barrier {
        kind: EditorUiDeltaBarrierKind,
        sequence: EditorEventSequence,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorUiDeltaBatch {
    entries: Vec<EditorUiDeltaEntry>,
}

impl EditorUiDeltaBatch {
    pub fn entries(&self) -> &[EditorUiDeltaEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn node_delta_count(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| match entry {
                EditorUiDeltaEntry::Nodes(deltas) => deltas.len(),
                EditorUiDeltaEntry::Barrier { .. } => 0,
            })
            .sum()
    }

    pub fn barrier_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| matches!(entry, EditorUiDeltaEntry::Barrier { .. }))
            .count()
    }

    pub fn reflection_patches(&self) -> Vec<UiReflectionNodePatch> {
        let mut patches = Vec::with_capacity(self.node_delta_count());
        for entry in &self.entries {
            let EditorUiDeltaEntry::Nodes(deltas) = entry else {
                continue;
            };
            patches.extend(deltas.iter().map(|delta| delta.patch.clone()));
        }
        patches
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EditorUiDeltaQueue {
    entries: Vec<EditorUiDeltaEntry>,
    pending: HashMap<UiNodePath, EditorUiNodeDelta>,
}

impl EditorUiDeltaQueue {
    pub fn push_patch(&mut self, view: ViewInstanceId, patch: UiReflectionNodePatch) {
        // Runtime reflection paths are global identities. Keep the latest view as provenance,
        // but coalesce by the identity that the Runtime patch API actually mutates.
        let key = patch.node_path.clone();
        let pending = self.pending.entry(key).or_insert_with(|| {
            EditorUiNodeDelta::new(
                view.clone(),
                UiReflectionNodePatch::new(patch.node_path.clone()),
            )
        });
        pending.view = view;
        pending.patch.properties.extend(patch.properties);
        if patch.pressed.is_some() {
            pending.patch.pressed = patch.pressed;
        }
    }

    pub fn push_barrier(&mut self, kind: EditorUiDeltaBarrierKind, sequence: EditorEventSequence) {
        self.flush_pending();
        self.entries
            .push(EditorUiDeltaEntry::Barrier { kind, sequence });
    }

    pub fn drain(&mut self) -> EditorUiDeltaBatch {
        self.flush_pending();
        EditorUiDeltaBatch {
            entries: std::mem::take(&mut self.entries),
        }
    }

    fn flush_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let mut deltas = std::mem::take(&mut self.pending)
            .into_values()
            .collect::<Vec<_>>();
        deltas.sort_unstable_by(|left, right| left.patch.node_path.cmp(&right.patch.node_path));
        self.entries.push(EditorUiDeltaEntry::Nodes(deltas));
    }
}

#[cfg(test)]
#[path = "tests/editor_ui_delta.rs"]
mod tests;

#[cfg(test)]
#[path = "editor_ui_delta/tests/hash_coalescing_tests.rs"]
mod hash_coalescing_tests;
