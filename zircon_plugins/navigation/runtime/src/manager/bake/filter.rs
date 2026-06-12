use zircon_runtime::core::framework::navigation::{
    NavMeshCollectMode, NavMeshSurfaceDescriptor, NAV_MESH_AGENT_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::Vec3;
use zircon_runtime::scene::components::{NodeKind, SceneNode};
use zircon_runtime::scene::World;

use super::modifier::effective_modifier;

pub(super) fn should_exclude_from_bake(world: &World, entity: u64) -> bool {
    world
        .dynamic_component(entity, NAV_MESH_SURFACE_COMPONENT_TYPE)
        .is_some()
        || world
            .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)
            .is_some()
        || world
            .dynamic_component(entity, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
            .is_some()
        || crate::off_mesh_connections::is_off_mesh_connection_entity(world, entity)
}

pub(super) fn node_matches_surface_collection(
    world: &World,
    node: &SceneNode,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
) -> bool {
    if world.active_in_hierarchy(node.id) == Some(false) {
        return false;
    }
    if !layer_included(world, node, surface) {
        return false;
    }
    match surface.collect_mode {
        NavMeshCollectMode::AllObjects => true,
        NavMeshCollectMode::Hierarchy => surface_entity
            .is_some_and(|root| node.id == root || is_descendant_of(world, node.id, root)),
        NavMeshCollectMode::Volume => {
            node_inside_surface_volume(world, node, surface_entity, surface)
        }
        NavMeshCollectMode::ModifierOnly => {
            effective_modifier(world, node.id, &surface.agent_type).is_some()
        }
    }
}

fn layer_included(world: &World, node: &SceneNode, surface: &NavMeshSurfaceDescriptor) -> bool {
    if surface.include_layers.is_empty() {
        return true;
    }
    let mask = world.render_layer_mask(node.id).unwrap_or_default();
    surface.include_layers.iter().any(|layer| {
        layer.eq_ignore_ascii_case(&node.name)
            || layer.eq_ignore_ascii_case(node_kind_name(&node.kind))
            || layer
                .strip_prefix("layer:")
                .and_then(|value| value.parse::<u32>().ok())
                .is_some_and(|bit| bit < 32 && (mask & (1_u32 << bit)) != 0)
    })
}

fn node_kind_name(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Empty => "empty",
        NodeKind::Camera => "camera",
        NodeKind::Cube => "cube",
        NodeKind::Mesh => "mesh",
        NodeKind::AmbientLight => "ambient_light",
        NodeKind::DirectionalLight => "directional_light",
        NodeKind::PointLight => "point_light",
        NodeKind::RectLight => "rect_light",
        NodeKind::SpotLight => "spot_light",
    }
}

fn is_descendant_of(world: &World, entity: u64, ancestor: u64) -> bool {
    let mut current = world.parent_of(entity);
    while let Some(parent) = current {
        if parent == ancestor {
            return true;
        }
        current = world.parent_of(parent);
    }
    false
}

fn node_inside_surface_volume(
    world: &World,
    node: &SceneNode,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
) -> bool {
    let center = surface_entity
        .and_then(|entity| world.world_transform(entity))
        .map(|transform| {
            transform
                .matrix()
                .transform_point3(Vec3::from_array(surface.volume_center))
        })
        .unwrap_or_else(|| Vec3::from_array(surface.volume_center));
    let half_size = Vec3::from_array(surface.volume_size).abs() * 0.5;
    let position = world
        .world_transform(node.id)
        .map(|transform| transform.translation)
        .unwrap_or(node.transform.translation);
    let delta = (position - center).abs();
    delta.x <= half_size.x && delta.y <= half_size.y && delta.z <= half_size.z
}
