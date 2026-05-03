use zircon_plugin_navigation_recast::{RecastNavigationObstacle, RecastNavigationObstacleShape};
use zircon_runtime::core::framework::navigation::{
    NavMeshObstacleDescriptor, NavMeshObstacleShape, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Real, Vec3};
use zircon_runtime::scene::components::{NodeKind, SceneNode};
use zircon_runtime::scene::World;

use crate::component_json::parse_component;

#[derive(Clone, Debug)]
pub(crate) struct RuntimeObstacle {
    pub(crate) entity: u64,
    pub(crate) center: Vec3,
    pub(crate) half_extents: Vec3,
    pub(crate) radius: Real,
    pub(crate) height: Real,
    pub(crate) shape: NavMeshObstacleShape,
    pub(crate) carve: bool,
    pub(crate) avoidance_enabled: bool,
}

pub(crate) fn collect_runtime_obstacles(world: &World) -> Vec<RuntimeObstacle> {
    world
        .nodes()
        .iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)?;
            let obstacle = parse_component::<NavMeshObstacleDescriptor>(value);
            let transform = world.world_transform(node.id).unwrap_or(node.transform);
            let center = transform
                .matrix()
                .transform_point3(Vec3::from_array(obstacle.center));
            let radius = match obstacle.shape {
                NavMeshObstacleShape::Box => {
                    let size = Vec3::from_array(obstacle.size).abs();
                    size.x.max(size.z) * 0.5
                }
                NavMeshObstacleShape::Capsule => obstacle.radius,
            }
            .max(0.05);
            let half_extents = match obstacle.shape {
                NavMeshObstacleShape::Box => Vec3::from_array(obstacle.size).abs() * 0.5,
                NavMeshObstacleShape::Capsule => Vec3::new(
                    obstacle.radius.max(0.05),
                    obstacle.height.max(0.05) * 0.5,
                    obstacle.radius.max(0.05),
                ),
            };
            Some(RuntimeObstacle {
                entity: node.id,
                center,
                half_extents,
                radius,
                height: obstacle.height.max(0.05),
                shape: obstacle.shape,
                carve: obstacle.carve,
                avoidance_enabled: obstacle.avoidance_enabled,
            })
        })
        .collect()
}

pub(crate) fn recast_carving_obstacles(
    obstacles: &[RuntimeObstacle],
) -> Vec<RecastNavigationObstacle> {
    obstacles
        .iter()
        .filter(|obstacle| obstacle.carve)
        .map(|obstacle| match obstacle.shape {
            NavMeshObstacleShape::Box => RecastNavigationObstacle {
                shape: RecastNavigationObstacleShape::Box,
                center: obstacle.center.to_array(),
                half_extents: obstacle.half_extents.to_array(),
                radius: obstacle.radius,
                height: obstacle.height,
            },
            NavMeshObstacleShape::Capsule => RecastNavigationObstacle::cylinder(
                obstacle.center.to_array(),
                obstacle.radius,
                obstacle.height,
            ),
        })
        .collect()
}

pub(crate) fn node_intersects_obstacle(
    world: &World,
    node: &SceneNode,
    obstacles: &[RuntimeObstacle],
) -> bool {
    let position = world
        .world_transform(node.id)
        .map(|transform| transform.translation)
        .unwrap_or(node.transform.translation);
    let node_radius = match node.kind {
        NodeKind::Cube | NodeKind::Mesh => 0.75,
        NodeKind::Camera
        | NodeKind::DirectionalLight
        | NodeKind::PointLight
        | NodeKind::SpotLight => 0.25,
    };
    obstacles.iter().any(|obstacle| {
        obstacle.entity != node.id
            && distance_xz(position, obstacle.center) <= obstacle.radius + node_radius
    })
}

pub(crate) fn distance_xz(left: Vec3, right: Vec3) -> Real {
    let delta = left - right;
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}
