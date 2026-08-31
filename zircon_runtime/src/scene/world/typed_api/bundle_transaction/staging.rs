use std::any::TypeId;

use crate::scene::components::{
    ActiveSelf, AmbientLight, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, Mesh2dComponent, Mobility, Name, NodeRecord,
    PointLight, RectLight, RenderLayerMask, RigidBodyComponent, SpotLight, Sprite2dComponent,
};
use crate::scene::ecs::{Component, ComponentId};

use super::super::{SceneError, SceneResult};
use super::{
    BundleInsertionTransaction, MAX_BUNDLE_COMPONENT_TYPES, MAX_BUNDLE_COMPONENTS,
    MAX_NODE_RECORD_COMPONENT_TYPES, PendingBundleValue, PendingDeferredRemoval,
    PreflightedBundleComponent, UnregisteredBundleComponentType, register_component_id,
};

impl BundleInsertionTransaction<'_> {
    pub(super) fn stage<T>(&mut self, component: T) -> SceneResult<()>
    where
        T: Component,
    {
        self.stage_component(component, false)
    }

    pub(super) fn stage_deferred<T>(&mut self, component: T) -> SceneResult<()>
    where
        T: Component,
    {
        self.stage_component(component, true)
    }

    fn stage_component<T>(&mut self, component: T, allow_replacement: bool) -> SceneResult<()>
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        let existing_component_index = self.components[..self.component_count]
            .iter()
            .flatten()
            .position(|preflight| preflight.type_id == type_id);
        if existing_component_index.is_some() && !allow_replacement {
            return Err(SceneError::DuplicateBundleComponentType);
        }
        if existing_component_index.is_none() && self.component_count >= MAX_BUNDLE_COMPONENTS {
            return Err(SceneError::BundleComponentLimitExceeded {
                limit: MAX_BUNDLE_COMPONENTS,
            });
        }

        let staged_mobility = (&component as &dyn std::any::Any)
            .downcast_ref::<Mobility>()
            .copied();
        if staged_mobility.is_none() {
            self.world
                .validate_fixed_component(self.entity, &component)?;
        }
        let component_id = self.staged_component_id::<T>()?;
        let storage_preflight = self
            .world
            .component_storage
            .preflight_insert::<T>(component_id, T::STORAGE_TYPE)?;

        self.cancel_deferred_removal(type_id);
        self.record_staged_fixed_state(&component, staged_mobility);

        let component_index = existing_component_index.unwrap_or(self.component_count);
        self.components[component_index] = Some(PreflightedBundleComponent {
            component_id,
            storage_type: T::STORAGE_TYPE,
            type_id,
        });
        self.pending_values[component_index] = Some(Box::new(PendingBundleValue {
            component,
            storage_preflight,
        }));
        if existing_component_index.is_none() {
            self.component_count += 1;
        }
        self.final_state_validated.set(false);
        Ok(())
    }

    pub(crate) fn stage_deferred_remove<T>(&mut self) -> SceneResult<()>
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        self.remove_staged_component(type_id);
        self.remove_staged_default_value(type_id);
        if type_id == TypeId::of::<Hierarchy>() {
            self.staged_hierarchy_parent = None;
        }
        if type_id == TypeId::of::<Mobility>() {
            self.staged_mobility = None;
        }

        let Some(component_id) = self.world.registered_component_id::<T>() else {
            self.final_state_validated.set(false);
            return Ok(());
        };
        self.world
            .component_storage
            .validate_insert::<T>(component_id, T::STORAGE_TYPE)?;
        if !self.world.contains_component_id(self.entity, component_id)
            || self.deferred_removals[..self.deferred_removal_count]
                .iter()
                .flatten()
                .any(|removal| removal.type_id() == type_id)
        {
            self.final_state_validated.set(false);
            return Ok(());
        }
        let Some(slot) = self.deferred_removals.get_mut(self.deferred_removal_count) else {
            return Err(SceneError::BundleComponentLimitExceeded {
                limit: MAX_BUNDLE_COMPONENT_TYPES,
            });
        };
        *slot = Some(PendingDeferredRemoval::new::<T>(component_id));
        self.deferred_removal_count += 1;
        self.final_state_validated.set(false);
        Ok(())
    }

    fn cancel_deferred_removal(&mut self, type_id: TypeId) {
        let Some(index) = self.deferred_removals[..self.deferred_removal_count]
            .iter()
            .flatten()
            .position(|removal| removal.type_id() == type_id)
        else {
            return;
        };
        for next in index + 1..self.deferred_removal_count {
            self.deferred_removals[next - 1] = self.deferred_removals[next].take();
        }
        self.deferred_removal_count -= 1;
        self.deferred_removals[self.deferred_removal_count] = None;
    }

    fn remove_staged_component(&mut self, type_id: TypeId) {
        let Some(index) = self.components[..self.component_count]
            .iter()
            .flatten()
            .position(|component| component.type_id == type_id)
        else {
            return;
        };
        for next in index + 1..self.component_count {
            self.components[next - 1] = self.components[next].take();
            self.pending_values[next - 1] = self.pending_values[next].take();
        }
        self.component_count -= 1;
        self.components[self.component_count] = None;
        self.pending_values[self.component_count] = None;
    }

    fn remove_staged_default_value(&mut self, type_id: TypeId) {
        let Some(index) = self.default_values[..self.default_value_count]
            .iter()
            .flatten()
            .position(|value| value.type_id() == type_id)
        else {
            return;
        };
        for next in index + 1..self.default_value_count {
            self.default_values[next - 1] = self.default_values[next].take();
        }
        self.default_value_count -= 1;
        self.default_values[self.default_value_count] = None;
    }

    pub(super) fn has_deferred_removal<T>(&self) -> bool
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        self.deferred_removals[..self.deferred_removal_count]
            .iter()
            .flatten()
            .any(|removal| removal.type_id() == type_id)
    }

    pub(super) fn stage_default_node_record_components(
        &mut self,
        record: &NodeRecord,
    ) -> SceneResult<()> {
        self.stage_default_component(Name(record.name.clone()))?;
        self.stage_default_component(Hierarchy {
            parent: record.parent,
        })?;
        self.stage_default_component(LocalTransform {
            transform: record.transform,
        })?;
        self.stage_default_component(ActiveSelf(record.active))?;
        self.stage_default_component(RenderLayerMask(record.render_layer_mask))?;
        if let Some(camera) = record.camera.clone() {
            self.stage_default_component(camera)?;
        }
        if let Some(mesh) = record.mesh.clone() {
            self.stage_default_component(mesh)?;
        }
        if let Some(sprite_2d) = record.sprite_2d.clone() {
            self.stage_default_component(sprite_2d)?;
        }
        if let Some(mesh_2d) = record.mesh_2d.clone() {
            self.stage_default_component(mesh_2d)?;
        }
        if let Some(rigid_body) = record.rigid_body.clone() {
            self.stage_default_component(rigid_body)?;
        }
        if let Some(collider) = record.collider.clone() {
            self.stage_default_component(collider)?;
        }
        if let Some(joint) = record.joint.clone() {
            self.stage_default_component(joint)?;
        }
        if let Some(animation_skeleton) = record.animation_skeleton.clone() {
            self.stage_default_component(animation_skeleton)?;
        }
        if let Some(animation_player) = record.animation_player.clone() {
            self.stage_default_component(animation_player)?;
        }
        if let Some(animation_sequence_player) = record.animation_sequence_player.clone() {
            self.stage_default_component(animation_sequence_player)?;
        }
        if let Some(animation_graph_player) = record.animation_graph_player.clone() {
            self.stage_default_component(animation_graph_player)?;
        }
        if let Some(animation_state_machine_player) = record.animation_state_machine_player.clone()
        {
            self.stage_default_component(animation_state_machine_player)?;
        }
        if let Some(ambient_light) = record.ambient_light.clone() {
            self.stage_default_component(ambient_light)?;
        }
        if let Some(directional_light) = record.directional_light.clone() {
            self.stage_default_component(directional_light)?;
        }
        if let Some(point_light) = record.point_light.clone() {
            self.stage_default_component(point_light)?;
        }
        if let Some(rect_light) = record.rect_light.clone() {
            self.stage_default_component(rect_light)?;
        }
        if let Some(spot_light) = record.spot_light.clone() {
            self.stage_default_component(spot_light)?;
        }
        self.stage_default_component(record.mobility)
    }

    fn stage_default_component<T>(&mut self, component: T) -> SceneResult<()>
    where
        T: Component,
    {
        if self.default_value_count >= MAX_NODE_RECORD_COMPONENT_TYPES {
            return Err(SceneError::BundleTransactionInvariant {
                reason: "node record default component capacity was exceeded",
            });
        }

        let staged_mobility = (&component as &dyn std::any::Any)
            .downcast_ref::<Mobility>()
            .copied();
        if staged_mobility.is_none() {
            self.world
                .validate_fixed_component(self.entity, &component)?;
        }
        let component_id = self.staged_component_id::<T>()?;
        let storage_preflight = self
            .world
            .component_storage
            .preflight_insert::<T>(component_id, T::STORAGE_TYPE)?;

        self.record_staged_fixed_state(&component, staged_mobility);
        self.default_values[self.default_value_count] = Some(Box::new(PendingBundleValue {
            component,
            storage_preflight,
        }));
        self.default_value_count += 1;
        Ok(())
    }

    fn record_staged_fixed_state<T>(&mut self, component: &T, staged_mobility: Option<Mobility>)
    where
        T: Component,
    {
        if let Some(hierarchy) = (component as &dyn std::any::Any).downcast_ref::<Hierarchy>() {
            self.staged_hierarchy_parent = Some(hierarchy.parent);
        }
        if let Some(mobility) = staged_mobility {
            self.staged_mobility = Some(mobility);
        }
    }

    fn reserve_unregistered_component_type<T>(&mut self) -> SceneResult<()>
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        if self.unregistered_component_types[..self.unregistered_component_count]
            .iter()
            .flatten()
            .any(|candidate| candidate.type_id == type_id)
        {
            return Ok(());
        }
        let Some(component_slot) = self
            .unregistered_component_types
            .get_mut(self.unregistered_component_count)
        else {
            return Err(SceneError::BundleTypeReservationLimitExceeded {
                limit: MAX_BUNDLE_COMPONENT_TYPES,
            });
        };
        *component_slot = Some(UnregisteredBundleComponentType {
            type_id,
            register_component_id: register_component_id::<T>,
        });
        self.unregistered_component_count += 1;
        Ok(())
    }

    fn staged_component_id<T>(&mut self) -> SceneResult<ComponentId>
    where
        T: Component,
    {
        if let Some(component_id) = self.world.registered_component_id::<T>() {
            return Ok(component_id);
        }

        let type_id = TypeId::of::<T>();
        self.reserve_unregistered_component_type::<T>()?;
        let Some(index) = self.unregistered_component_types[..self.unregistered_component_count]
            .iter()
            .flatten()
            .position(|candidate| candidate.type_id == type_id)
        else {
            return Err(SceneError::BundleTransactionInvariant {
                reason: "staged component type is not reserved",
            });
        };
        Ok(ComponentId::new(
            self.world.component_registry.descriptors().len() + index,
        ))
    }

    pub(super) fn materialize_reserved_component_types(&mut self) {
        let descriptor_start = self.world.component_registry.descriptors().len();
        for index in 0..self.unregistered_component_count {
            let reservation = self.unregistered_component_types[index]
                .expect("validated component type reservation must exist");
            let component_id = (reservation.register_component_id)(&mut *self.world);
            debug_assert_eq!(component_id, ComponentId::new(descriptor_start + index));
        }
    }
}
