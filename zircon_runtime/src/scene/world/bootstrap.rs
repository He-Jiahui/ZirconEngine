use std::collections::HashMap;

use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::{world::QueryCacheRevision, World};
use crate::scene::components::{
    default_render_layer_mask, ActiveInHierarchy, ActiveSelf, AmbientLight, CameraComponent,
    DirectionalLight, Hierarchy, LocalTransform, MeshRenderer, Mobility, Name, NodeKind,
    PointLight, RectLight, RenderLayerMask, SpotLight,
};
use crate::scene::ecs::Schedule;
use crate::scene::EntityId;

impl World {
    pub fn empty() -> Self {
        let mut world = Self {
            entities: Vec::new(),
            kinds: HashMap::new(),
            names: HashMap::new(),
            hierarchy: HashMap::new(),
            local_transforms: HashMap::new(),
            world_matrices: HashMap::new(),
            cameras: HashMap::new(),
            mesh_renderers: HashMap::new(),
            sprite_2d: HashMap::new(),
            mesh_2d: HashMap::new(),
            ambient_lights: HashMap::new(),
            directional_lights: HashMap::new(),
            point_lights: HashMap::new(),
            rect_lights: HashMap::new(),
            spot_lights: HashMap::new(),
            rigid_bodies: HashMap::new(),
            colliders: HashMap::new(),
            joints: HashMap::new(),
            animation_skeletons: HashMap::new(),
            animation_players: HashMap::new(),
            animation_sequence_players: HashMap::new(),
            animation_graph_players: HashMap::new(),
            animation_state_machine_players: HashMap::new(),
            active_self: HashMap::new(),
            active_in_hierarchy: HashMap::new(),
            render_layer_masks: HashMap::new(),
            mobility: HashMap::new(),
            dynamic_components: HashMap::new(),
            component_types: Default::default(),
            type_registry: Default::default(),
            next_id: 1,
            active_camera: 0,
            schedule: Schedule::default(),
            archetype_index: Default::default(),
            entity_registry: Default::default(),
            component_registry: Default::default(),
            component_storage: Default::default(),
            removed_component_events: Default::default(),
            resource_registry: Default::default(),
            resources: Default::default(),
            events: Default::default(),
            messages: Default::default(),
            observers: Default::default(),
            command_queue: Default::default(),
            query_cache_revision: QueryCacheRevision::default(),
            change_tick: crate::scene::ecs::ChangeTick::INITIAL,
            active_change_tick: None,
            node_cache: Vec::new(),
            derived_state_dirty: Default::default(),
        };
        crate::scene::reflect::register_builtin_reflection(&mut world);
        world
    }

    pub fn new() -> Self {
        let mut world = Self::empty();

        let camera = world.spawn_node(NodeKind::Camera);
        world.active_camera = camera;
        world.spawn_node(NodeKind::DirectionalLight);
        world.spawn_node(NodeKind::Cube);
        world.flush_scene_systems_now();
        world
    }

    pub fn spawn_node(&mut self, kind: NodeKind) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        let default_name = default_name(&kind, self.ordinal_for(kind.clone()));
        self.register_stable_entity(id)
            .expect("spawned scene entity must have a unique stable id");
        self.entities.push(id);
        self.kinds.insert(id, kind.clone());
        self.names.insert(id, Name(default_name));
        self.hierarchy.insert(id, Hierarchy::default());
        self.active_self.insert(id, ActiveSelf::default());
        self.active_in_hierarchy
            .insert(id, ActiveInHierarchy::default());
        self.render_layer_masks
            .insert(id, RenderLayerMask(default_render_layer_mask()));
        self.mobility.insert(id, Mobility::default());

        match kind {
            NodeKind::Camera => {
                self.local_transforms.insert(
                    id,
                    LocalTransform {
                        transform: Transform::looking_at(
                            Vec3::new(3.0, 2.0, 5.0),
                            Vec3::ZERO,
                            Vec3::Y,
                        ),
                    },
                );
                self.cameras.insert(id, CameraComponent::default());
                if self.active_camera == 0 {
                    self.active_camera = id;
                }
            }
            NodeKind::Cube => {
                self.local_transforms.insert(id, LocalTransform::default());
                self.mesh_renderers.insert(id, MeshRenderer::default());
            }
            NodeKind::Mesh => {
                self.local_transforms.insert(id, LocalTransform::default());
                self.mesh_renderers.insert(id, MeshRenderer::default());
            }
            NodeKind::AmbientLight => {
                self.local_transforms.insert(id, LocalTransform::default());
                self.ambient_lights.insert(id, AmbientLight::default());
            }
            NodeKind::DirectionalLight => {
                let mut transform = Transform::default();
                transform.translation = Vec3::new(1.5, 2.0, 1.5);
                transform.rotation = Quat::from_rotation_x(-45.0_f32.to_radians());
                self.local_transforms
                    .insert(id, LocalTransform { transform });
                self.directional_lights
                    .insert(id, DirectionalLight::default());
            }
            NodeKind::PointLight => {
                let mut transform = Transform::default();
                transform.translation = Vec3::new(0.0, 2.0, 0.0);
                self.local_transforms
                    .insert(id, LocalTransform { transform });
                self.point_lights.insert(id, PointLight::default());
            }
            NodeKind::RectLight => {
                let mut transform = Transform::default();
                transform.translation = Vec3::new(0.0, 3.0, 0.0);
                transform.rotation = Quat::from_rotation_x(-90.0_f32.to_radians());
                self.local_transforms
                    .insert(id, LocalTransform { transform });
                self.rect_lights.insert(id, RectLight::default());
            }
            NodeKind::SpotLight => {
                let mut transform = Transform::default();
                transform.translation = Vec3::new(0.0, 4.0, 0.0);
                self.local_transforms
                    .insert(id, LocalTransform { transform });
                self.spot_lights.insert(id, SpotLight::default());
            }
        }

        self.rebuild_fixed_component_presence_for_entity(id);
        self.bump_query_cache_revision();
        self.mark_derived_state_dirty();
        id
    }

    pub fn spawn_mesh_node(
        &mut self,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> EntityId {
        let id = self.spawn_node(NodeKind::Mesh);
        self.insert(id, Name(mesh_display_name(model, self.entities.len())))
            .expect("spawned mesh entity must accept a name component");
        self.insert(id, MeshRenderer::from_handles(model, material))
            .expect("spawned mesh entity must accept a mesh renderer component");
        id
    }
}

fn default_name(kind: &NodeKind, ordinal: usize) -> String {
    match kind {
        NodeKind::Camera => format!("Camera {ordinal}"),
        NodeKind::Cube => format!("Cube {ordinal}"),
        NodeKind::Mesh => format!("Mesh {ordinal}"),
        NodeKind::AmbientLight => format!("Ambient Light {ordinal}"),
        NodeKind::DirectionalLight => format!("Directional Light {ordinal}"),
        NodeKind::PointLight => format!("Point Light {ordinal}"),
        NodeKind::RectLight => format!("Rect Light {ordinal}"),
        NodeKind::SpotLight => format!("Spot Light {ordinal}"),
    }
}

fn mesh_display_name(model: ResourceHandle<ModelMarker>, fallback_ordinal: usize) -> String {
    if model.id() == ResourceId::from_stable_label("builtin://cube") {
        format!("Cube {fallback_ordinal}")
    } else {
        format!("Mesh {fallback_ordinal}")
    }
}
