use crate::scene::components::{
    ActiveInHierarchy, ActiveSelf, AmbientLight, AnimationGraphPlayerComponent,
    AnimationPlayerComponent, AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, Mesh2dComponent, MeshRenderer, Mobility, Name,
    PointLight, PostProcessSettingsComponent, PostProcessVolumeComponent, RectLight,
    RenderLayerMask, RigidBodyComponent, SpotLight, Sprite2dComponent, WorldMatrix,
};
use crate::scene::ecs::Component;
use crate::scene::EntityId;

use crate::scene::{SceneResult, World};

use super::super::transform_validation::validate_transform_for_write;

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
    Sprite2dComponent,
    Mesh2dComponent,
    RigidBodyComponent,
    ColliderComponent,
    JointComponent,
    AnimationSkeletonComponent,
    AnimationPlayerComponent,
    AnimationSequencePlayerComponent,
    AnimationGraphPlayerComponent,
    AnimationStateMachinePlayerComponent,
    AmbientLight,
    DirectionalLight,
    PointLight,
    RectLight,
    SpotLight,
    PostProcessSettingsComponent,
    PostProcessVolumeComponent,
    Mobility,
);

trait FixedSceneComponent: Component {
    fn insert_fixed(world: &mut World, entity: EntityId, component: &Self);
}

macro_rules! fixed_component_map {
    ($ty:ty, $field:ident) => {
        impl FixedSceneComponent for $ty {
            fn insert_fixed(world: &mut World, entity: EntityId, component: &Self) {
                world.$field.insert(entity, component.clone());
            }
        }
    };
}

fixed_component_map!(Name, names);
fixed_component_map!(Hierarchy, hierarchy);
fixed_component_map!(ActiveSelf, active_self);
fixed_component_map!(RenderLayerMask, render_layer_masks);
fixed_component_map!(CameraComponent, cameras);
fixed_component_map!(MeshRenderer, mesh_renderers);
fixed_component_map!(Sprite2dComponent, sprite_2d);
fixed_component_map!(Mesh2dComponent, mesh_2d);
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
fixed_component_map!(AmbientLight, ambient_lights);
fixed_component_map!(DirectionalLight, directional_lights);
fixed_component_map!(PointLight, point_lights);
fixed_component_map!(RectLight, rect_lights);
fixed_component_map!(SpotLight, spot_lights);
fixed_component_map!(PostProcessSettingsComponent, post_process_settings);
fixed_component_map!(PostProcessVolumeComponent, post_process_volumes);
fixed_component_map!(Mobility, mobility);

impl FixedSceneComponent for LocalTransform {
    fn insert_fixed(world: &mut World, entity: EntityId, component: &Self) {
        world.local_transforms.insert(entity, *component);
    }
}

impl World {
    pub(super) fn validate_fixed_component<T>(
        &self,
        entity: EntityId,
        component: &T,
    ) -> SceneResult<()>
    where
        T: Component,
    {
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<LocalTransform>()
        {
            return validate_transform_for_write(entity, component.transform);
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<Mobility>() {
            return self.validate_mobility_change(entity, *component);
        }
        Ok(())
    }

    pub(super) fn insert_fixed_component<T>(
        &mut self,
        entity: EntityId,
        component: &T,
    ) -> SceneResult<()>
    where
        T: Component,
    {
        self.validate_fixed_component(entity, component)?;
        self.insert_prevalidated_fixed_component(entity, component);
        Ok(())
    }

    pub(super) fn insert_prevalidated_fixed_component<T>(&mut self, entity: EntityId, component: &T)
    where
        T: Component,
    {
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<Name>() {
            Name::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<Hierarchy>() {
            Hierarchy::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<LocalTransform>()
        {
            self.local_transforms.insert(entity, *component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<ActiveSelf>() {
            ActiveSelf::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<RenderLayerMask>()
        {
            RenderLayerMask::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<CameraComponent>()
        {
            CameraComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<MeshRenderer>() {
            MeshRenderer::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<Sprite2dComponent>()
        {
            Sprite2dComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<Mesh2dComponent>()
        {
            Mesh2dComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<RigidBodyComponent>()
        {
            RigidBodyComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<ColliderComponent>()
        {
            ColliderComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<JointComponent>()
        {
            JointComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationSkeletonComponent>()
        {
            AnimationSkeletonComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationPlayerComponent>()
        {
            AnimationPlayerComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationSequencePlayerComponent>()
        {
            AnimationSequencePlayerComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationGraphPlayerComponent>()
        {
            AnimationGraphPlayerComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<AnimationStateMachinePlayerComponent>()
        {
            AnimationStateMachinePlayerComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<AmbientLight>() {
            AmbientLight::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<DirectionalLight>()
        {
            DirectionalLight::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<PointLight>() {
            PointLight::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<RectLight>() {
            RectLight::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<SpotLight>() {
            SpotLight::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<PostProcessSettingsComponent>()
        {
            PostProcessSettingsComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) =
            (component as &dyn std::any::Any).downcast_ref::<PostProcessVolumeComponent>()
        {
            PostProcessVolumeComponent::insert_fixed(self, entity, component);
            return;
        }
        if let Some(component) = (component as &dyn std::any::Any).downcast_ref::<Mobility>() {
            Mobility::insert_fixed(self, entity, component);
        }
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
        } else if type_id == std::any::TypeId::of::<ActiveSelf>() {
            self.active_self.remove(&entity).map(cast_fixed_component)
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
        } else if type_id == std::any::TypeId::of::<Sprite2dComponent>() {
            self.sprite_2d.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<Mesh2dComponent>() {
            self.mesh_2d.remove(&entity).map(cast_fixed_component)
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
        } else if type_id == std::any::TypeId::of::<AmbientLight>() {
            self.ambient_lights
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<DirectionalLight>() {
            self.directional_lights
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<PointLight>() {
            self.point_lights.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<RectLight>() {
            self.rect_lights.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<SpotLight>() {
            self.spot_lights.remove(&entity).map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<PostProcessSettingsComponent>() {
            self.post_process_settings
                .remove(&entity)
                .map(cast_fixed_component)
        } else if type_id == std::any::TypeId::of::<PostProcessVolumeComponent>() {
            self.post_process_volumes
                .remove(&entity)
                .map(cast_fixed_component)
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
        } else if type_id == std::any::TypeId::of::<ActiveSelf>() {
            self.active_self.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<RenderLayerMask>() {
            self.render_layer_masks
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<CameraComponent>() {
            self.cameras.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<MeshRenderer>() {
            self.mesh_renderers.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<Sprite2dComponent>() {
            self.sprite_2d.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<Mesh2dComponent>() {
            self.mesh_2d.get(&entity).and_then(cast_fixed_ref)
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
        } else if type_id == std::any::TypeId::of::<AmbientLight>() {
            self.ambient_lights.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<DirectionalLight>() {
            self.directional_lights
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<PointLight>() {
            self.point_lights.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<RectLight>() {
            self.rect_lights.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<SpotLight>() {
            self.spot_lights.get(&entity).and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<PostProcessSettingsComponent>() {
            self.post_process_settings
                .get(&entity)
                .and_then(cast_fixed_ref)
        } else if type_id == std::any::TypeId::of::<PostProcessVolumeComponent>() {
            self.post_process_volumes
                .get(&entity)
                .and_then(cast_fixed_ref)
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
        } else if type_id == std::any::TypeId::of::<ActiveSelf>() {
            self.active_self.get_mut(&entity).and_then(cast_fixed_mut)
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
        } else if type_id == std::any::TypeId::of::<Sprite2dComponent>() {
            self.sprite_2d.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<Mesh2dComponent>() {
            self.mesh_2d.get_mut(&entity).and_then(cast_fixed_mut)
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
        } else if type_id == std::any::TypeId::of::<AmbientLight>() {
            self.ambient_lights
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<DirectionalLight>() {
            self.directional_lights
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<PointLight>() {
            self.point_lights.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<RectLight>() {
            self.rect_lights.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<SpotLight>() {
            self.spot_lights.get_mut(&entity).and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<PostProcessSettingsComponent>() {
            self.post_process_settings
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
        } else if type_id == std::any::TypeId::of::<PostProcessVolumeComponent>() {
            self.post_process_volumes
                .get_mut(&entity)
                .and_then(cast_fixed_mut)
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
            || type_id == std::any::TypeId::of::<ActiveSelf>()
            || type_id == std::any::TypeId::of::<RenderLayerMask>()
            || type_id == std::any::TypeId::of::<CameraComponent>()
            || type_id == std::any::TypeId::of::<MeshRenderer>()
            || type_id == std::any::TypeId::of::<Sprite2dComponent>()
            || type_id == std::any::TypeId::of::<Mesh2dComponent>()
            || type_id == std::any::TypeId::of::<RigidBodyComponent>()
            || type_id == std::any::TypeId::of::<ColliderComponent>()
            || type_id == std::any::TypeId::of::<JointComponent>()
            || type_id == std::any::TypeId::of::<AnimationSkeletonComponent>()
            || type_id == std::any::TypeId::of::<AnimationPlayerComponent>()
            || type_id == std::any::TypeId::of::<AnimationSequencePlayerComponent>()
            || type_id == std::any::TypeId::of::<AnimationGraphPlayerComponent>()
            || type_id == std::any::TypeId::of::<AnimationStateMachinePlayerComponent>()
            || type_id == std::any::TypeId::of::<AmbientLight>()
            || type_id == std::any::TypeId::of::<DirectionalLight>()
            || type_id == std::any::TypeId::of::<PointLight>()
            || type_id == std::any::TypeId::of::<RectLight>()
            || type_id == std::any::TypeId::of::<SpotLight>()
            || type_id == std::any::TypeId::of::<PostProcessSettingsComponent>()
            || type_id == std::any::TypeId::of::<PostProcessVolumeComponent>()
            || type_id == std::any::TypeId::of::<Mobility>()
    }

    pub(in crate::scene::world) fn rebuild_fixed_component_presence_for_entity(
        &mut self,
        entity: EntityId,
    ) {
        macro_rules! insert_presence {
            ($field:ident) => {
                if let Some(component) = self.$field.get(&entity).cloned() {
                    self.insert_rebuilt_fixed_component_presence(entity, component);
                }
            };
        }

        insert_presence!(names);
        insert_presence!(hierarchy);
        insert_presence!(local_transforms);
        insert_presence!(active_self);
        insert_presence!(render_layer_masks);
        insert_presence!(cameras);
        insert_presence!(mesh_renderers);
        insert_presence!(sprite_2d);
        insert_presence!(mesh_2d);
        insert_presence!(rigid_bodies);
        insert_presence!(colliders);
        insert_presence!(joints);
        insert_presence!(animation_skeletons);
        insert_presence!(animation_players);
        insert_presence!(animation_sequence_players);
        insert_presence!(animation_graph_players);
        insert_presence!(animation_state_machine_players);
        insert_presence!(ambient_lights);
        insert_presence!(directional_lights);
        insert_presence!(point_lights);
        insert_presence!(rect_lights);
        insert_presence!(spot_lights);
        insert_presence!(post_process_settings);
        insert_presence!(post_process_volumes);
        insert_presence!(mobility);
    }

    /// Populates fixed storage before assigning one final archetype signature.
    pub(in crate::scene::world) fn rebuild_fixed_component_presence_into_final_archetype(
        &mut self,
        entity: EntityId,
    ) {
        self.rebuild_fixed_component_presence_without_final_archetype(entity);
        self.refresh_entity_archetype(entity);
    }

    pub(in crate::scene::world) fn rebuild_fixed_component_presence_without_final_archetype(
        &mut self,
        entity: EntityId,
    ) {
        macro_rules! insert_presence {
            ($field:ident) => {
                if let Some(component) = self.$field.get(&entity).cloned() {
                    self.insert_rebuilt_fixed_component_presence_without_archetype(
                        entity, component,
                    );
                }
            };
        }

        insert_presence!(names);
        insert_presence!(hierarchy);
        insert_presence!(local_transforms);
        insert_presence!(active_self);
        insert_presence!(render_layer_masks);
        insert_presence!(cameras);
        insert_presence!(mesh_renderers);
        insert_presence!(sprite_2d);
        insert_presence!(mesh_2d);
        insert_presence!(rigid_bodies);
        insert_presence!(colliders);
        insert_presence!(joints);
        insert_presence!(animation_skeletons);
        insert_presence!(animation_players);
        insert_presence!(animation_sequence_players);
        insert_presence!(animation_graph_players);
        insert_presence!(animation_state_machine_players);
        insert_presence!(ambient_lights);
        insert_presence!(directional_lights);
        insert_presence!(point_lights);
        insert_presence!(rect_lights);
        insert_presence!(spot_lights);
        insert_presence!(post_process_settings);
        insert_presence!(post_process_volumes);
        insert_presence!(mobility);
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
