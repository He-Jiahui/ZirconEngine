use std::collections::{BTreeMap, HashSet};

use super::transform_validation::validate_transform_for_write;
use super::{SceneError, SceneResult, World};
use crate::scene::EntityId;
use crate::scene::components::{
    ActiveSelf, Hierarchy, LocalTransform, Mobility, Name, NodeRecord, RenderLayerMask,
};

struct PreparedNodeRecordBatch {
    records: Vec<NodeRecord>,
    next_id: EntityId,
}

impl World {
    pub fn node_record(&self, entity: EntityId) -> Option<NodeRecord> {
        let name = self.names.get(&entity)?.0.clone();
        let kind = self.node_kind(entity)?;
        let parent = match self.hierarchy.get(&entity) {
            Some(hierarchy) => hierarchy.parent,
            None => None,
        };
        let transform = match self.local_transforms.get(&entity) {
            Some(local) => local.transform,
            None => LocalTransform::default().transform,
        };
        let active = match self.active_self.get(&entity) {
            Some(active) => active.0,
            None => ActiveSelf::default().0,
        };
        let render_layer_mask = match self.render_layer_masks.get(&entity) {
            Some(mask) => mask.0,
            None => RenderLayerMask::default().0,
        };
        let mobility = match self.mobility.get(&entity) {
            Some(mobility) => *mobility,
            None => Mobility::default(),
        };

        Some(NodeRecord {
            id: entity,
            name,
            kind,
            parent,
            transform,
            camera: self.cameras.get(&entity).cloned(),
            mesh: self.mesh_renderers.get(&entity).cloned(),
            sprite_2d: self.sprite_2d.get(&entity).cloned(),
            mesh_2d: self.mesh_2d.get(&entity).cloned(),
            ambient_light: self.ambient_lights.get(&entity).cloned(),
            directional_light: self.directional_lights.get(&entity).cloned(),
            point_light: self.point_lights.get(&entity).cloned(),
            rect_light: self.rect_lights.get(&entity).cloned(),
            spot_light: self.spot_lights.get(&entity).cloned(),
            active,
            render_layer_mask,
            mobility,
            rigid_body: self.rigid_bodies.get(&entity).cloned(),
            collider: self.colliders.get(&entity).cloned(),
            joint: self.joints.get(&entity).cloned(),
            animation_skeleton: self.animation_skeletons.get(&entity).cloned(),
            animation_player: self.animation_players.get(&entity).cloned(),
            animation_sequence_player: self.animation_sequence_players.get(&entity).cloned(),
            animation_graph_player: self.animation_graph_players.get(&entity).cloned(),
            animation_state_machine_player: self
                .animation_state_machine_players
                .get(&entity)
                .cloned(),
        })
    }

    pub fn insert_node_record(&mut self, record: NodeRecord) -> SceneResult<()> {
        self.insert_owned_node_records(vec![record])
    }

    fn prepare_owned_node_record_batch(
        &self,
        records: Vec<NodeRecord>,
    ) -> SceneResult<PreparedNodeRecordBatch> {
        let (_, next_id) = self.validate_node_record_batch_input(&records)?;
        Ok(PreparedNodeRecordBatch { records, next_id })
    }

    pub(in crate::scene) fn validate_owned_node_records(
        &self,
        records: &[NodeRecord],
    ) -> SceneResult<()> {
        let _ = self.validate_node_record_batch_input(records)?;
        Ok(())
    }

    fn validate_node_record_batch_input<'a>(
        &self,
        records: &'a [NodeRecord],
    ) -> SceneResult<(BTreeMap<EntityId, &'a NodeRecord>, EntityId)> {
        let mut records_by_id = BTreeMap::new();
        let mut next_id = self.next_id;
        for record in &records {
            if self.contains_entity(record.id)
                || self.entity_registry.contains_stable(record.id)
                || records_by_id.insert(record.id, record).is_some()
            {
                return Err(SceneError::DuplicateEntity { entity: record.id });
            }
            validate_transform_for_write(record.id, record.transform)?;
            let candidate_next_id = record
                .id
                .checked_add(1)
                .ok_or(SceneError::EntityIdExhausted { entity: record.id })?;
            next_id = next_id.max(candidate_next_id);
        }

        self.validate_node_record_batch_mobility(&records_by_id)?;
        Ok((records_by_id, next_id))
    }

    fn commit_node_record_batch(&mut self, batch: PreparedNodeRecordBatch) -> SceneResult<()> {
        if batch.records.is_empty() {
            return Ok(());
        }

        let PreparedNodeRecordBatch { records, next_id } = batch;
        let inserted_entities = records.iter().map(|record| record.id).collect::<Vec<_>>();
        let prior_staging = std::mem::replace(&mut self.record_staged_lifecycle_events, true);
        let lifecycle_start = self.staged_lifecycle_events.len();
        for record in records {
            if let Err(error) = self.insert_prevalidated_node_record(record) {
                self.staged_lifecycle_events.truncate(lifecycle_start);
                self.record_staged_lifecycle_events = prior_staging;
                return Err(error);
            }
        }
        self.next_id = self.next_id.max(next_id);
        self.mark_derived_state_dirty();
        self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        self.advance_world_generation();
        for entity in inserted_entities {
            self.advance_scene_binding_generations_for_new_descendant(entity);
        }
        self.record_staged_lifecycle_events = prior_staging;

        if prior_staging {
            return Ok(());
        }
        let lifecycle_events = self.staged_lifecycle_events.split_off(lifecycle_start);
        for event in lifecycle_events {
            self.dispatch_component_lifecycle(event);
        }
        Ok(())
    }

    fn insert_prevalidated_node_record(&mut self, record: NodeRecord) -> SceneResult<()> {
        self.register_stable_entity(record.id)?;
        self.entities.push(record.id);
        self.kinds.insert(record.id, record.kind);
        self.record_node_kind_added(record.kind);
        self.names.insert(record.id, Name(record.name));
        self.hierarchy.insert(
            record.id,
            Hierarchy {
                parent: record.parent,
            },
        );
        self.local_transforms.insert(
            record.id,
            LocalTransform {
                transform: record.transform,
            },
        );
        self.active_self
            .insert(record.id, ActiveSelf(record.active));
        self.render_layer_masks
            .insert(record.id, RenderLayerMask(record.render_layer_mask));
        self.mobility.insert(record.id, record.mobility);

        if let Some(camera) = record.camera {
            self.cameras.insert(record.id, camera);
            if self.active_camera == 0 || !self.cameras.contains_key(&self.active_camera) {
                self.active_camera = record.id;
            }
        }
        if let Some(mesh) = record.mesh {
            self.mesh_renderers.insert(record.id, mesh);
        }
        if let Some(sprite_2d) = record.sprite_2d {
            self.sprite_2d.insert(record.id, sprite_2d);
        }
        if let Some(mesh_2d) = record.mesh_2d {
            self.mesh_2d.insert(record.id, mesh_2d);
        }
        if let Some(ambient_light) = record.ambient_light {
            self.ambient_lights.insert(record.id, ambient_light);
        }
        if let Some(directional_light) = record.directional_light {
            self.directional_lights.insert(record.id, directional_light);
        }
        if let Some(point_light) = record.point_light {
            self.point_lights.insert(record.id, point_light);
        }
        if let Some(rect_light) = record.rect_light {
            self.rect_lights.insert(record.id, rect_light);
        }
        if let Some(spot_light) = record.spot_light {
            self.spot_lights.insert(record.id, spot_light);
        }
        if let Some(rigid_body) = record.rigid_body {
            self.rigid_bodies.insert(record.id, rigid_body);
        }
        if let Some(collider) = record.collider {
            self.colliders.insert(record.id, collider);
        }
        if let Some(joint) = record.joint {
            self.joints.insert(record.id, joint);
        }
        if let Some(animation_skeleton) = record.animation_skeleton {
            self.animation_skeletons
                .insert(record.id, animation_skeleton);
        }
        if let Some(animation_player) = record.animation_player {
            self.animation_players.insert(record.id, animation_player);
        }
        if let Some(animation_sequence_player) = record.animation_sequence_player {
            self.animation_sequence_players
                .insert(record.id, animation_sequence_player);
        }
        if let Some(animation_graph_player) = record.animation_graph_player {
            self.animation_graph_players
                .insert(record.id, animation_graph_player);
        }
        if let Some(animation_state_machine_player) = record.animation_state_machine_player {
            self.animation_state_machine_players
                .insert(record.id, animation_state_machine_player);
        }

        self.rebuild_fixed_component_presence_for_entity(record.id);
        Ok(())
    }

    fn validate_node_record_batch_mobility(
        &self,
        records_by_id: &BTreeMap<EntityId, &NodeRecord>,
    ) -> SceneResult<()> {
        let mut existing_static_child_parents = HashSet::new();
        for child in self.entities.iter().copied() {
            if self.mobility(child) != Some(Mobility::Static) {
                continue;
            }
            if let Some(parent) = self.parent_of(child) {
                existing_static_child_parents.insert(parent);
            }
        }
        let mut incoming_static_child_parents = HashSet::new();
        for record in records_by_id.values().copied() {
            if record.mobility == Mobility::Static {
                if let Some(parent) = record.parent {
                    incoming_static_child_parents.insert(parent);
                }
            }
        }

        for record in records_by_id.values().copied() {
            if record.mobility == Mobility::Static {
                let Some(parent) = record.parent else {
                    continue;
                };
                let parent_mobility = match records_by_id.get(&parent) {
                    Some(parent) => parent.mobility,
                    None => self.mobility(parent).unwrap_or_default(),
                };
                if parent_mobility == Mobility::Dynamic {
                    return Err(SceneError::StaticMobilityUnderDynamicParent {
                        entity: record.id,
                        parent,
                    });
                }
            }

            if record.mobility != Mobility::Dynamic {
                continue;
            }
            if existing_static_child_parents.contains(&record.id)
                || incoming_static_child_parents.contains(&record.id)
            {
                return Err(SceneError::DynamicMobilityWithStaticChildren { entity: record.id });
            }
        }
        Ok(())
    }

    pub fn insert_node_records(&mut self, records: &[NodeRecord]) -> SceneResult<()> {
        self.insert_owned_node_records(records.to_vec())
    }

    /// Validates then commits an owned node-record batch as one visible world mutation.
    ///
    /// Callers that already own undo, import, or dynamic-scene payloads should use
    /// this entry point to avoid copying the affected records before publication.
    pub fn insert_owned_node_records(&mut self, records: Vec<NodeRecord>) -> SceneResult<()> {
        let batch = self.prepare_owned_node_record_batch(records)?;
        self.commit_node_record_batch(batch)
    }

    pub fn rename_node(&mut self, entity: EntityId, name: impl Into<String>) -> SceneResult<bool> {
        let name = name.into();
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(SceneError::EmptyNodeName);
        }
        let Some(current) = self.names.get(&entity) else {
            if !self.contains_entity(entity) {
                return Err(SceneError::missing_entity("rename", entity));
            }
            return Err(SceneError::MissingRequiredComponent {
                operation: "rename",
                entity,
                component: "Name",
            });
        };
        if current.0 == trimmed {
            return Ok(false);
        }
        self.insert(entity, Name(trimmed.to_string()))?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::scene::{
        NodeKind, World,
        components::{Mobility, Name},
        ecs::LifecycleEventKind,
    };

    fn node_record_with_id(id: u64) -> crate::scene::components::NodeRecord {
        let mut source = World::empty();
        let entity = source.spawn_node(NodeKind::Empty);
        let mut record = source
            .node_record(entity)
            .expect("spawned node must produce a record");
        record.id = id;
        record
    }

    #[test]
    fn node_record_batch_publishes_once_after_all_records_are_visible() {
        let mut world = World::empty();
        let mut first = node_record_with_id(41);
        first.name = "First".to_string();
        let mut second = node_record_with_id(42);
        second.name = "Second".to_string();
        second.parent = Some(first.id);

        let observed = Arc::new(Mutex::new(Vec::new()));
        let observed_for_callback = Arc::clone(&observed);
        let second_id = second.id;
        world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |world, event| {
            observed_for_callback
                .lock()
                .expect("test observer lock")
                .push((event.entity(), world.contains_entity(second_id)));
        });

        let generation_before = world.world_generation();
        world
            .insert_node_records(&[first.clone(), second.clone()])
            .expect("validated batch must commit");

        assert_eq!(world.world_generation(), generation_before + 1);
        assert_eq!(world.node_record(first.id), Some(first));
        assert_eq!(world.node_record(second.id), Some(second));
        let observed = observed.lock().expect("test observer lock");
        assert_eq!(observed.len(), 2);
        assert!(observed.iter().all(|(_, second_visible)| *second_visible));
    }

    #[test]
    fn duplicate_node_record_batch_leaves_world_unchanged() {
        let mut world = World::empty();
        let record = node_record_with_id(41);
        let generation_before = world.world_generation();

        let error = world
            .insert_node_records(&[record.clone(), record])
            .expect_err("duplicate batch identities must fail before commit");

        assert!(matches!(
            error,
            crate::scene::SceneError::DuplicateEntity { .. }
        ));
        assert_eq!(world.world_generation(), generation_before);
        assert!(world.node_records().is_empty());
    }

    #[test]
    fn stale_registry_identity_rejects_the_entire_node_record_batch_before_commit() {
        let mut world = World::empty();
        world
            .register_stable_entity(42)
            .expect("test fixture should create a registry-only identity");
        let first = node_record_with_id(41);
        let second = node_record_with_id(42);
        let generation_before = world.world_generation();

        let error = world
            .insert_owned_node_records(vec![first, second])
            .expect_err("registry identity must be rejected during batch prevalidation");

        assert!(matches!(
            error,
            crate::scene::SceneError::DuplicateEntity { entity: 42 }
        ));
        assert_eq!(world.world_generation(), generation_before);
        assert!(world.node_records().is_empty());
    }

    #[test]
    fn dynamic_parent_with_static_incoming_child_rejects_the_whole_batch() {
        let mut world = World::empty();
        let mut parent = node_record_with_id(41);
        parent.mobility = Mobility::Dynamic;
        let mut child = node_record_with_id(42);
        child.parent = Some(parent.id);
        let generation_before = world.world_generation();

        let error = world
            .insert_owned_node_records(vec![parent, child])
            .expect_err("dynamic parents cannot gain static children in a batch");

        assert!(matches!(
            error,
            crate::scene::SceneError::DynamicMobilityWithStaticChildren { entity: 41 }
        ));
        assert_eq!(world.world_generation(), generation_before);
        assert!(world.node_records().is_empty());
    }
}
