use std::collections::{HashMap, HashSet};

use zircon_runtime::scene::WorldInspectionHierarchyRow;

use crate::core::editor_message::SceneInspectionMessage;

use super::SceneEntries;

/// A generation-checked retained hierarchy update.
///
/// `Patch` deliberately carries only exact changed rows. `Reflow` is the explicit complete-view
/// path for topology changes, filtering, and receiver generation gaps.
#[derive(Clone, Debug)]
pub(crate) enum SceneInspectionHierarchyFragment {
    Patch {
        message: SceneInspectionMessage,
        changed_rows: Vec<WorldInspectionHierarchyRow>,
    },
    Reflow {
        message: SceneInspectionMessage,
        entries: SceneEntries,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SceneInspectionHierarchyFragmentError {
    GenerationMismatch {
        message_generation: u64,
        entries_generation: Option<u64>,
    },
    PatchContainsStructuralRows,
    PatchRowMismatch {
        entity: u64,
    },
}

impl SceneInspectionHierarchyFragment {
    pub(crate) fn patch(
        message: SceneInspectionMessage,
        changed_rows: Vec<WorldInspectionHierarchyRow>,
    ) -> Result<Self, SceneInspectionHierarchyFragmentError> {
        if message.requires_resync()
            || message.requires_hierarchy_reflow()
            || !message.added_anchors().is_empty()
            || !message.removed_entities().is_empty()
        {
            return Err(SceneInspectionHierarchyFragmentError::PatchContainsStructuralRows);
        }
        let rows_by_entity = changed_rows
            .iter()
            .map(|row| (row.entity, row))
            .collect::<HashMap<_, _>>();
        let mut anchored_entities = HashSet::with_capacity(message.changed_anchors().len());
        let invalid_anchor_entity = message.changed_anchors().iter().find_map(|anchor| {
            if !anchored_entities.insert(anchor.entity()) {
                return Some(anchor.entity());
            }
            match rows_by_entity.get(&anchor.entity()) {
                Some(row)
                    if row.parent == anchor.parent()
                        && row.depth == anchor.depth()
                        && row.subtree_hash == anchor.subtree_hash() =>
                {
                    None
                }
                Some(_) | None => Some(anchor.entity()),
            }
        });
        let rows_match_anchors = message.changed_anchors().len() == changed_rows.len()
            && rows_by_entity.len() == changed_rows.len()
            && anchored_entities.len() == message.changed_anchors().len()
            && invalid_anchor_entity.is_none();
        if !rows_match_anchors {
            let entity = invalid_anchor_entity.unwrap_or_default();
            return Err(SceneInspectionHierarchyFragmentError::PatchRowMismatch { entity });
        }
        Ok(Self::Patch {
            message,
            changed_rows,
        })
    }

    pub(crate) fn reflow(
        message: SceneInspectionMessage,
        entries: SceneEntries,
    ) -> Result<Self, SceneInspectionHierarchyFragmentError> {
        if entries.inspection_generation() != Some(message.generation()) {
            return Err(SceneInspectionHierarchyFragmentError::GenerationMismatch {
                message_generation: message.generation(),
                entries_generation: entries.inspection_generation(),
            });
        }
        Ok(Self::Reflow { message, entries })
    }

    pub(crate) fn message(&self) -> &SceneInspectionMessage {
        match self {
            Self::Patch { message, .. } | Self::Reflow { message, .. } => message,
        }
    }

    pub(crate) fn changed_rows(&self) -> Option<&[WorldInspectionHierarchyRow]> {
        match self {
            Self::Patch { changed_rows, .. } => Some(changed_rows),
            Self::Reflow { .. } => None,
        }
    }

    pub(crate) fn reflow_entries(&self) -> Option<&SceneEntries> {
        match self {
            Self::Patch { .. } => None,
            Self::Reflow { entries, .. } => Some(entries),
        }
    }
}
