use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::scene::components::{CameraComponent, NodeRecord};
use crate::scene::ecs::{
    ArchetypeSignature, ChangeTick, ComponentId, ComponentStorage, ComponentTicks,
    LifecycleEventKind, PreflightedTransferredDescriptorImports, Resource, ResourceStore,
    StorageError, StorageType, TransferredComponentDescriptor, TransferredComponentRow,
    TransferredResourceRow,
};
use crate::scene::EntityId;
use zircon_runtime_interface::reflect::ReflectError;

use super::{SceneError, SceneResult, World};

/// One component row detached from the isolated preflight World. Both the
/// stable entity id and descriptor transfer are target-independent.
pub(in crate::scene) struct PreflightComponentRow {
    pub(in crate::scene) entity: EntityId,
    pub(in crate::scene) descriptor: TransferredComponentDescriptor,
    pub(in crate::scene) row: TransferredComponentRow,
}

/// Plugin JSON stays separate from its sparse presence row, which is carried
/// by [`PreflightComponentRow`] and enters the final archetype signature.
pub(in crate::scene) struct PreflightDynamicComponent {
    pub(in crate::scene) entity: EntityId,
    pub(in crate::scene) component_id: String,
    pub(in crate::scene) value: serde_json::Value,
}

pub(in crate::scene) struct RetiredWorldTransactionState {
    _world: World,
    _replaced_resources: ResourceStore,
}

pub(in crate::scene) struct PreflightedDynamicComponentType {
    descriptor: crate::core::framework::scene::ComponentTypeDescriptor,
    registration: crate::scene::reflect::RuntimeTypeRegistration,
}

pub(in crate::scene) struct PreflightedDynamicScenePublication {
    records: Vec<NodeRecord>,
    descriptor_imports: PreflightedTransferredDescriptorImports,
    rows_by_entity: HashMap<
        EntityId,
        Vec<(
            ComponentId,
            crate::scene::ecs::PreflightedTransferredComponentRow,
        )>,
    >,
    dynamic_components: Vec<PreflightDynamicComponent>,
    resource_rows: Vec<TransferredResourceRow>,
    component_types: Vec<PreflightedDynamicComponentType>,
    next_id: EntityId,
}

mod detached_entity_batch;
pub use detached_entity_batch::{DetachedEntityBatch, DetachedEntityBatchRestoreError};

impl World {
    /// Clones one reflected component into an isolated preflight World. The
    /// caller must have staged the target identity before this operation.
    pub(in crate::scene) fn stage_reflected_component_clone(
        &self,
        entity: EntityId,
        type_path: &str,
        target: &mut World,
    ) -> Result<(), ReflectError> {
        let registration = self.type_registry.runtime_registration(type_path)?;
        let Some(component) = registration.component.as_ref() else {
            return Err(ReflectError::InvalidRegistration {
                type_path: registration.registration.type_path.type_path.clone(),
                reason: "registered type has no component staging adapter".to_string(),
            });
        };
        if !component.stage_clone(self, entity, target)? {
            return Err(ReflectError::InvalidRegistration {
                type_path: component.type_path.clone(),
                reason: "component reflection has no affected-row staging clone adapter"
                    .to_string(),
            });
        }
        Ok(())
    }

    /// Moves one concrete resource out of an isolated preflight World. Resource
    /// adapters invoke this before target publication, so their failure cannot
    /// expose a partial live-world mutation.
    pub(in crate::scene) fn transfer_preflight_resource<T>(
        &mut self,
        artifact: &mut World,
    ) -> Result<(), ReflectError>
    where
        T: Resource,
    {
        let resource =
            self.resources
                .remove::<T>()
                .ok_or_else(|| ReflectError::MissingResource {
                    type_path: std::any::type_name::<T>().to_string(),
                })?;
        artifact
            .resources
            .insert_at_tick(resource, ChangeTick::INITIAL);
        Ok(())
    }

    pub(in crate::scene) fn take_preflight_resource_rows(&mut self) -> Vec<TransferredResourceRow> {
        self.resources.take_transferred_rows()
    }

    pub(in crate::scene) fn publish_preflight_resource_rows(
        &mut self,
        rows: Vec<TransferredResourceRow>,
        tick: ChangeTick,
    ) {
        self.resources.insert_transferred_rows(rows, tick);
    }

    pub(in crate::scene) fn take_preflight_component_rows(
        &mut self,
        entities: &[EntityId],
    ) -> SceneResult<Vec<PreflightComponentRow>> {
        let mut rows = Vec::new();
        for entity in entities {
            let internal = self.internal_entity(*entity).ok_or_else(|| {
                SceneError::missing_entity("extract preflight component rows for", *entity)
            })?;
            let location = self
                .internal_entity_location(*entity)
                .expect("preflight component extraction requires an archetype row")
                .location;
            let signature = self
                .archetype_index
                .signature(location.archetype_id)
                .expect("preflight entity location must identify an archetype");
            let table_components = signature.table_components().to_vec();
            let sparse_components = signature.sparse_set_components().to_vec();
            let components = self.take_archetype_row_components(*entity, location);
            debug_assert_eq!(components.len(), table_components.len());
            for (component_id, (value, ticks)) in components {
                let descriptor = self
                    .component_registry
                    .transferred_descriptor(component_id)
                    .ok_or(StorageError::ComponentTypeMismatch { component_id })?;
                rows.push(PreflightComponentRow {
                    entity: *entity,
                    descriptor,
                    row: ComponentStorage::transferred_table_row(component_id, ticks, value),
                });
            }
            for row in self
                .component_storage
                .extract_entity_rows(internal, &sparse_components)
            {
                let component_id = row.component_id();
                let descriptor = self
                    .component_registry
                    .transferred_descriptor(component_id)
                    .ok_or(StorageError::ComponentTypeMismatch { component_id })?;
                rows.push(PreflightComponentRow {
                    entity: *entity,
                    descriptor,
                    row,
                });
            }
        }
        Ok(rows)
    }

    pub(in crate::scene) fn preflight_dynamic_components(
        &self,
        entities: &[EntityId],
    ) -> Vec<PreflightDynamicComponent> {
        let mut components = Vec::new();
        for entity in entities {
            components.extend(self.dynamic_components_for_entity(*entity).into_iter().map(
                |component| PreflightDynamicComponent {
                    entity: *entity,
                    component_id: component.component_id,
                    value: component.value,
                },
            ));
        }
        components
    }

    /// Resolves every target-local descriptor and storage row before the live
    /// World exposes any part of the dynamic-scene mutation.
    pub(in crate::scene) fn preflight_dynamic_scene_publication(
        &self,
        component_type_descriptors: Vec<crate::core::framework::scene::ComponentTypeDescriptor>,
        records: Vec<NodeRecord>,
        component_rows: Vec<PreflightComponentRow>,
        dynamic_components: Vec<PreflightDynamicComponent>,
        resource_rows: Vec<TransferredResourceRow>,
    ) -> SceneResult<PreflightedDynamicScenePublication> {
        self.validate_owned_node_records(&records)?;
        let mut descriptor_imports = self
            .component_registry
            .begin_transferred_descriptor_imports();
        let mut component_types = Vec::new();
        let mut registration_validation = crate::scene::reflect::TypeRegistry::default();
        let mut pending_component_type_ids = BTreeSet::new();
        for descriptor in component_type_descriptors {
            if let Some(existing) = self.component_type_descriptor(&descriptor.type_id) {
                if existing != &descriptor {
                    return Err(SceneError::DuplicateComponentType {
                        type_id: descriptor.type_id,
                    });
                }
                continue;
            }
            self.component_types.validate_new_descriptor(&descriptor)?;
            if !pending_component_type_ids.insert(descriptor.type_id.clone()) {
                return Err(SceneError::DuplicateComponentType {
                    type_id: descriptor.type_id,
                });
            }
            if self.type_registry.contains_type_path(&descriptor.type_id) {
                return Err(
                    zircon_runtime_interface::reflect::ReflectError::DuplicateTypePath {
                        type_path: descriptor.type_id,
                    }
                    .into(),
                );
            }
            let registration =
                crate::scene::reflect::registration_from_component_descriptor(&descriptor)?;
            let component =
                crate::scene::reflect::reflect_component_for_dynamic_descriptor(&descriptor);
            let runtime_registration = crate::scene::reflect::RuntimeTypeRegistration {
                component: Some(component),
                registration,
                resource: None,
            };
            registration_validation.register(runtime_registration.clone())?;
            self.component_registry
                .preflight_dynamic_descriptor_import(&mut descriptor_imports, &descriptor.type_id);
            component_types.push(PreflightedDynamicComponentType {
                descriptor,
                registration: runtime_registration,
            });
        }
        let mut rows_by_entity: HashMap<
            EntityId,
            Vec<(
                ComponentId,
                crate::scene::ecs::PreflightedTransferredComponentRow,
            )>,
        > = HashMap::new();
        for component_row in component_rows {
            let component_id = self
                .component_registry
                .preflight_transferred_descriptor_import(
                    &mut descriptor_imports,
                    &component_row.descriptor,
                )
                .ok_or(StorageError::ComponentTypeMismatch {
                    component_id: component_row.row.component_id(),
                })?;
            let row = self
                .component_storage
                .preflight_transferred_row(component_id, component_row.row)?;
            rows_by_entity
                .entry(component_row.entity)
                .or_default()
                .push((component_id, row));
        }

        let mut next_id = self.next_id;
        for record in &records {
            next_id = next_id.max(
                record
                    .id
                    .checked_add(1)
                    .ok_or(SceneError::EntityIdExhausted { entity: record.id })?,
            );
        }

        Ok(PreflightedDynamicScenePublication {
            records,
            descriptor_imports,
            rows_by_entity,
            dynamic_components,
            resource_rows,
            component_types,
            next_id,
        })
    }

    /// Publishes an artifact whose descriptor, row, identity, and allocator
    /// checks have all completed. No recoverable work remains in this half.
    pub(in crate::scene) fn publish_preflighted_dynamic_scene(
        &mut self,
        publication: PreflightedDynamicScenePublication,
    ) {
        let PreflightedDynamicScenePublication {
            records,
            descriptor_imports,
            mut rows_by_entity,
            dynamic_components,
            resource_rows,
            component_types,
            next_id,
        } = publication;

        crate::profile_counter!(
            "runtime",
            "dynamic_scene.transaction.component_registry.imported_descriptors",
            descriptor_imports.imported_descriptor_count()
        );
        crate::profile_counter!(
            "runtime",
            "dynamic_scene.transaction.component_registry.reused_descriptor_resolves",
            descriptor_imports.reused_descriptor_resolve_count()
        );
        self.component_registry
            .publish_preflighted_transferred_descriptor_imports(descriptor_imports);
        for component_type in component_types {
            self.publish_prevalidated_dynamic_component_type(
                component_type.descriptor,
                component_type.registration,
            );
        }
        let prior_lifecycle_staging =
            std::mem::replace(&mut self.record_staged_lifecycle_events, true);
        let lifecycle_start = self.staged_lifecycle_events.len();
        let tick = self.mutation_change_tick();
        let entities = records.iter().map(|record| record.id).collect::<Vec<_>>();
        let mut internals = HashMap::with_capacity(entities.len());
        for record in records {
            let entity = record.id;
            let internal = self.register_prevalidated_node_identity_without_components(&record);
            let prior = internals.insert(entity, internal);
            debug_assert!(
                prior.is_none(),
                "prevalidated records must have unique entity ids"
            );
        }
        self.next_id = self.next_id.max(next_id);

        for component in dynamic_components {
            self.dynamic_components
                .entry(component.entity)
                .or_default()
                .insert(component.component_id.clone(), component.value);
            self.inspection_artifact_cache
                .mark_fields_dirty(component.entity);
            self.advance_dynamic_component_generation(&component.component_id);
            self.invalidate_world_component_type(&component.component_id);
        }
        self.publish_preflight_resource_rows(resource_rows, tick);

        for entity in entities.iter().copied() {
            let internal = *internals
                .get(&entity)
                .expect("preflight signatures must target a published transaction identity");
            let prepared_rows = rows_by_entity.remove(&entity).unwrap_or_default();
            let mut table_components = Vec::new();
            let mut sparse_set_components = Vec::new();
            let mut table_updates = BTreeMap::new();
            let mut lifecycle_component_ids = Vec::with_capacity(prepared_rows.len());
            for (component_id, row) in prepared_rows {
                lifecycle_component_ids.push(component_id);
                match ComponentStorage::preflighted_transferred_storage_type(&row) {
                    StorageType::Table => {
                        table_components.push(component_id);
                        let previous = table_updates.insert(
                            component_id,
                            Some((
                                ComponentStorage::take_preflighted_transferred_value(row),
                                ComponentTicks::new(tick),
                            )),
                        );
                        debug_assert!(previous.is_none());
                    }
                    StorageType::SparseSet => {
                        sparse_set_components.push(component_id);
                        let replaced = self
                            .component_storage
                            .insert_preflighted_transferred_row(internal, row, tick);
                        debug_assert!(
                            !replaced,
                            "preflight rows may only target identities created by this transaction"
                        );
                    }
                }
            }

            let signature = ArchetypeSignature::new(table_components, sparse_set_components);
            if signature != ArchetypeSignature::empty() {
                let previous =
                    self.transition_entity_archetype_row(entity, signature, table_updates);
                debug_assert!(previous.is_some_and(|values| values.is_empty()));
            } else {
                debug_assert!(table_updates.is_empty());
            }
            lifecycle_component_ids.sort_unstable();
            for component_id in lifecycle_component_ids {
                self.trigger_component_lifecycle(LifecycleEventKind::Add, entity, component_id);
                self.trigger_component_lifecycle(LifecycleEventKind::Insert, entity, component_id);
            }
            self.advance_scene_binding_generations_for_new_descendant(entity);
            self.record_world_fact(zircon_runtime_interface::world_sync::WorldFact::Spawned(
                entity,
            ));
            if (self.active_camera == 0
                || !self.contains_component::<CameraComponent>(self.active_camera))
                && self.contains_component::<CameraComponent>(entity)
            {
                self.active_camera = entity;
            }
        }
        debug_assert!(rows_by_entity.is_empty());

        if !entities.is_empty() {
            self.bump_lifecycle_visibility_revision();
            self.mark_derived_state_dirty();
            self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        }
        self.advance_world_generation();
        self.record_staged_lifecycle_events = prior_lifecycle_staging;
        if !prior_lifecycle_staging {
            for event in self.staged_lifecycle_events.split_off(lifecycle_start) {
                self.dispatch_component_lifecycle(event);
            }
        }
    }

    /// Creates an isolated World that can validate a dynamic-scene mutation.
    ///
    /// The target's entity rows, component storage, resources, callbacks, and
    /// runtime queues deliberately stay out of this projection. Schema state is
    /// projected by canonical type path, so preflight work scales with the
    /// compiled mutation instead of the target's complete catalog.
    pub(in crate::scene) fn dynamic_scene_preflight_world<'type_path>(
        &self,
        affected_type_paths: impl IntoIterator<Item = &'type_path str>,
    ) -> Self {
        let mut preflight = Self::empty();
        preflight.component_types = Default::default();
        preflight.type_registry = Default::default();
        preflight.vm_catalog_type_paths.clear();
        preflight.vm_dynamic_type_paths.clear();

        let affected_type_paths = affected_type_paths
            .into_iter()
            .map(|type_path| {
                self.type_registry
                    .resolve(type_path)
                    .unwrap_or(type_path)
                    .to_string()
            })
            .collect::<BTreeSet<_>>();
        let affected_type_path_count = affected_type_paths.len();
        let mut projected_component_descriptors = 0_usize;
        let mut projected_runtime_registrations = 0_usize;
        let mut projected_vm_catalog_type_paths = 0_usize;
        let mut projected_vm_dynamic_type_paths = 0_usize;
        for type_path in affected_type_paths {
            if let Some(descriptor) = self.component_types.descriptor(&type_path) {
                preflight
                    .component_types
                    .register(descriptor.clone())
                    .expect("registered component descriptors must remain valid when projected");
                projected_component_descriptors += 1;
            }
            if let Ok(registration) = self.type_registry.runtime_registration(&type_path) {
                let canonical_type_path = registration.registration.type_path.type_path.as_str();
                if !preflight
                    .type_registry
                    .contains_type_path(canonical_type_path)
                {
                    preflight
                        .type_registry
                        .register(registration.clone())
                        .expect("registered runtime types must remain valid when projected");
                    projected_runtime_registrations += 1;
                }
            }
            if self.vm_catalog_type_paths.contains(&type_path) {
                preflight
                    .vm_catalog_type_paths
                    .insert(type_path.to_string());
                projected_vm_catalog_type_paths += 1;
            }
            if self.vm_dynamic_type_paths.contains(&type_path) {
                preflight
                    .vm_dynamic_type_paths
                    .insert(type_path.to_string());
                projected_vm_dynamic_type_paths += 1;
            }
        }
        crate::profile_counter!(
            "runtime",
            "dynamic_scene.transaction.preflight_schema.affected_type_paths",
            affected_type_path_count
        );
        crate::profile_counter!(
            "runtime",
            "dynamic_scene.transaction.preflight_schema.projected_component_descriptors",
            projected_component_descriptors
        );
        crate::profile_counter!(
            "runtime",
            "dynamic_scene.transaction.preflight_schema.projected_runtime_registrations",
            projected_runtime_registrations
        );
        crate::profile_counter!(
            "runtime",
            "dynamic_scene.transaction.preflight_schema.projected_vm_catalog_type_paths",
            projected_vm_catalog_type_paths
        );
        crate::profile_counter!(
            "runtime",
            "dynamic_scene.transaction.preflight_schema.projected_vm_dynamic_type_paths",
            projected_vm_dynamic_type_paths
        );
        preflight
    }

    pub(in crate::scene) fn dynamic_component_type_catalog_is_empty(&self) -> bool {
        self.component_types.is_empty()
    }

    pub(in crate::scene) fn commit_staged_scene_state(
        &mut self,
        mut staged: World,
    ) -> RetiredWorldTransactionState {
        staged.advance_dynamic_component_generations_after(self);
        staged.advance_scene_binding_generations_after(self);
        staged.advance_world_generation_after(self.world_generation());
        staged.record_staged_lifecycle_events = false;
        let staged_lifecycle_events = std::mem::take(&mut staged.staged_lifecycle_events);
        let mut live_resources = std::mem::take(&mut self.resources);
        let replaced_resources =
            live_resources.merge_overrides_from(std::mem::take(&mut staged.resources));
        staged.resources = live_resources;

        // These containers carry live callbacks, queued work, and runtime-only state.
        // Scene staging intentionally uses their empty/clone projections and must not
        // replace the authoritative instances at commit.
        staged.schedule = std::mem::take(&mut self.schedule);
        staged.removed_component_events = std::mem::take(&mut self.removed_component_events);
        staged.events = std::mem::take(&mut self.events);
        staged.event_mirrors = std::mem::take(&mut self.event_mirrors);
        staged.messages = std::mem::take(&mut self.messages);
        staged.observers = std::mem::take(&mut self.observers);
        staged.command_queue = std::mem::take(&mut self.command_queue);
        staged.deferred_command_errors = std::mem::take(&mut self.deferred_command_errors);
        staged.ecs_frame_performance_diagnostics =
            std::mem::take(&mut self.ecs_frame_performance_diagnostics);

        let retired = RetiredWorldTransactionState {
            _world: std::mem::replace(self, staged),
            _replaced_resources: replaced_resources,
        };
        for event in staged_lifecycle_events {
            self.dispatch_component_lifecycle(event);
        }
        retired
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
    use crate::scene::NodeKind;

    use super::World;

    #[test]
    fn staged_world_commit_stales_compiled_binding_when_entity_ids_are_reused() {
        let mut current = World::empty();
        let root = current.spawn_node(NodeKind::Empty);
        let hero = current.spawn_node(NodeKind::Mesh);
        current.rename_node(root, "Root").unwrap();
        current.rename_node(hero, "Hero").unwrap();
        current.set_parent_checked(hero, Some(root)).unwrap();
        let writer = current
            .compile_scene_property_writer(
                &EntityPath::parse("Root/Hero").unwrap(),
                &ComponentPropertyPath::parse("Transform.translation").unwrap(),
            )
            .unwrap()
            .unwrap();

        let mut staged = World::empty();
        let staged_root = staged.spawn_node(NodeKind::Empty);
        let staged_hero = staged.spawn_node(NodeKind::Mesh);
        assert_eq!(root, staged_root);
        assert_eq!(hero, staged_hero);
        staged.rename_node(staged_root, "Root").unwrap();
        staged.rename_node(staged_hero, "Hero").unwrap();
        staged
            .set_parent_checked(staged_hero, Some(staged_root))
            .unwrap();

        let _retired = current.commit_staged_scene_state(staged);

        assert!(!writer.is_current_for(&current));
    }
}
