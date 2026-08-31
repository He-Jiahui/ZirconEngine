use std::collections::BTreeSet;
use std::sync::Arc;

use zircon_runtime::scene::{
    EntityId, Scene, WorldInspectionArtifact, WorldInspectionFieldsArtifact,
};

use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, SceneInspectionFieldsDelta,
    SceneInspectionHierarchyAnchor, SceneInspectionMessage, SceneInspectionPropertyPath,
    SceneInspectionSelectionDelta, TOPIC_SCENE_INSPECTION,
};
use crate::core::play::WorldDomain;
use crate::ui::workbench::snapshot::{SceneEntries, SceneInspectionHierarchyFragment};

use super::EditorHostEventController;

#[derive(Default)]
pub(super) struct SceneInspectionPublication {
    previous: Option<PublishedInspection>,
}

struct PublishedInspection {
    artifact: Arc<WorldInspectionArtifact>,
    focused_entity: Option<EntityId>,
    focused_fields: Option<Arc<WorldInspectionFieldsArtifact>>,
    selection_revision: u64,
    selected_entities: Arc<BTreeSet<EntityId>>,
}

impl SceneInspectionPublication {
    fn reset(
        &mut self,
        scene: &Scene,
        focused_entity: Option<EntityId>,
        selection_revision: u64,
        selected_entities: impl Iterator<Item = EntityId>,
    ) -> SceneInspectionMessage {
        let artifact = scene.inspection_artifact();
        let focused_entity = focused_entity.filter(|entity| scene.contains_entity(*entity));
        let selected_entities = Arc::new(
            selected_entities
                .into_iter()
                .filter(|entity| scene.contains_entity(*entity))
                .collect(),
        );
        let focused_fields =
            focused_entity.and_then(|entity| scene.inspection_fields_artifact(entity));
        self.previous = Some(PublishedInspection {
            artifact: artifact.clone(),
            focused_entity,
            focused_fields,
            selection_revision,
            selected_entities,
        });
        SceneInspectionMessage::resync_with_selection_revision(
            artifact.generation(),
            focused_entity,
            selection_revision,
        )
    }

    fn observe(
        &mut self,
        scene: &Scene,
        focused_entity: Option<EntityId>,
        selection_revision: u64,
        selected_entities: impl Iterator<Item = EntityId>,
    ) -> Option<SceneInspectionMessage> {
        let artifact = scene.inspection_artifact();
        let focused_entity = focused_entity.filter(|entity| scene.contains_entity(*entity));
        let selected_entities = self
            .previous
            .as_ref()
            .filter(|previous| previous.selection_revision == selection_revision)
            .map(|previous| previous.selected_entities.clone())
            .unwrap_or_else(|| {
                Arc::new(
                    selected_entities
                        .filter(|entity| scene.contains_entity(*entity))
                        .collect(),
                )
            });
        let focused_fields =
            focused_entity.and_then(|entity| scene.inspection_fields_artifact(entity));
        let current = PublishedInspection {
            artifact,
            focused_entity,
            focused_fields,
            selection_revision,
            selected_entities,
        };
        let Some(previous) = self.previous.replace(current) else {
            return None;
        };
        let current = self
            .previous
            .as_ref()
            .expect("inspection publication was just stored");
        let hierarchy_changed = !Arc::ptr_eq(&previous.artifact, &current.artifact);
        let focus_changed = previous.focused_entity != current.focused_entity;
        let selection_changed = previous.selection_revision != current.selection_revision;
        let fields_changed = !same_optional_arc(&previous.focused_fields, &current.focused_fields);
        if !hierarchy_changed && !focus_changed && !selection_changed && !fields_changed {
            return None;
        }

        let (added_anchors, changed_anchors, removed_entities, hierarchy_reflow_required) =
            if hierarchy_changed {
                let delta = current
                    .artifact
                    .published_delta_from(previous.artifact.generation())
                    .cloned()
                    .unwrap_or_else(|| current.artifact.delta_from(&previous.artifact));
                (
                    delta
                        .added_rows()
                        .iter()
                        .map(scene_inspection_anchor_from_row)
                        .collect(),
                    delta
                        .changed_rows()
                        .iter()
                        .map(scene_inspection_anchor_from_row)
                        .collect(),
                    delta.removed_entities().to_vec(),
                    delta.requires_hierarchy_reflow(),
                )
            } else {
                (Vec::new(), Vec::new(), Vec::new(), false)
            };
        let previous_generation = previous.artifact.generation();
        let selection = if selection_changed {
            SceneInspectionSelectionDelta::between(
                previous.selection_revision,
                current.selection_revision,
                current
                    .selected_entities
                    .as_ref()
                    .difference(previous.selected_entities.as_ref())
                    .copied()
                    .collect(),
                previous
                    .selected_entities
                    .as_ref()
                    .difference(current.selected_entities.as_ref())
                    .copied()
                    .collect(),
            )
        } else {
            SceneInspectionSelectionDelta::unchanged_at(current.selection_revision)
        };
        let focused_fields = focused_fields_delta(previous.focused_fields, current, focus_changed);
        Some(SceneInspectionMessage::delta(
            previous_generation,
            current.artifact.generation(),
            current.focused_entity,
            added_anchors,
            changed_anchors,
            removed_entities,
            hierarchy_reflow_required,
            focused_fields,
            selection,
        ))
    }
}

fn scene_inspection_anchor_from_row(
    row: &zircon_runtime::scene::WorldInspectionHierarchyRow,
) -> SceneInspectionHierarchyAnchor {
    SceneInspectionHierarchyAnchor::new(row.entity, row.parent, row.depth, row.subtree_hash)
}

impl EditorHostEventController {
    pub(super) fn seed_scene_inspection_publication(&self) {
        self.observe_scene_inspection_publication();
    }

    pub(super) fn publish_scene_inspection_publication(&self) {
        if matches!(self.active_hierarchy_world_domain(), WorldDomain::Play(_)) {
            return;
        }
        let Some(message) = self.observe_scene_inspection_publication() else {
            return;
        };
        let topic = EditorTopic::parse(TOPIC_SCENE_INSPECTION)
            .expect("scene inspection topic must remain a valid editor topic");
        self.context().bus().publish(
            topic,
            EditorMessage::new(EditorMessagePayload::SceneInspection(message)),
        );
    }

    pub(crate) fn publish_scene_inspection_resync(&self) {
        if matches!(self.active_hierarchy_world_domain(), WorldDomain::Play(_)) {
            return;
        }
        let mut publication = self
            .scene_inspection_publication
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let shell = self.shell().lock();
        let selection = shell.state.viewport_controller.selection();
        let focused_entity = selection.active_primary();
        let selection_revision = selection.revision();
        let message = match shell.state.world.with_world(|scene| {
            publication.reset(
                scene,
                focused_entity,
                selection_revision,
                selection.active_items().iter().copied(),
            )
        }) {
            Ok(message) => message,
            Err(error) => {
                shell
                    .state
                    .report_authoring_world_access_failure("scene inspection resync", &error);
                None
            }
        };
        drop(shell);
        drop(publication);
        let Some(message) = message else {
            return;
        };
        let topic = EditorTopic::parse(TOPIC_SCENE_INSPECTION)
            .expect("scene inspection topic must remain a valid editor topic");
        self.context().bus().publish(
            topic,
            EditorMessage::new(EditorMessagePayload::SceneInspection(message)),
        );
    }

    fn observe_scene_inspection_publication(&self) -> Option<SceneInspectionMessage> {
        if matches!(self.active_hierarchy_world_domain(), WorldDomain::Play(_)) {
            return None;
        }
        let mut publication = self
            .scene_inspection_publication
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let shell = self.shell().lock();
        let selection = shell.state.viewport_controller.selection();
        let focused_entity = selection.active_primary();
        let selection_revision = selection.revision();
        match shell.state.world.with_world(|scene| {
            publication.observe(
                scene,
                focused_entity,
                selection_revision,
                selection.active_items().iter().copied(),
            )
        }) {
            Ok(message) => message.flatten(),
            Err(error) => {
                shell
                    .state
                    .report_authoring_world_access_failure("scene inspection publication", &error);
                None
            }
        }
    }

    /// Delivers the newest hierarchy publication to the retained surface once per host frame.
    pub(crate) fn take_retained_scene_inspection_message(&self) -> Option<SceneInspectionMessage> {
        self.context()
            .bus()
            .drain_deliveries(self.retained_scene_inspection_subscriber)
            .into_iter()
            .filter_map(|delivery| match delivery.message().payload() {
                EditorMessagePayload::SceneInspection(message) => Some(message.clone()),
                _ => None,
            })
            .last()
    }

    /// Resolves a hierarchy publication into either a sparse patch or explicit reflow.
    pub(crate) fn scene_inspection_hierarchy_fragment(
        &self,
        message: SceneInspectionMessage,
    ) -> Option<SceneInspectionHierarchyFragment> {
        if matches!(self.active_hierarchy_world_domain(), WorldDomain::Play(_)) {
            return self.play_scene_inspection_hierarchy_reflow();
        }
        zircon_runtime::profile_scope!(
            "editor",
            "scene_inspection",
            "hierarchy_fragment_projection"
        );
        let shell = self.shell().lock();
        let selection = shell.state.viewport_controller.selection();
        let focused_entity = selection.active_primary();
        let selection_revision = selection.revision();
        match shell.state.world.with_world(|scene| {
            let artifact = scene.inspection_artifact();
            let message = if artifact.generation() == message.generation() {
                message
            } else {
                SceneInspectionMessage::resync_with_selection_revision(
                    artifact.generation(),
                    focused_entity,
                    selection_revision,
                )
            };
            if message.requires_resync()
                || message.requires_hierarchy_reflow()
                || !message.added_anchors().is_empty()
                || !message.removed_entities().is_empty()
            {
                let selected = selection
                    .active_items()
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>();
                return scene_inspection_reflow_fragment(
                    &artifact,
                    message.with_selection_resync_at(selection_revision),
                    &selected,
                )
                .ok();
            }

            let changed_rows = message
                .changed_anchors()
                .iter()
                .map(|anchor| artifact.hierarchy_row(anchor.entity()).cloned())
                .collect::<Option<Vec<_>>>()?;
            SceneInspectionHierarchyFragment::patch(message, changed_rows).ok()
        }) {
            Ok(fragment) => fragment.flatten(),
            Err(error) => {
                shell.state.report_authoring_world_access_failure(
                    "scene inspection hierarchy fragment",
                    &error,
                );
                None
            }
        }
    }

    /// Resolves a complete hierarchy only after an explicit reflow request from the retained
    /// consumer (filtering, structural change, or a missed generation).
    pub(crate) fn scene_inspection_hierarchy_reflow(
        &self,
        message: SceneInspectionMessage,
    ) -> Option<SceneInspectionHierarchyFragment> {
        if matches!(self.active_hierarchy_world_domain(), WorldDomain::Play(_)) {
            return self.play_scene_inspection_hierarchy_reflow();
        }
        let shell = self.shell().lock();
        let selection = shell.state.viewport_controller.selection();
        let focused_entity = selection.active_primary();
        let selection_revision = selection.revision();
        let selected = selection
            .active_items()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        match shell.state.world.with_world(|scene| {
            let artifact = scene.inspection_artifact();
            let message = if artifact.generation() == message.generation() {
                message
            } else {
                SceneInspectionMessage::resync_with_selection_revision(
                    artifact.generation(),
                    focused_entity,
                    selection_revision,
                )
            };
            scene_inspection_reflow_fragment(
                &artifact,
                message.with_selection_resync_at(selection_revision),
                &selected,
            )
            .ok()
        }) {
            Ok(fragment) => fragment.flatten(),
            Err(error) => {
                shell.state.report_authoring_world_access_failure(
                    "scene inspection hierarchy reflow",
                    &error,
                );
                None
            }
        }
    }

    /// Returns the authoritative selection only when the retained projection reports a revision
    /// gap. Normal sparse hierarchy patches never materialize this snapshot.
    pub(crate) fn scene_inspection_selection_snapshot(&self) -> (u64, Vec<EntityId>) {
        let shell = self.shell().lock();
        let selection = shell.state.viewport_controller.selection();
        (
            selection.revision(),
            selection.active_items().iter().copied().collect(),
        )
    }

    /// Resolves one authoritative hierarchy row without materializing the complete sparse view.
    pub(crate) fn scene_inspection_hierarchy_row(
        &self,
        entity: EntityId,
    ) -> Option<zircon_runtime::scene::WorldInspectionHierarchyRow> {
        if matches!(self.active_hierarchy_world_domain(), WorldDomain::Play(_)) {
            return self
                .play_hierarchy_projection
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .row(entity);
        }
        let shell = self.shell().lock();
        match shell
            .state
            .world
            .with_world(|scene| scene.inspection_artifact().hierarchy_row(entity).cloned())
        {
            Ok(row) => row.flatten(),
            Err(error) => {
                shell
                    .state
                    .report_authoring_world_access_failure("scene inspection row", &error);
                None
            }
        }
    }

    fn play_scene_inspection_hierarchy_reflow(&self) -> Option<SceneInspectionHierarchyFragment> {
        let (focused_entity, selection_revision, selected_entities) = {
            let shell = self.shell().lock();
            let selection = shell.state.viewport_controller.selection();
            (
                selection.active_primary(),
                selection.revision(),
                selection.active_items().iter().copied().collect::<Vec<_>>(),
            )
        };
        self.play_hierarchy_projection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reflow(selection_revision, selected_entities, focused_entity)
    }
}

fn scene_inspection_reflow_fragment(
    artifact: &WorldInspectionArtifact,
    message: SceneInspectionMessage,
    selected: &BTreeSet<EntityId>,
) -> Result<
    SceneInspectionHierarchyFragment,
    crate::ui::workbench::snapshot::SceneInspectionHierarchyFragmentError,
> {
    let entries = SceneEntries::from_artifact(
        artifact,
        selected
            .iter()
            .copied()
            .filter(|entity| artifact.hierarchy_row(*entity).is_some()),
    );
    SceneInspectionHierarchyFragment::reflow(message, entries)
}

fn focused_fields_delta(
    previous_fields: Option<Arc<WorldInspectionFieldsArtifact>>,
    current: &PublishedInspection,
    focus_changed: bool,
) -> SceneInspectionFieldsDelta {
    if focus_changed {
        return SceneInspectionFieldsDelta::resync(current.focused_entity);
    }
    match (&previous_fields, &current.focused_fields) {
        (Some(previous), Some(current)) if !Arc::ptr_eq(previous, current) => {
            let delta = current.delta_from(previous);
            SceneInspectionFieldsDelta::delta(
                current.entity(),
                delta
                    .changed_fields()
                    .iter()
                    .map(property_path_from_field)
                    .collect(),
                delta
                    .removed_fields()
                    .iter()
                    .map(|field| {
                        SceneInspectionPropertyPath::new(
                            field.component_type_path(),
                            field.field_name(),
                        )
                    })
                    .collect(),
            )
        }
        (Some(_), None) | (None, Some(_)) => {
            SceneInspectionFieldsDelta::resync(current.focused_entity)
        }
        (None, None) | (Some(_), Some(_)) => {
            SceneInspectionFieldsDelta::unchanged(current.focused_entity)
        }
    }
}

fn property_path_from_field(
    field: &zircon_runtime::scene::WorldInspectionField,
) -> SceneInspectionPropertyPath {
    SceneInspectionPropertyPath::new(&field.component_type_path, &field.field_name)
}

fn same_optional_arc<T>(left: &Option<Arc<T>>, right: &Option<Arc<T>>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        (None, None) => true,
        (None, Some(_)) | (Some(_), None) => false,
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::scene::components::NodeKind;
    use zircon_runtime::scene::{EntityId, Scene};

    use super::SceneInspectionPublication;

    #[test]
    fn stable_large_selection_rename_reuses_the_published_selection_snapshot() {
        const SELECTED_ENTITY_COUNT: usize = 10_000;
        const SELECTION_REVISION: u64 = 42;

        let mut scene = Scene::new();
        let selected_entities = (0..SELECTED_ENTITY_COUNT)
            .map(|_| {
                scene
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed")
            })
            .collect::<Vec<_>>();
        let renamed_entity = selected_entities[0];
        let mut publication = SceneInspectionPublication::default();
        publication.reset(
            &scene,
            Some(renamed_entity),
            SELECTION_REVISION,
            selected_entities.into_iter(),
        );
        scene
            .rename_node(renamed_entity, "Renamed selected scene item")
            .expect("selected entity should remain available for rename");

        let message = publication
            .observe(
                &scene,
                Some(renamed_entity),
                SELECTION_REVISION,
                std::iter::from_fn(|| -> Option<EntityId> {
                    panic!("stable selection must reuse the published Arc instead of collecting")
                }),
            )
            .expect("renaming a selected node should publish a sparse hierarchy patch");

        assert_eq!(message.changed_anchors().len(), 1);
        assert_eq!(
            message.selection().previous_revision(),
            Some(SELECTION_REVISION)
        );
        assert_eq!(message.selection().revision(), SELECTION_REVISION);
        assert!(message.selection().added_entities().is_empty());
        assert!(message.selection().removed_entities().is_empty());
    }
}
