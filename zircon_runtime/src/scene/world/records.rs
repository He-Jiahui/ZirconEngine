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
use crate::scene::{ecs::InternalEntity, EntityId, EntityRemap};
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

    /// Prepares copies with fresh IDs without mutating this World or its allocator.
    ///
    /// Parents within the batch follow their copies; external parents must still exist.
    /// Native references follow the dynamic-scene remapping contract. All fallible
    /// preparation finishes before any record changes. Insert the prepared records
    /// through [`Self::insert_owned_node_records`] while the source generation is current.
    pub fn prepare_node_record_copies(
        &self,
        records: &mut [NodeRecord],
    ) -> SceneResult<EntityRemap> {
        let mut sources = HashSet::with_capacity(records.len());
        for record in records.iter() {
            if !sources.insert(record.id) {
                return Err(SceneError::DuplicateEntity { entity: record.id });
            }
        }

        let mut allocator = self.entity_id_allocator;
        let mut remap = EntityRemap::new();
        for record in records.iter() {
            remap.insert(record.id, allocator.reserve_next()?);
        }
        // The ordinary insertion path must also be able to publish the successor state.
        allocator.next_available()?;
        for record in records.iter() {
            if let Some(parent) = record.parent {
                if remap.get(parent).is_none() && !self.contains_entity(parent) {
                    return Err(SceneError::MissingParent {
                        child: record.id,
                        parent,
                    });
                }
            }
        }

        for record in records {
            record.id = remap
                .get(record.id)
                .expect("every validated copy source must have a reserved entity ID");
            record.parent = record
                .parent
                .map(|parent| remap.get(parent).unwrap_or(parent));
            remap.remap_node_record_entity_references(record);
        }
        Ok(remap)
    }

    /// Quotes temporary validation and publication allocations without cloning records.
    ///
    /// Includes the post-insertion population so the same quote also covers an undo
    /// followed by compensation. Input record payloads, persistent World tables and
    /// callback-owned effects remain with their respective owners.
    pub fn node_record_insertion_peak_v1(
        &self,
        records: &[NodeRecord],
        meter: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        use crate::scene::ecs::{
            ComponentId, ComponentLifecycleEvent, ComponentTicks, StoredComponent,
        };
        use zircon_runtime_interface::retained_memory::{
            FreshHashStorageV1, RetainedBoundBuilderV1, RetentionContractError,
        };

        fn column_layout_array<T>(
            _: &[T],
            count: usize,
            meter: &mut RetainedBoundBuilderV1,
        ) -> Result<(), RetentionContractError> {
            meter.array::<T>(count)
        }

        meter.items(records.len())?;
        meter.btree_construction::<EntityId, &NodeRecord>(records.len())?;
        let mut existing_static_parents = 0usize;
        for entity in self.stable_entity_ids() {
            meter.items(1)?;
            if self.mobility(entity) == Some(Mobility::Static) && self.parent_of(entity).is_some() {
                existing_static_parents = existing_static_parents
                    .checked_add(1)
                    .ok_or(RetentionContractError::Overflow)?;
            }
        }
        let mut incoming_static_parents = 0usize;
        let mut missing_records = 0usize;
        for record in records {
            let missing = !self.contains_entity(record.id);
            missing_records = missing_records
                .checked_add(usize::from(missing))
                .ok_or(RetentionContractError::Overflow)?;
            if record.mobility == Mobility::Static && record.parent.is_some() {
                incoming_static_parents = incoming_static_parents
                    .checked_add(1)
                    .ok_or(RetentionContractError::Overflow)?;
                if missing {
                    existing_static_parents = existing_static_parents
                        .checked_add(1)
                        .ok_or(RetentionContractError::Overflow)?;
                }
            }
        }
        // Both mobility sets start empty and grow only by insertion. Counting
        // parent insertions bounds unique keys; two final tables cover rehash overlap.
        for count in [existing_static_parents, incoming_static_parents] {
            let table = FreshHashStorageV1::for_request::<EntityId>(count)?;
            meter.add(
                table
                    .allocation::<EntityId>()?
                    .checked_mul(2)
                    .ok_or(RetentionContractError::Overflow)?,
            )?;
        }
        meter.array::<EntityId>(records.len())?;

        // This empty projection allocates nothing and exposes the private column
        // metadata's actual element layout to the generic accounting operation.
        let column_layouts = self
            .component_registry
            .table_column_layouts_for_ids(&[])
            .ok_or(RetentionContractError::Stale)?;
        let mut event_count = 0usize;
        for record in records {
            let mut components = 0usize;
            macro_rules! component {
                ($ty:ty) => {{
                    components = components
                        .checked_add(1)
                        .ok_or(RetentionContractError::Overflow)?;
                    meter.items(1)?;
                    // Values move through boxed native transport; their owned
                    // children are already in the caller's record payload quote.
                    meter.layout(std::mem::size_of::<$ty>(), std::mem::align_of::<$ty>())?;
                    // Add and Insert each construct a distinct type-name Arc.
                    meter.arc_payload(std::any::type_name::<$ty>())?;
                    meter.arc_payload(std::any::type_name::<$ty>())?;
                }};
            }
            component!(Name);
            component!(Hierarchy);
            component!(LocalTransform);
            component!(ActiveSelf);
            component!(RenderLayerMask);
            component!(Mobility);
            macro_rules! optional_component {
                ($($field:ident: $ty:ty),* $(,)?) => {$(
                    if record.$field.is_some() {
                        component!($ty);
                    }
                )*};
            }
            optional_component!(
                camera: CameraComponent,
                mesh: MeshRenderer,
                sprite_2d: Sprite2dComponent,
                mesh_2d: Mesh2dComponent,
                ambient_light: AmbientLight,
                directional_light: DirectionalLight,
                point_light: PointLight,
                rect_light: RectLight,
                spot_light: SpotLight,
                rigid_body: RigidBodyComponent,
                collider: ColliderComponent,
                joint: JointComponent,
                animation_skeleton: AnimationSkeletonComponent,
                animation_player: AnimationPlayerComponent,
                animation_sequence_player: AnimationSequencePlayerComponent,
                animation_graph_player: AnimationGraphPlayerComponent,
                animation_state_machine_player: AnimationStateMachinePlayerComponent,
            );
            meter.btree_construction::<ComponentId, bool>(components)?;
            meter.btree_construction::<ComponentId, Option<(StoredComponent, ComponentTicks)>>(
                components,
            )?;
            meter
                .btree_construction::<ComponentId, (StoredComponent, ComponentTicks)>(components)?;
            // Signature insertion, its growth, publication clones and table
            // membership validation coexist; match the staged-row allowance.
            let signature_entries = components
                .checked_add(4)
                .and_then(|count| count.checked_mul(8))
                .ok_or(RetentionContractError::Overflow)?;
            meter.array::<ComponentId>(signature_entries)?;
            column_layout_array(
                &column_layouts,
                components
                    .checked_mul(2)
                    .ok_or(RetentionContractError::Overflow)?,
                meter,
            )?;
            event_count = components
                .checked_mul(2)
                .and_then(|count| event_count.checked_add(count))
                .ok_or(RetentionContractError::Overflow)?;
        }
        // Push growth and split_off's dispatch vector can overlap. Existing
        // World-owned spare capacity is never inferred to be a fresh allocation.
        let event_capacity = self
            .staged_lifecycle_events
            .len()
            .checked_add(event_count)
            .and_then(|count| count.max(4).checked_mul(4))
            .ok_or(RetentionContractError::Overflow)?;
        meter.array::<ComponentLifecycleEvent>(event_capacity)?;
        // New-descendant publication builds one ancestor Vec at a time. Bound
        // its longest possible chain, including the incoming rows and Vec growth.
        let ancestors = self
            .entities
            .len()
            .checked_add(missing_records)
            .and_then(|count| count.checked_add(1))
            .and_then(|count| count.max(4).checked_mul(4))
            .ok_or(RetentionContractError::Overflow)?;
        meter.array::<EntityId>(ancestors)
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

    // 整批记录已预检；暂存生命周期回调，直到所有实体和最终组件签名可见后才向观察者发布。
    fn commit_node_record_batch(&mut self, batch: PreparedNodeRecordBatch) -> SceneResult<()> {
        if batch.records.is_empty() {
            return Ok(());
        }

        let execution = self.begin_schedule_execution();
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
            if let Some(target) = execution.target_change(self) {
                return Err(SceneError::LifecycleTargetReplaced {
                    operation: "insert node records",
                    target,
                });
            }
            self.dispatch_component_lifecycle(event);
        }
        if let Some(target) = execution.target_change(self) {
            return Err(SceneError::LifecycleTargetReplaced {
                operation: "insert node records",
                target,
            });
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
#[path = "tests/records.rs"]
mod tests;
