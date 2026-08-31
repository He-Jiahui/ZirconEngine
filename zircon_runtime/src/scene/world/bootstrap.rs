use std::collections::HashMap;

use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::{
    World,
    compiled_binding::SceneBindingGenerations,
    entity_id_allocator::EntityIdAllocator,
    generation::{LifecycleVisibilityRevision, WorldGeneration},
};
use crate::scene::EntityId;
use crate::scene::components::{
    ActiveSelf, AmbientLight, CameraComponent, DirectionalLight, MeshRenderer, Mobility, Name,
    NodeKind, NodeRecord, PointLight, RectLight, SpotLight, default_render_layer_mask,
};
use crate::scene::ecs::Schedule;
use zircon_runtime_interface::world_sync::WorldFact;

impl World {
    pub fn empty() -> Self {
        let mut world = Self {
            entities: Vec::new(),
            entity_dense_rows: HashMap::new(),
            kinds: HashMap::new(),
            node_kind_ordinals: Default::default(),
            dynamic_components: HashMap::new(),
            dynamic_component_generations: HashMap::new(),
            component_types: Default::default(),
            type_registry: Default::default(),
            vm_catalog_type_paths: Default::default(),
            vm_dynamic_type_paths: Default::default(),
            entity_id_allocator: EntityIdAllocator::default(),
            active_camera: 0,
            schedule: Schedule::default(),
            archetype_index: Default::default(),
            stable_query_order: Default::default(),
            hierarchy_mutation_index: Default::default(),
            entity_registry: Default::default(),
            component_registry: Default::default(),
            component_storage: Default::default(),
            removed_component_events: Default::default(),
            resource_registry: Default::default(),
            resources: Default::default(),
            events: Default::default(),
            event_mirrors: Default::default(),
            messages: Default::default(),
            observers: Default::default(),
            world_sync_subscriptions: Default::default(),
            staged_lifecycle_events: Vec::new(),
            record_staged_lifecycle_events: false,
            command_queue: Default::default(),
            deferred_command_errors: Vec::new(),
            deferred_direct_spawn_ordinal: 0,
            deferred_direct_system_ordinal: 0,
            deferred_spawn_resolutions: Default::default(),
            published_deferred_spawns: Default::default(),
            ecs_frame_performance_diagnostics: Default::default(),
            archetype_assignment_counter: Default::default(),
            lifecycle_visibility_revision: LifecycleVisibilityRevision::default(),
            world_generation: WorldGeneration::default(),
            scene_binding_generations: SceneBindingGenerations::default(),
            compiled_scene_property_access_diagnostics: Default::default(),
            change_tick: crate::scene::ecs::ChangeTick::INITIAL,
            last_change_tick: crate::scene::ecs::ChangeTick::ZERO,
            active_change_tick: None,
            node_cache: Vec::new(),
            node_cache_rows: HashMap::new(),
            node_cache_topology_generation: 0,
            inspection_artifact_cache: Default::default(),
            derived_state_dirty: Default::default(),
        };
        crate::scene::reflect::register_builtin_reflection(&mut world);
        world
    }

    pub fn new() -> Self {
        let mut world = Self::empty();

        let camera = world
            .spawn_node(NodeKind::Camera)
            .expect("a fresh World must allocate its default camera");
        world.active_camera = camera;
        world
            .spawn_node(NodeKind::DirectionalLight)
            .expect("a fresh World must allocate its default directional light");
        world
            .spawn_node(NodeKind::Cube)
            .expect("a fresh World must allocate its default cube");
        world.flush_scene_systems_now();
        world
    }

    pub fn spawn_node(&mut self, kind: NodeKind) -> crate::scene::SceneResult<EntityId> {
        let id = self.entity_id_allocator.next_available()?;
        let record = self.default_node_record(id, kind);
        self.insert_node_record(record)?;
        Ok(id)
    }

    pub(super) fn default_node_record(&self, id: EntityId, kind: NodeKind) -> NodeRecord {
        let mut record = NodeRecord {
            id,
            name: default_name(&kind, self.ordinal_for(kind)),
            kind,
            parent: None,
            transform: Transform::default(),
            camera: None,
            mesh: None,
            sprite_2d: None,
            mesh_2d: None,
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            active: ActiveSelf::default().0,
            render_layer_mask: default_render_layer_mask(),
            mobility: Mobility::default(),
            rigid_body: None,
            collider: None,
            joint: None,
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
        };

        match kind {
            NodeKind::Empty => {}
            NodeKind::Camera => {
                record.transform =
                    Transform::looking_at(Vec3::new(3.0, 2.0, 5.0), Vec3::ZERO, Vec3::Y);
                record.camera = Some(CameraComponent::default());
            }
            NodeKind::Cube | NodeKind::Mesh => {
                record.mesh = Some(MeshRenderer::default());
            }
            NodeKind::AmbientLight => {
                record.ambient_light = Some(AmbientLight::default());
            }
            NodeKind::DirectionalLight => {
                record.transform.translation = Vec3::new(1.5, 2.0, 1.5);
                record.transform.rotation = Quat::from_rotation_x(-45.0_f32.to_radians());
                record.directional_light = Some(DirectionalLight::default());
            }
            NodeKind::PointLight => {
                record.transform.translation = Vec3::new(0.0, 2.0, 0.0);
                record.point_light = Some(PointLight::default());
            }
            NodeKind::RectLight => {
                record.transform.translation = Vec3::new(0.0, 3.0, 0.0);
                record.transform.rotation = Quat::from_rotation_x(-90.0_f32.to_radians());
                record.rect_light = Some(RectLight::default());
            }
            NodeKind::SpotLight => {
                record.transform.translation = Vec3::new(0.0, 4.0, 0.0);
                record.spot_light = Some(SpotLight::default());
            }
        }
        record
    }

    pub fn spawn_mesh_node(
        &mut self,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> crate::scene::SceneResult<EntityId> {
        let id = self.spawn_node(NodeKind::Mesh)?;
        self.insert(id, Name(mesh_display_name(model, self.entities.len())))?;
        self.insert(id, MeshRenderer::from_handles(model, material))?;
        Ok(id)
    }
}

fn default_name(kind: &NodeKind, ordinal: usize) -> String {
    match kind {
        NodeKind::Empty => format!("Empty {ordinal}"),
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
