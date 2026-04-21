use std::collections::HashMap;

use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::World;
use crate::scene::components::{
    default_render_layer_mask, ActiveInHierarchy, ActiveSelf, CameraComponent, DirectionalLight,
    Hierarchy, LocalTransform, MeshRenderer, Mobility, Name, NodeKind, PointLight, RenderLayerMask,
    Schedule, SpotLight,
};
use crate::scene::EntityId;

impl World {
    pub(crate) fn empty() -> Self {
        Self {
            entities: Vec::new(),
            kinds: HashMap::new(),
            names: HashMap::new(),
            hierarchy: HashMap::new(),
            local_transforms: HashMap::new(),
            world_matrices: HashMap::new(),
            cameras: HashMap::new(),
            mesh_renderers: HashMap::new(),
            directional_lights: HashMap::new(),
            point_lights: HashMap::new(),
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
            next_id: 1,
            active_camera: 0,
            schedule: Schedule::default(),
            node_cache: Vec::new(),
        }
    }

    pub fn new() -> Self {
        let mut world = Self::empty();

        let camera = world.spawn_node(NodeKind::Camera);
        world.active_camera = camera;
        world.spawn_node(NodeKind::DirectionalLight);
        world.spawn_node(NodeKind::Cube);
        world
    }

    pub fn spawn_node(&mut self, kind: NodeKind) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.push(id);
        self.kinds.insert(id, kind.clone());
        self.names.insert(
            id,
            Name(default_name(&kind, self.ordinal_for(kind.clone()))),
        );
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
            NodeKind::SpotLight => {
                let mut transform = Transform::default();
                transform.translation = Vec3::new(0.0, 4.0, 0.0);
                self.local_transforms
                    .insert(id, LocalTransform { transform });
                self.spot_lights.insert(id, SpotLight::default());
            }
        }

        self.rebuild_derived_state();
        id
    }

    pub fn spawn_mesh_node(
        &mut self,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> EntityId {
        let id = self.spawn_node(NodeKind::Mesh);
        self.names
            .insert(id, Name(mesh_display_name(model, self.entities.len())));
        self.mesh_renderers
            .insert(id, MeshRenderer::from_handles(model, material));
        self.rebuild_derived_state();
        id
    }
}

fn default_name(kind: &NodeKind, ordinal: usize) -> String {
    match kind {
        NodeKind::Camera => format!("Camera {ordinal}"),
        NodeKind::Cube => format!("Cube {ordinal}"),
        NodeKind::Mesh => format!("Mesh {ordinal}"),
        NodeKind::DirectionalLight => format!("Directional Light {ordinal}"),
        NodeKind::PointLight => format!("Point Light {ordinal}"),
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
