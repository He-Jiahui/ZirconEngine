use crate::scene::components::{
    ActiveInHierarchy, ActiveSelf, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, MeshRenderer, Mobility, Name, PointLight,
    RenderLayerMask, RigidBodyComponent, SpotLight, WorldMatrix,
};
use crate::scene::ecs::Component;
use crate::scene::EntityId;

use crate::scene::World;

macro_rules! impl_component_for_scene_type {
    ($($ty:ty),* $(,)?) => {
        $(impl Component for $ty {})*
    };
}

impl_component_for_scene_type!(
    Name,
    Hierarchy,
    LocalTransform,
    WorldMatrix,
    ActiveSelf,
    ActiveInHierarchy,
    RenderLayerMask,
    CameraComponent,
    MeshRenderer,
    RigidBodyComponent,
    ColliderComponent,
    JointComponent,
    AnimationSkeletonComponent,
    AnimationPlayerComponent,
    AnimationSequencePlayerComponent,
    AnimationGraphPlayerComponent,
    AnimationStateMachinePlayerComponent,
    DirectionalLight,
    PointLight,
    SpotLight,
    Mobility,
);

trait FixedSceneComponent: Component {
    fn insert_fixed(world: &mut World, entity: EntityId, component: &Self) -> Result<(), String>;
}

macro_rules! fixed_component_map {
    ($ty:ty, $field:ident) => {
        impl FixedSceneComponent for $ty {
            fn insert_fixed(
                world: &mut World,
                entity: EntityId,
                component: &Self,
            ) -> Result<(), String> {
                world.$field.insert(entity, component.clone());
                Ok(())
            }
        }
    };
}

fixed_component_map!(Name, names);
fixed_component_map!(Hierarchy, hierarchy);
fixed_component_map!(LocalTransform, local_transforms);
fixed_component_map!(WorldMatrix, world_matrices);
fixed_component_map!(ActiveSelf, active_self);
fixed_component_map!(ActiveInHierarchy, active_in_hierarchy);
fixed_component_map!(RenderLayerMask, render_layer_masks);
fixed_component_map!(CameraComponent, cameras);
fixed_component_map!(MeshRenderer, mesh_renderers);
fixed_component_map!(RigidBodyComponent, rigid_bodies);
fixed_component_map!(ColliderComponent, colliders);
fixed_component_map!(JointComponent, joints);
fixed_component_map!(AnimationSkeletonComponent, animation_skeletons);
fixed_component_map!(AnimationPlayerComponent, animation_players);
fixed_component_map!(AnimationSequencePlayerComponent, animation_sequence_players);
fixed_component_map!(AnimationGraphPlayerComponent, animation_graph_players);
fixed_component_map!(
    AnimationStateMachinePlayerComponent,
    animation_state_machine_players
);
fixed_component_map!(DirectionalLight, directional_lights);
fixed_component_map!(PointLight, point_lights);
fixed_component_map!(SpotLight, spot_lights);
fixed_component_map!(Mobility, mobility);

impl World {
    pub(super) fn insert_fixed_component<T>(
        &mut self,
        entity: EntityId,
        component: &T,
    ) -> Result<(), String>
    where
        T: Component,
    {
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<Name>() {
            return Name::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<Hierarchy>() {
            return Hierarchy::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<LocalTransform>()
        {
            return LocalTransform::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<WorldMatrix>() {
            return WorldMatrix::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<ActiveSelf>() {
            return ActiveSelf::insert_fixed(self, entity, component);
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<ActiveInHierarchy>()
        {
            return ActiveInHierarchy::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<RenderLayerMask>()
        {
            return RenderLayerMask::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<CameraComponent>()
        {
            return CameraComponent::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<MeshRenderer>() {
            return MeshRenderer::insert_fixed(self, entity, component);
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<RigidBodyComponent>()
        {
            return RigidBodyComponent::insert_fixed(self, entity, component);
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<ColliderComponent>()
        {
            return ColliderComponent::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<JointComponent>()
        {
            return JointComponent::insert_fixed(self, entity, component);
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationSkeletonComponent>()
        {
            return AnimationSkeletonComponent::insert_fixed(self, entity, component);
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationPlayerComponent>()
        {
            return AnimationPlayerComponent::insert_fixed(self, entity, component);
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationSequencePlayerComponent>()
        {
            return AnimationSequencePlayerComponent::insert_fixed(self, entity, component);
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationGraphPlayerComponent>()
        {
            return AnimationGraphPlayerComponent::insert_fixed(self, entity, component);
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationStateMachinePlayerComponent>()
        {
            return AnimationStateMachinePlayerComponent::insert_fixed(self, entity, component);
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<DirectionalLight>()
        {
            return DirectionalLight::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<PointLight>() {
            return PointLight::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<SpotLight>() {
            return SpotLight::insert_fixed(self, entity, component);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<Mobility>() {
            self.validate_mobility_change(entity, *component)?;
            return Mobility::insert_fixed(self, entity, component);
        }
        Ok(())
    }

    pub(super) fn remove_fixed_component_value<T>(&mut self, entity: EntityId) -> Option<T>
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        if type_id == std::any::TypeId::of::<Name>() {
            self.names.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<Hierarchy>() {
            self.hierarchy.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<LocalTransform>() {
            self.local_transforms
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<WorldMatrix>() {
            self.world_matrices
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<ActiveSelf>() {
            self.active_self.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<ActiveInHierarchy>() {
            self.active_in_hierarchy
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<RenderLayerMask>() {
            self.render_layer_masks
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<CameraComponent>() {
            self.cameras.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<MeshRenderer>() {
            self.mesh_renderers
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<RigidBodyComponent>() {
            self.rigid_bodies.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<ColliderComponent>() {
            self.colliders.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<JointComponent>() {
            self.joints.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<AnimationSkeletonComponent>() {
            self.animation_skeletons
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<AnimationPlayerComponent>() {
            self.animation_players
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<AnimationSequencePlayerComponent>() {
            self.animation_sequence_players
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<AnimationGraphPlayerComponent>() {
            self.animation_graph_players
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<AnimationStateMachinePlayerComponent>() {
            self.animation_state_machine_players
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<DirectionalLight>() {
            self.directional_lights
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<PointLight>() {
            self.point_lights.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<SpotLight>() {
            self.spot_lights.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<Mobility>() {
            self.mobility.remove(&entity).map(cast_fixed_component)
        } else {
            None
        }
    }

    pub(super) fn fixed_component_ref<T>(&self, entity: EntityId) -> Option<&T>
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        if type_id == std::any::TypeId::of::<Name>() {
            self.names.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<Hierarchy>() {
            self.hierarchy.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<LocalTransform>() {
            self.local_transforms.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<WorldMatrix>() {
            self.world_matrices.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<ActiveSelf>() {
            self.active_self.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<ActiveInHierarchy>() {
            self.active_in_hierarchy
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<RenderLayerMask>() {
            self.render_layer_masks
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<CameraComponent>() {
            self.cameras.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<MeshRenderer>() {
            self.mesh_renderers.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<RigidBodyComponent>() {
            self.rigid_bodies.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<ColliderComponent>() {
            self.colliders.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<JointComponent>() {
            self.joints.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<AnimationSkeletonComponent>() {
            self.animation_skeletons
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<AnimationPlayerComponent>() {
            self.animation_players.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<AnimationSequencePlayerComponent>() {
            self.animation_sequence_players
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<AnimationGraphPlayerComponent>() {
            self.animation_graph_players
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<AnimationStateMachinePlayerComponent>() {
            self.animation_state_machine_players
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<DirectionalLight>() {
            self.directional_lights
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<PointLight>() {
            self.point_lights.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<SpotLight>() {
            self.spot_lights.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<Mobility>() {
            self.mobility.get(&entity).and_then(cast_fixed_ref)
        } else {
            None
        }
    }

    pub(super) fn fixed_component_mut<T>(&mut self, entity: EntityId) -> Option<&mut T>
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        if type_id == std::any::TypeId::of::<Name>() {
            self.names.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<Hierarchy>() {
            self.hierarchy.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<LocalTransform>() {
            self.local_transforms
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<WorldMatrix>() {
            self.world_matrices
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<ActiveSelf>() {
            self.active_self.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<ActiveInHierarchy>() {
            self.active_in_hierarchy
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<RenderLayerMask>() {
            self.render_layer_masks
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<CameraComponent>() {
            self.cameras.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<MeshRenderer>() {
            self.mesh_renderers
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<RigidBodyComponent>() {
            self.rigid_bodies.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<ColliderComponent>() {
            self.colliders.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<JointComponent>() {
            self.joints.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<AnimationSkeletonComponent>() {
            self.animation_skeletons
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<AnimationPlayerComponent>() {
            self.animation_players
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<AnimationSequencePlayerComponent>() {
            self.animation_sequence_players
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<AnimationGraphPlayerComponent>() {
            self.animation_graph_players
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<AnimationStateMachinePlayerComponent>() {
            self.animation_state_machine_players
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<DirectionalLight>() {
            self.directional_lights
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<PointLight>() {
            self.point_lights.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<SpotLight>() {
            self.spot_lights.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<Mobility>() {
            self.mobility.get_mut(&entity).and_then(cast_fixed_mut)
        } else {
            None
        }
    }

    pub(super) fn is_fixed_component_type<T>(&self) -> bool
    where
        T: Component,
    {
        let type_id = std::any::TypeId::of::<T>();
        type_id == std::any::TypeId::of::<Name>()
            || type_id == std::any::TypeId::of::<Hierarchy>()
            || type_id == std::any::TypeId::of::<LocalTransform>()
            || type_id == std::any::TypeId::of::<WorldMatrix>()
            || type_id == std::any::TypeId::of::<ActiveSelf>()
            || type_id == std::any::TypeId::of::<ActiveInHierarchy>()
            || type_id == std::any::TypeId::of::<RenderLayerMask>()
            || type_id == std::any::TypeId::of::<CameraComponent>()
            || type_id == std::any::TypeId::of::<MeshRenderer>()
            || type_id == std::any::TypeId::of::<RigidBodyComponent>()
            || type_id == std::any::TypeId::of::<ColliderComponent>()
            || type_id == std::any::TypeId::of::<JointComponent>()
            || type_id == std::any::TypeId::of::<AnimationSkeletonComponent>()
            || type_id == std::any::TypeId::of::<AnimationPlayerComponent>()
            || type_id == std::any::TypeId::of::<AnimationSequencePlayerComponent>()
            || type_id == std::any::TypeId::of::<AnimationGraphPlayerComponent>()
            || type_id == std::any::TypeId::of::<AnimationStateMachinePlayerComponent>()
            || type_id == std::any::TypeId::of::<DirectionalLight>()
            || type_id == std::any::TypeId::of::<PointLight>()
            || type_id == std::any::TypeId::of::<SpotLight>()
            || type_id == std::any::TypeId::of::<Mobility>()
    }

    pub(in crate::scene::world) fn rebuild_fixed_component_presence_for_entity(
        &mut self,
        entity: EntityId,
    ) {
        if let Some(component) = self.names.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.hierarchy.get(&entity).copied() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.local_transforms.get(&entity).copied() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.world_matrices.get(&entity).copied() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.active_self.get(&entity).copied() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.active_in_hierarchy.get(&entity).copied() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.render_layer_masks.get(&entity).copied() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.cameras.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.mesh_renderers.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.rigid_bodies.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.colliders.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.joints.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.animation_skeletons.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.animation_players.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.animation_sequence_players.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.animation_graph_players.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.animation_state_machine_players.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.directional_lights.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.point_lights.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.spot_lights.get(&entity).cloned() {
            let _ = self.insert(entity, component);
        }
        if let Some(component) = self.mobility.get(&entity).copied() {
            let _ = self.insert(entity, component);
        }
    }
}

fn cast_fixed_component<T, U>(component: U) -> T
where
    T: Component,
    U: std::any::Any,
{
    match (Box::new(component) as Box<dyn std::any::Any>).downcast::<T>() {
        Ok(component) => *component,
        Err(_) => panic!("fixed component type dispatch must match concrete component"),
    }
}

fn cast_fixed_ref<T, U>(component: &U) -> Option<&T>
where
    T: Component,
    U: std::any::Any,
{
    (component as &dyn std::any::Any).downcast_ref::<T>()
}

fn cast_fixed_mut<T, U>(component: &mut U) -> Option<&mut T>
where
    T: Component,
    U: std::any::Any,
{
    (component as &mut dyn std::any::Any).downcast_mut::<T>()
}
