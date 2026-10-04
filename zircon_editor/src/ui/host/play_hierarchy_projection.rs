use std::collections::BTreeSet;
use std::sync::Arc;

use thiserror::Error;
use zircon_runtime::scene::{EntityId, WorldInspectionHierarchyRow};
use zircon_runtime_interface::world_sync::{WorldHierarchyRow, WorldQueryResult};
use zircon_runtime_interface::GatewaySessionIdentity;

use crate::core::editor_message::{
    SceneInspectionFieldsDelta, SceneInspectionHierarchyAnchor, SceneInspectionMessage,
    SceneInspectionSelectionDelta,
};
use crate::ui::workbench::snapshot::{SceneEntries, SceneInspectionHierarchyFragment};

#[derive(Debug, Error)]
pub(super) enum PlayHierarchyProjectionError {
    #[error("play hierarchy query returned a non-hierarchy projection")]
    UnexpectedProjection,
    #[error("play hierarchy returned NotModified before an identity-qualified base snapshot")]
    MissingBaseSnapshot,
    #[error(
        "play hierarchy NotModified generation {observed} does not match cached generation {expected}"
    )]
    NotModifiedGenerationMismatch { expected: u64, observed: u64 },
    #[error("play hierarchy generation regressed from {previous} to {observed}")]
    GenerationRegression { previous: u64, observed: u64 },
    #[error("play hierarchy changed rows without advancing generation {generation}")]
    SameGenerationMutation { generation: u64 },
    #[error("play hierarchy fragment is internally inconsistent: {0:?}")]
    InvalidFragment(crate::ui::workbench::snapshot::SceneInspectionHierarchyFragmentError),
}

impl From<crate::ui::workbench::snapshot::SceneInspectionHierarchyFragmentError>
    for PlayHierarchyProjectionError
{
    fn from(error: crate::ui::workbench::snapshot::SceneInspectionHierarchyFragmentError) -> Self {
        Self::InvalidFragment(error)
    }
}

#[derive(Clone, Debug)]
struct PlayHierarchySnapshot {
    identity: GatewaySessionIdentity,
    generation: u64,
    rows: Arc<[WorldInspectionHierarchyRow]>,
    selection_revision: u64,
    selected_entities: BTreeSet<EntityId>,
}

/// Identity-qualified retained projection of the runtime-owned Play hierarchy.
#[derive(Default)]
pub(super) struct PlayHierarchyProjection {
    snapshot: Option<PlayHierarchySnapshot>,
}

impl PlayHierarchyProjection {
    pub(super) fn generation_hint(&self, identity: &GatewaySessionIdentity) -> Option<u64> {
        self.snapshot
            .as_ref()
            .filter(|snapshot| &snapshot.identity == identity)
            .map(|snapshot| snapshot.generation)
    }

    pub(super) fn apply(
        &mut self,
        identity: GatewaySessionIdentity,
        result: WorldQueryResult,
        focused_entity: Option<EntityId>,
        selection_revision: u64,
        selected_entities: impl IntoIterator<Item = EntityId>,
        force_reflow: bool,
    ) -> Result<Option<SceneInspectionHierarchyFragment>, PlayHierarchyProjectionError> {
        let selected_entities = selected_entities.into_iter().collect::<BTreeSet<_>>();
        match result {
            WorldQueryResult::HierarchyRows { generation, rows } => self.apply_rows(
                identity,
                generation,
                rows,
                focused_entity,
                selection_revision,
                selected_entities,
                force_reflow,
            ),
            WorldQueryResult::NotModified { generation } => self.apply_not_modified(
                &identity,
                generation,
                focused_entity,
                selection_revision,
                selected_entities,
                force_reflow,
            ),
            WorldQueryResult::ComponentRows { .. }
            | WorldQueryResult::InspectionFields { .. }
            | WorldQueryResult::TransformSnapshot { .. }
            | WorldQueryResult::EntityMissing { .. } => {
                Err(PlayHierarchyProjectionError::UnexpectedProjection)
            }
        }
    }

    pub(super) fn reflow(
        &self,
        selection_revision: u64,
        selected_entities: impl IntoIterator<Item = EntityId>,
        focused_entity: Option<EntityId>,
    ) -> Option<SceneInspectionHierarchyFragment> {
        let snapshot = self.snapshot.as_ref()?;
        let selected_entities = selected_entities.into_iter().collect::<BTreeSet<_>>();
        Self::reflow_fragment(
            snapshot.generation,
            snapshot.rows.clone(),
            focused_entity,
            selection_revision,
            &selected_entities,
        )
        .ok()
    }

    pub(super) fn clear(&mut self) -> bool {
        self.snapshot.take().is_some()
    }

    pub(super) fn row(&self, entity: EntityId) -> Option<WorldInspectionHierarchyRow> {
        self.snapshot
            .as_ref()?
            .rows
            .iter()
            .find(|row| row.entity == entity)
            .cloned()
    }

    fn apply_rows(
        &mut self,
        identity: GatewaySessionIdentity,
        generation: u64,
        rows: Vec<WorldHierarchyRow>,
        focused_entity: Option<EntityId>,
        selection_revision: u64,
        selected_entities: BTreeSet<EntityId>,
        force_reflow: bool,
    ) -> Result<Option<SceneInspectionHierarchyFragment>, PlayHierarchyProjectionError> {
        let rows: Arc<[WorldInspectionHierarchyRow]> = rows.into();
        let previous = self
            .snapshot
            .as_ref()
            .filter(|snapshot| snapshot.identity == identity);
        if let Some(previous) = previous {
            if generation < previous.generation {
                return Err(PlayHierarchyProjectionError::GenerationRegression {
                    previous: previous.generation,
                    observed: generation,
                });
            }
            if generation == previous.generation && previous.rows.as_ref() != rows.as_ref() {
                return Err(PlayHierarchyProjectionError::SameGenerationMutation { generation });
            }
        }

        let must_reflow = force_reflow
            || previous.is_none()
            || previous.is_some_and(|previous| !same_topology(&previous.rows, &rows));
        if must_reflow {
            let fragment = Self::reflow_fragment(
                generation,
                rows.clone(),
                focused_entity,
                selection_revision,
                &selected_entities,
            )?;
            self.snapshot = Some(PlayHierarchySnapshot {
                identity,
                generation,
                rows,
                selection_revision,
                selected_entities,
            });
            return Ok(Some(fragment));
        }

        let previous = previous.expect("non-reflow projection must have a same-identity base");
        let mut changed_rows = Vec::with_capacity(rows.len());
        for (previous_row, current_row) in previous.rows.iter().zip(rows.iter()) {
            if previous_row != current_row {
                changed_rows.push(current_row.clone());
            }
        }
        let selection = selection_delta(previous, selection_revision, &selected_entities);
        let selection_changed = selection_revision != previous.selection_revision;
        if generation == previous.generation && changed_rows.is_empty() && !selection_changed {
            return Ok(None);
        }
        let mut changed_anchors = Vec::with_capacity(changed_rows.len());
        for row in &changed_rows {
            changed_anchors.push(hierarchy_anchor(row));
        }
        let message = SceneInspectionMessage::delta(
            previous.generation,
            generation,
            focused_entity,
            Vec::new(),
            changed_anchors,
            Vec::new(),
            false,
            SceneInspectionFieldsDelta::unchanged(focused_entity),
            selection,
        );
        let fragment = SceneInspectionHierarchyFragment::patch(message, changed_rows)?;
        self.snapshot = Some(PlayHierarchySnapshot {
            identity,
            generation,
            rows,
            selection_revision,
            selected_entities,
        });
        Ok(Some(fragment))
    }

    fn apply_not_modified(
        &mut self,
        identity: &GatewaySessionIdentity,
        generation: u64,
        focused_entity: Option<EntityId>,
        selection_revision: u64,
        selected_entities: BTreeSet<EntityId>,
        force_reflow: bool,
    ) -> Result<Option<SceneInspectionHierarchyFragment>, PlayHierarchyProjectionError> {
        let Some(previous) = self
            .snapshot
            .as_ref()
            .filter(|snapshot| &snapshot.identity == identity)
        else {
            return Err(PlayHierarchyProjectionError::MissingBaseSnapshot);
        };
        if previous.generation != generation {
            return Err(
                PlayHierarchyProjectionError::NotModifiedGenerationMismatch {
                    expected: previous.generation,
                    observed: generation,
                },
            );
        }
        if force_reflow {
            return Self::reflow_fragment(
                generation,
                previous.rows.clone(),
                focused_entity,
                selection_revision,
                &selected_entities,
            )
            .map(Some);
        }
        if selection_revision == previous.selection_revision {
            return Ok(None);
        }
        let message = SceneInspectionMessage::delta(
            generation,
            generation,
            focused_entity,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            false,
            SceneInspectionFieldsDelta::unchanged(focused_entity),
            selection_delta(previous, selection_revision, &selected_entities),
        );
        let fragment = SceneInspectionHierarchyFragment::patch(message, Vec::new())?;
        if let Some(snapshot) = self.snapshot.as_mut() {
            snapshot.selection_revision = selection_revision;
            snapshot.selected_entities = selected_entities;
        }
        Ok(Some(fragment))
    }

    fn reflow_fragment(
        generation: u64,
        rows: Arc<[WorldInspectionHierarchyRow]>,
        focused_entity: Option<EntityId>,
        selection_revision: u64,
        selected_entities: &BTreeSet<EntityId>,
    ) -> Result<SceneInspectionHierarchyFragment, PlayHierarchyProjectionError> {
        let focused_entity =
            focused_entity.filter(|entity| rows.iter().any(|row| row.entity == *entity));
        let entries = SceneEntries::from_hierarchy_rows_at_generation(
            rows.clone(),
            selected_entities
                .iter()
                .copied()
                .filter(|entity| rows.iter().any(|row| row.entity == *entity)),
            generation,
        );
        let message = SceneInspectionMessage::resync_with_selection_revision(
            generation,
            focused_entity,
            selection_revision,
        );
        Ok(SceneInspectionHierarchyFragment::reflow(message, entries)?)
    }
}

fn same_topology(
    previous: &[WorldInspectionHierarchyRow],
    current: &[WorldInspectionHierarchyRow],
) -> bool {
    previous.len() == current.len()
        && previous.iter().zip(current).all(|(previous, current)| {
            previous.entity == current.entity
                && previous.parent == current.parent
                && previous.depth == current.depth
        })
}

fn hierarchy_anchor(row: &WorldInspectionHierarchyRow) -> SceneInspectionHierarchyAnchor {
    SceneInspectionHierarchyAnchor::new(row.entity, row.parent, row.depth, row.subtree_hash)
}

fn selection_delta(
    previous: &PlayHierarchySnapshot,
    selection_revision: u64,
    selected_entities: &BTreeSet<EntityId>,
) -> SceneInspectionSelectionDelta {
    if previous.selection_revision == selection_revision {
        return SceneInspectionSelectionDelta::unchanged_at(selection_revision);
    }
    SceneInspectionSelectionDelta::between(
        previous.selection_revision,
        selection_revision,
        selected_entities
            .difference(&previous.selected_entities)
            .copied()
            .collect(),
        previous
            .selected_entities
            .difference(selected_entities)
            .copied()
            .collect(),
    )
}

#[cfg(test)]
#[path = "tests/play_hierarchy_projection_optimization_batch_tests.rs"]
mod optimization_batch_tests;

#[cfg(test)]
#[path = "tests/play_hierarchy_projection.rs"]
mod tests;
