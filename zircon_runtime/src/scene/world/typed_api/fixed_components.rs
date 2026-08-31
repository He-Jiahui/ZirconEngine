use crate::scene::EntityId;
use crate::scene::components::{
    ActiveInHierarchy, ActiveSelf, AmbientLight, AnimationGraphPlayerComponent,
    AnimationPlayerComponent, AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, ColliderComponent, DirectionalLight,
    Hierarchy, JointComponent, LocalTransform, Mesh2dComponent, MeshRenderer, Mobility, Name,
    PointLight, PostProcessSettingsComponent, PostProcessVolumeComponent, RectLight,
    RenderLayerMask, RigidBodyComponent, SpotLight, Sprite2dComponent, WorldMatrix,
};
use crate::scene::ecs::Component;
use std::collections::HashMap;

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

/// Temporary clone/rebuild transport for runtime-only post-process values.
///
/// These components deliberately have no persistent `World` map. The snapshot
/// exists only while rebuilding the canonical generic storage, so it cannot
/// become a second live owner.
pub(in crate::scene::world) struct RuntimeOnlyPostProcessComponentSnapshot {
    pub(in crate::scene::world) settings: Vec<(EntityId, PostProcessSettingsComponent)>,
    pub(in crate::scene::world) volumes: Vec<(EntityId, PostProcessVolumeComponent)>,
}

/// Temporary persistence transport for the core entity values whose runtime
/// owner is the generic component store.
pub(in crate::scene::world) struct PersistentEntityCoreComponentSnapshot {
    pub(in crate::scene::world) names: HashMap<EntityId, Name>,
    pub(in crate::scene::world) hierarchy: HashMap<EntityId, Hierarchy>,
    pub(in crate::scene::world) local_transforms: HashMap<EntityId, LocalTransform>,
    pub(in crate::scene::world) active_self: HashMap<EntityId, ActiveSelf>,
}

/// Temporary persistence transport for scene-render values whose runtime owner
/// is the generic component store.
pub(in crate::scene::world) struct PersistentSceneRenderComponentSnapshot {
    pub(in crate::scene::world) render_layer_masks: HashMap<EntityId, RenderLayerMask>,
    pub(in crate::scene::world) cameras: HashMap<EntityId, CameraComponent>,
    pub(in crate::scene::world) mesh_renderers: HashMap<EntityId, MeshRenderer>,
    pub(in crate::scene::world) mobility: HashMap<EntityId, Mobility>,
}

/// Temporary persistence transport for physics values whose runtime owner is
/// the generic component store. This exists only across clone, serde and
/// storage-projection rebuild boundaries.
pub(in crate::scene::world) struct PersistentPhysicsComponentSnapshot {
    pub(in crate::scene::world) rigid_bodies: HashMap<EntityId, RigidBodyComponent>,
    pub(in crate::scene::world) colliders: HashMap<EntityId, ColliderComponent>,
    pub(in crate::scene::world) joints: HashMap<EntityId, JointComponent>,
}

/// Temporary persistence transport for light values whose runtime owner is
/// the generic component store.
pub(in crate::scene::world) struct PersistentLightingComponentSnapshot {
    pub(in crate::scene::world) ambient_lights: HashMap<EntityId, AmbientLight>,
    pub(in crate::scene::world) directional_lights: HashMap<EntityId, DirectionalLight>,
    pub(in crate::scene::world) point_lights: HashMap<EntityId, PointLight>,
    pub(in crate::scene::world) rect_lights: HashMap<EntityId, RectLight>,
    pub(in crate::scene::world) spot_lights: HashMap<EntityId, SpotLight>,
}

/// Temporary persistence transport for 2D render values whose runtime owner
/// is the generic component store.
pub(in crate::scene::world) struct PersistentRender2dComponentSnapshot {
    pub(in crate::scene::world) sprite_2d: HashMap<EntityId, Sprite2dComponent>,
    pub(in crate::scene::world) mesh_2d: HashMap<EntityId, Mesh2dComponent>,
}

/// Temporary persistence transport for animation runtime values whose
/// runtime owner is the generic component store.
pub(in crate::scene::world) struct PersistentAnimationRuntimeComponentSnapshot {
    pub(in crate::scene::world) skeletons: HashMap<EntityId, AnimationSkeletonComponent>,
    pub(in crate::scene::world) players: HashMap<EntityId, AnimationPlayerComponent>,
    pub(in crate::scene::world) sequence_players:
        HashMap<EntityId, AnimationSequencePlayerComponent>,
    pub(in crate::scene::world) graph_players: HashMap<EntityId, AnimationGraphPlayerComponent>,
    pub(in crate::scene::world) state_machine_players:
        HashMap<EntityId, AnimationStateMachinePlayerComponent>,
}

impl World {
    pub(in crate::scene::world) fn persistent_scene_render_component_snapshot(
        &self,
    ) -> PersistentSceneRenderComponentSnapshot {
        PersistentSceneRenderComponentSnapshot {
            render_layer_masks: self.persistent_component_snapshot::<RenderLayerMask>(),
            cameras: self.persistent_component_snapshot::<CameraComponent>(),
            mesh_renderers: self.persistent_component_snapshot::<MeshRenderer>(),
            mobility: self.persistent_component_snapshot::<Mobility>(),
        }
    }

    pub(in crate::scene::world) fn persistent_scene_render_component_snapshot_from_serialized_maps(
        render_layer_masks: HashMap<EntityId, RenderLayerMask>,
        cameras: HashMap<EntityId, CameraComponent>,
        mesh_renderers: HashMap<EntityId, MeshRenderer>,
        mobility: HashMap<EntityId, Mobility>,
    ) -> PersistentSceneRenderComponentSnapshot {
        PersistentSceneRenderComponentSnapshot {
            render_layer_masks,
            cameras,
            mesh_renderers,
            mobility,
        }
    }

    pub(in crate::scene::world) fn persistent_entity_core_component_snapshot(
        &self,
    ) -> PersistentEntityCoreComponentSnapshot {
        PersistentEntityCoreComponentSnapshot {
            names: self.persistent_component_snapshot::<Name>(),
            hierarchy: self.persistent_component_snapshot::<Hierarchy>(),
            local_transforms: self.persistent_component_snapshot::<LocalTransform>(),
            active_self: self.persistent_component_snapshot::<ActiveSelf>(),
        }
    }

    pub(in crate::scene::world) fn persistent_entity_core_component_snapshot_from_serialized_maps(
        names: HashMap<EntityId, Name>,
        hierarchy: HashMap<EntityId, Hierarchy>,
        local_transforms: HashMap<EntityId, LocalTransform>,
        active_self: HashMap<EntityId, ActiveSelf>,
    ) -> PersistentEntityCoreComponentSnapshot {
        PersistentEntityCoreComponentSnapshot {
            names,
            hierarchy,
            local_transforms,
            active_self,
        }
    }

    pub(in crate::scene::world) fn runtime_only_post_process_component_snapshot(
        &self,
    ) -> RuntimeOnlyPostProcessComponentSnapshot {
        RuntimeOnlyPostProcessComponentSnapshot {
            settings: self.runtime_only_component_snapshot::<PostProcessSettingsComponent>(),
            volumes: self.runtime_only_component_snapshot::<PostProcessVolumeComponent>(),
        }
    }

    pub(in crate::scene::world) fn persistent_physics_component_snapshot(
        &self,
    ) -> PersistentPhysicsComponentSnapshot {
        PersistentPhysicsComponentSnapshot {
            rigid_bodies: self.persistent_component_snapshot::<RigidBodyComponent>(),
            colliders: self.persistent_component_snapshot::<ColliderComponent>(),
            joints: self.persistent_component_snapshot::<JointComponent>(),
        }
    }

    pub(in crate::scene::world) fn persistent_physics_component_snapshot_from_serialized_maps(
        rigid_bodies: HashMap<EntityId, RigidBodyComponent>,
        colliders: HashMap<EntityId, ColliderComponent>,
        joints: HashMap<EntityId, JointComponent>,
    ) -> PersistentPhysicsComponentSnapshot {
        PersistentPhysicsComponentSnapshot {
            rigid_bodies,
            colliders,
            joints,
        }
    }

    pub(in crate::scene::world) fn persistent_lighting_component_snapshot(
        &self,
    ) -> PersistentLightingComponentSnapshot {
        PersistentLightingComponentSnapshot {
            ambient_lights: self.persistent_component_snapshot::<AmbientLight>(),
            directional_lights: self.persistent_component_snapshot::<DirectionalLight>(),
            point_lights: self.persistent_component_snapshot::<PointLight>(),
            rect_lights: self.persistent_component_snapshot::<RectLight>(),
            spot_lights: self.persistent_component_snapshot::<SpotLight>(),
        }
    }

    pub(in crate::scene::world) fn persistent_lighting_component_snapshot_from_serialized_maps(
        ambient_lights: HashMap<EntityId, AmbientLight>,
        directional_lights: HashMap<EntityId, DirectionalLight>,
        point_lights: HashMap<EntityId, PointLight>,
        rect_lights: HashMap<EntityId, RectLight>,
        spot_lights: HashMap<EntityId, SpotLight>,
    ) -> PersistentLightingComponentSnapshot {
        PersistentLightingComponentSnapshot {
            ambient_lights,
            directional_lights,
            point_lights,
            rect_lights,
            spot_lights,
        }
    }

    pub(in crate::scene::world) fn persistent_render_2d_component_snapshot(
        &self,
    ) -> PersistentRender2dComponentSnapshot {
        PersistentRender2dComponentSnapshot {
            sprite_2d: self.persistent_component_snapshot::<Sprite2dComponent>(),
            mesh_2d: self.persistent_component_snapshot::<Mesh2dComponent>(),
        }
    }

    pub(in crate::scene::world) fn persistent_render_2d_component_snapshot_from_serialized_maps(
        sprite_2d: HashMap<EntityId, Sprite2dComponent>,
        mesh_2d: HashMap<EntityId, Mesh2dComponent>,
    ) -> PersistentRender2dComponentSnapshot {
        PersistentRender2dComponentSnapshot { sprite_2d, mesh_2d }
    }

    pub(in crate::scene::world) fn persistent_animation_runtime_component_snapshot(
        &self,
    ) -> PersistentAnimationRuntimeComponentSnapshot {
        PersistentAnimationRuntimeComponentSnapshot {
            skeletons: self.persistent_component_snapshot::<AnimationSkeletonComponent>(),
            players: self.persistent_component_snapshot::<AnimationPlayerComponent>(),
            sequence_players: self
                .persistent_component_snapshot::<AnimationSequencePlayerComponent>(),
            graph_players: self.persistent_component_snapshot::<AnimationGraphPlayerComponent>(),
            state_machine_players: self
                .persistent_component_snapshot::<AnimationStateMachinePlayerComponent>(),
        }
    }

    pub(in crate::scene::world) fn persistent_animation_runtime_component_snapshot_from_serialized_maps(
        skeletons: HashMap<EntityId, AnimationSkeletonComponent>,
        players: HashMap<EntityId, AnimationPlayerComponent>,
        sequence_players: HashMap<EntityId, AnimationSequencePlayerComponent>,
        graph_players: HashMap<EntityId, AnimationGraphPlayerComponent>,
        state_machine_players: HashMap<EntityId, AnimationStateMachinePlayerComponent>,
    ) -> PersistentAnimationRuntimeComponentSnapshot {
        PersistentAnimationRuntimeComponentSnapshot {
            skeletons,
            players,
            sequence_players,
            graph_players,
            state_machine_players,
        }
    }

    fn runtime_only_component_snapshot<T>(&self) -> Vec<(EntityId, T)>
    where
        T: Component + Clone,
    {
        let Some(component_id) = self.registered_component_id::<T>() else {
            return Vec::new();
        };

        let mut snapshot = Vec::with_capacity(self.archetype_index.component_len(component_id));
        self.archetype_index
            .for_each_table_component::<T>(component_id, |entity, component| {
                snapshot.push((entity, component.clone()));
            });
        snapshot
    }

    fn persistent_component_snapshot<T>(&self) -> HashMap<EntityId, T>
    where
        T: Component + Clone,
    {
        let Some(component_id) = self.registered_component_id::<T>() else {
            return HashMap::new();
        };

        let mut snapshot = HashMap::with_capacity(self.archetype_index.component_len(component_id));
        self.archetype_index
            .for_each_table_component::<T>(component_id, |entity, component| {
                snapshot.insert(entity, component.clone());
            });
        snapshot
    }

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
}
