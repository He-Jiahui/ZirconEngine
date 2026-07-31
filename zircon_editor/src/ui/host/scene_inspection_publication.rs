use std::sync::Arc;

use zircon_runtime::scene::{
    EntityId, Scene, WorldInspectionArtifact, WorldInspectionFieldsArtifact,
};

use crate::core::editor_message::{
    EditorMessage, EditorMessagePayload, EditorTopic, SceneInspectionFieldsDelta,
    SceneInspectionMessage, SceneInspectionPropertyPath, TOPIC_SCENE_INSPECTION,
};

use super::EditorHostEventController;

#[derive(Default)]
pub(super) struct SceneInspectionPublication {
    previous: Option<PublishedInspection>,
}

struct PublishedInspection {
    artifact: Arc<WorldInspectionArtifact>,
    focused_entity: Option<EntityId>,
    focused_fields: Option<Arc<WorldInspectionFieldsArtifact>>,
}

impl SceneInspectionPublication {
    fn reset(&mut self, scene: &Scene, focused_entity: Option<EntityId>) -> SceneInspectionMessage {
        let artifact = scene.inspection_artifact();
        let focused_entity = focused_entity.filter(|entity| scene.contains_entity(*entity));
        let focused_fields =
            focused_entity.and_then(|entity| scene.inspection_fields_artifact(entity));
        self.previous = Some(PublishedInspection {
            artifact: artifact.clone(),
            focused_entity,
            focused_fields,
        });
        SceneInspectionMessage::resync(artifact.generation(), focused_entity)
    }

    fn observe(
        &mut self,
        scene: &Scene,
        focused_entity: Option<EntityId>,
    ) -> Option<SceneInspectionMessage> {
        let artifact = scene.inspection_artifact();
        let focused_entity = focused_entity.filter(|entity| scene.contains_entity(*entity));
        let focused_fields =
            focused_entity.and_then(|entity| scene.inspection_fields_artifact(entity));
        let current = PublishedInspection {
            artifact,
            focused_entity,
            focused_fields,
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
        let fields_changed = !same_optional_arc(&previous.focused_fields, &current.focused_fields);
        if !hierarchy_changed && !focus_changed && !fields_changed {
            return None;
        }

        let (added_entities, changed_entities, removed_entities) = if hierarchy_changed {
            let delta = current
                .artifact
                .published_delta_from(previous.artifact.generation())
                .cloned()
                .unwrap_or_else(|| current.artifact.delta_from(&previous.artifact));
            (
                delta.added_rows().iter().map(|row| row.entity).collect(),
                delta.changed_rows().iter().map(|row| row.entity).collect(),
                delta.removed_entities().to_vec(),
            )
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };
        let previous_generation = previous.artifact.generation();
        let focused_fields = focused_fields_delta(previous, current, focus_changed);
        Some(SceneInspectionMessage::delta(
            previous_generation,
            current.artifact.generation(),
            current.focused_entity,
            added_entities,
            changed_entities,
            removed_entities,
            focused_fields,
        ))
    }
}

impl EditorHostEventController {
    pub(super) fn seed_scene_inspection_publication(&self) {
        self.observe_scene_inspection_publication();
    }

    pub(super) fn publish_scene_inspection_publication(&self) {
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

    pub(super) fn publish_scene_inspection_resync(&self) {
        let mut publication = self
            .scene_inspection_publication
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let shell = self.shell().lock();
        let focused_entity = shell.state.viewport_controller.selection().active_primary();
        let message = shell
            .state
            .world
            .try_with_world(|scene| publication.reset(scene, focused_entity));
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
        let mut publication = self
            .scene_inspection_publication
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let shell = self.shell().lock();
        let focused_entity = shell.state.viewport_controller.selection().active_primary();
        shell
            .state
            .world
            .try_with_world(|scene| publication.observe(scene, focused_entity))
            .flatten()
    }
}

fn focused_fields_delta(
    previous: PublishedInspection,
    current: &PublishedInspection,
    focus_changed: bool,
) -> SceneInspectionFieldsDelta {
    if focus_changed {
        return SceneInspectionFieldsDelta::resync(current.focused_entity);
    }
    match (&previous.focused_fields, &current.focused_fields) {
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
