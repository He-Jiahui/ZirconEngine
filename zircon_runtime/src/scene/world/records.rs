use std::collections::{BTreeMap, HashSet};

use super::transform_validation::validate_transform_for_write;
use super::{SceneError, SceneResult, World};
use crate::scene::components::{
    ActiveSelf, AmbientLight, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, Mesh2dComponent, MeshRenderer, Mobility, Name,
    NodeRecord, PointLight, RectLight, RenderLayerMask, RigidBodyComponent, SpotLight,
    Sprite2dComponent,
};
use crate::scene::{EntityId, ecs::InternalEntity};
use zircon_runtime_interface::world_sync::WorldFact;

struct PreparedNodeRecordBatch {
    records: Vec<NodeRecord>,
    next_id: EntityId,
}

impl World {
    pub fn node_record(&self, entity: EntityId) -> Option<NodeRecord> {
        let name = self.get::<Name>(entity)?.0.clone();
        let kind = self.node_kind(entity)?;
        let parent = match self.get::<Hierarchy>(entity) {
            Some(hierarchy) => hierarchy.parent,
            None => None,
        };
        let transform = match self.get::<LocalTransform>(entity) {
            Some(local) => local.transform,
            None => LocalTransform::default().transform,
        };
        let active = match self.get::<ActiveSelf>(entity) {
            Some(active) => active.0,
            None => ActiveSelf::default().0,
        };
        let render_layer_mask = match self.get::<RenderLayerMask>(entity) {
            Some(mask) => mask.0,
            None => RenderLayerMask::default().0,
        };
        let mobility = match self.get::<Mobility>(entity) {
            Some(mobility) => *mobility,
            None => Mobility::default(),
        };

        Some(NodeRecord {
            id: entity,
            name,
            kind,
            parent,
            transform,
            camera: self.get::<CameraComponent>(entity).cloned(),
            mesh: self.get::<MeshRenderer>(entity).cloned(),
            sprite_2d: self.get::<Sprite2dComponent>(entity).cloned(),
            mesh_2d: self.get::<Mesh2dComponent>(entity).cloned(),
            ambient_light: self.get::<AmbientLight>(entity).cloned(),
            directional_light: self.get::<DirectionalLight>(entity).cloned(),
            point_light: self.get::<PointLight>(entity).cloned(),
            rect_light: self.get::<RectLight>(entity).cloned(),
            spot_light: self.get::<SpotLight>(entity).cloned(),
            active,
            render_layer_mask,
            mobility,
            rigid_body: self.get::<RigidBodyComponent>(entity).cloned(),
            collider: self.get::<ColliderComponent>(entity).cloned(),
            joint: self.get::<JointComponent>(entity).cloned(),
            animation_skeleton: self.get::<AnimationSkeletonComponent>(entity).cloned(),
            animation_player: self.get::<AnimationPlayerComponent>(entity).cloned(),
            animation_sequence_player: self
                .get::<AnimationSequencePlayerComponent>(entity)
                .cloned(),
            animation_graph_player: self.get::<AnimationGraphPlayerComponent>(entity).cloned(),
            animation_state_machine_player: self
                .get::<AnimationStateMachinePlayerComponent>(entity)
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
        let mut allocator = self.entity_id_allocator;
        for record in records {
            if self.contains_entity(record.id)
                || self.entity_registry.contains_stable(record.id)
                || records_by_id.insert(record.id, record).is_some()
            {
                return Err(SceneError::DuplicateEntity { entity: record.id });
            }
            validate_transform_for_write(record.id, record.transform)?;
            allocator.advance_past(record.id)?;
        }

        self.entity_registry
            .ensure_capacity_for_additional(records_by_id.len())?;
        self.validate_node_record_batch_mobility(&records_by_id)?;
        Ok((records_by_id, allocator.next_id()))
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
            self.insert_prevalidated_node_record(record);
        }
        self.entity_id_allocator
            .replace_next(next_id)
            .expect("prevalidated node records must retain a valid entity allocator state");
        self.bump_lifecycle_visibility_revision();
        self.mark_derived_state_dirty();
        self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        self.advance_world_generation();
        for entity in inserted_entities.iter().copied() {
            self.advance_scene_binding_generations_for_new_descendant(entity);
        }
        for entity in inserted_entities {
            self.record_world_fact(WorldFact::Spawned(entity));
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

    pub(super) fn insert_prevalidated_node_record(&mut self, record: NodeRecord) -> InternalEntity {
        let internal_entity = self.register_prevalidated_node_identity_without_components(&record);
        let mut row = self.begin_component_row(record.id);
        self.stage_component_row_value(&mut row, Name(record.name));
        self.stage_component_row_value(
            &mut row,
            Hierarchy {
                parent: record.parent,
            },
        );
        self.stage_component_row_value(
            &mut row,
            LocalTransform {
                transform: record.transform,
            },
        );
        self.stage_component_row_value(&mut row, ActiveSelf(record.active));
        self.stage_component_row_value(&mut row, RenderLayerMask(record.render_layer_mask));
        self.stage_component_row_value(&mut row, record.mobility);

        if let Some(camera) = record.camera {
            self.stage_component_row_value(&mut row, camera);
        }
        if let Some(mesh) = record.mesh {
            self.stage_component_row_value(&mut row, mesh);
        }
        if let Some(sprite_2d) = record.sprite_2d {
            self.stage_component_row_value(&mut row, sprite_2d);
        }
        if let Some(mesh_2d) = record.mesh_2d {
            self.stage_component_row_value(&mut row, mesh_2d);
        }
        if let Some(ambient_light) = record.ambient_light {
            self.stage_component_row_value(&mut row, ambient_light);
        }
        if let Some(directional_light) = record.directional_light {
            self.stage_component_row_value(&mut row, directional_light);
        }
        if let Some(point_light) = record.point_light {
            self.stage_component_row_value(&mut row, point_light);
        }
        if let Some(rect_light) = record.rect_light {
            self.stage_component_row_value(&mut row, rect_light);
        }
        if let Some(spot_light) = record.spot_light {
            self.stage_component_row_value(&mut row, spot_light);
        }
        if let Some(rigid_body) = record.rigid_body {
            self.stage_component_row_value(&mut row, rigid_body);
        }
        if let Some(collider) = record.collider {
            self.stage_component_row_value(&mut row, collider);
        }
        if let Some(joint) = record.joint {
            self.stage_component_row_value(&mut row, joint);
        }
        if let Some(animation_skeleton) = record.animation_skeleton {
            self.stage_component_row_value(&mut row, animation_skeleton);
        }
        if let Some(animation_player) = record.animation_player {
            self.stage_component_row_value(&mut row, animation_player);
        }
        if let Some(animation_sequence_player) = record.animation_sequence_player {
            self.stage_component_row_value(&mut row, animation_sequence_player);
        }
        if let Some(animation_graph_player) = record.animation_graph_player {
            self.stage_component_row_value(&mut row, animation_graph_player);
        }
        if let Some(animation_state_machine_player) = record.animation_state_machine_player {
            self.stage_component_row_value(&mut row, animation_state_machine_player);
        }
        self.commit_component_row(record.id, row, true);
        if self.active_camera == 0
            || !self.contains_component::<CameraComponent>(self.active_camera)
        {
            if self.contains_component::<CameraComponent>(record.id) {
                self.active_camera = record.id;
            }
        }
        internal_entity
    }

    /// Registers only the entity bookkeeping required by a bundle spawn.
    /// The bundle transaction remains the sole publisher of component values.
    pub(super) fn register_prevalidated_node_identity_without_components(
        &mut self,
        record: &NodeRecord,
    ) -> InternalEntity {
        let internal_entity = self.register_prevalidated_stable_entity(record.id);
        self.append_entity_to_dense_storage(record.id);
        self.kinds.insert(record.id, record.kind);
        self.record_node_kind_added(record.kind);
        internal_entity
    }

    fn validate_node_record_batch_mobility(
        &self,
        records_by_id: &BTreeMap<EntityId, &NodeRecord>,
    ) -> SceneResult<()> {
        let mut existing_static_child_parents = HashSet::new();
        for child in self.stable_entity_ids() {
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
        let Some(current) = self.get::<Name>(entity) else {
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
        components::{ActiveSelf, Hierarchy, LocalTransform, Mobility, Name, RenderLayerMask},
        ecs::{Component, LifecycleEventKind, StorageType},
    };

    #[derive(Debug, PartialEq, Eq)]
    struct RebuiltArchetypeMarker;

    impl Component for RebuiltArchetypeMarker {}

    #[derive(Debug, PartialEq, Eq)]
    struct RebuiltSparseArchetypeMarker;

    impl Component for RebuiltSparseArchetypeMarker {
        const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    }

    fn node_record_with_id(id: u64) -> crate::scene::components::NodeRecord {
        let mut source = World::empty();
        let entity = source
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
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
        let name_component_id = world.component_id::<Name>();
        world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |world, event| {
            observed_for_callback
                .lock()
                .expect("test observer lock")
                .push((
                    event.entity(),
                    world.contains_entity(second_id),
                    world
                        .entity_archetype_component_ids(second_id)
                        .contains(&name_component_id),
                ));
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
        assert!(
            observed
                .iter()
                .all(|(_, second_visible, second_has_final_signature)| {
                    *second_visible && *second_has_final_signature
                })
        );
    }

    #[test]
    fn node_record_batch_assigns_the_complete_fixed_component_signature() {
        let mut world = World::empty();
        let record = node_record_with_id(41);

        world
            .insert_node_record(record.clone())
            .expect("validated record must commit with its final signature");

        let name = world.component_id::<Name>();
        let hierarchy = world.component_id::<Hierarchy>();
        let transform = world.component_id::<LocalTransform>();
        let active = world.component_id::<ActiveSelf>();
        let render_layer_mask = world.component_id::<RenderLayerMask>();
        let mobility = world.component_id::<Mobility>();
        let signature = world.entity_archetype_component_ids(record.id);
        assert!(signature.contains(&name));
        assert!(signature.contains(&hierarchy));
        assert!(signature.contains(&transform));
        assert!(signature.contains(&active));
        assert!(signature.contains(&render_layer_mask));
        assert!(signature.contains(&mobility));
    }

    #[test]
    fn pending_component_row_publishes_dense_and_sparse_values_in_one_transition() {
        let mut world = World::empty();
        let entity = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        let marker = world.component_id::<RebuiltArchetypeMarker>();
        let sparse_marker = world.component_id::<RebuiltSparseArchetypeMarker>();
        let assignments_before = world.archetype_assignment_count();

        let mut row = world.begin_component_row(entity);
        world.stage_component_row_value(&mut row, RebuiltArchetypeMarker);
        world.stage_component_row_value(&mut row, RebuiltSparseArchetypeMarker);
        world.commit_component_row(entity, row, true);

        assert!(
            world
                .entity_archetype_component_ids(entity)
                .contains(&marker),
            "final signature reconstruction must not depend on a fixed component whitelist"
        );
        assert!(
            world
                .entity_archetype_component_ids(entity)
                .contains(&sparse_marker),
            "final signature reconstruction must preserve sparse canonical storage rows"
        );
        assert!(world.get::<RebuiltArchetypeMarker>(entity).is_some());
        assert!(world.get::<RebuiltSparseArchetypeMarker>(entity).is_some());
        assert_eq!(world.archetype_assignment_count() - assignments_before, 1);
    }

    #[test]
    fn node_record_batch_publishes_one_final_archetype_assignment_per_record() {
        let mut world = World::empty();
        let first = node_record_with_id(41);
        let second = node_record_with_id(42);
        let assignments_before = world.archetype_assignment_count();

        world
            .insert_owned_node_records(vec![first, second])
            .expect("prevalidated records should publish their final signatures");

        assert_eq!(world.archetype_assignment_count() - assignments_before, 2);
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
