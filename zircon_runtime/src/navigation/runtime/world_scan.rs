use crate::core::framework::navigation::{
    NavMeshAgentDescriptor, NavMeshObstacleDescriptor, NavMeshObstacleShape,
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use crate::core::math::{Real, Vec3};
use crate::scene::World;

#[derive(Clone, Debug)]
pub(super) struct RuntimeObstacle {
    pub(super) entity: u64,
    pub(super) center: Vec3,
    pub(super) radius: Real,
    pub(super) avoidance_enabled: bool,
}

pub(super) fn collect_agents(world: &World) -> Vec<(u64, NavMeshAgentDescriptor)> {
    world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_AGENT_COMPONENT_TYPE)?;
            serde_json::from_value::<NavMeshAgentDescriptor>(value.clone())
                .ok()
                .map(|agent| (node.id, agent))
        })
        .collect()
}

pub(super) fn collect_agent(world: &World, entity: u64) -> Option<NavMeshAgentDescriptor> {
    let value = world.dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)?;
    serde_json::from_value::<NavMeshAgentDescriptor>(value.clone()).ok()
}

pub(super) fn collect_agent_positions(
    world: &World,
    agents: &[(u64, NavMeshAgentDescriptor)],
) -> Vec<(u64, Vec3, Real)> {
    agents
        .iter()
        .filter_map(|(entity, agent)| {
            world
                .world_transform(*entity)
                .map(|transform| (*entity, transform.translation, agent.radius.max(0.05)))
        })
        .collect()
}

pub(super) fn collect_obstacles(world: &World) -> Vec<RuntimeObstacle> {
    world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)?;
            let obstacle =
                serde_json::from_value::<NavMeshObstacleDescriptor>(value.clone()).ok()?;
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
            Some(RuntimeObstacle {
                entity: node.id,
                center,
                radius,
                avoidance_enabled: obstacle.avoidance_enabled,
            })
        })
        .collect()
}
