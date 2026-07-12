use crate::component_json::parse_component;
use zircon_runtime::asset::{NavMeshLinkAsset, NavMeshLinkCapacity};
use zircon_runtime::core::framework::navigation::{
    NavMeshOffMeshBridgeDescriptor, NavMeshOffMeshLinkDescriptor, MAX_OFF_MESH_BRIDGE_LANES,
    NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Real, Vec3};
use zircon_runtime::scene::World;

pub(crate) fn collect_off_mesh_connections(
    world: &World,
    agent_type: &str,
) -> Vec<NavMeshLinkAsset> {
    let mut links = collect_off_mesh_links(world, agent_type);
    links.extend(collect_off_mesh_bridges(world, agent_type));
    for (index, link) in links.iter_mut().enumerate() {
        link.id = u32::try_from(index + 1).expect("navigation asset link count exceeds u32");
    }
    links
}

pub(crate) fn is_off_mesh_connection_entity(world: &World, entity: u64) -> bool {
    world
        .dynamic_component(entity, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE)
        .is_some()
        || world
            .dynamic_component(entity, NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE)
            .is_some()
}

pub(crate) fn count_off_mesh_links(world: &World) -> usize {
    count_component_instances(world, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE)
}

pub(crate) fn count_off_mesh_bridges(world: &World) -> usize {
    count_component_instances(world, NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE)
}

fn collect_off_mesh_links(world: &World, agent_type: &str) -> Vec<NavMeshLinkAsset> {
    world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE)?;
            let link = parse_component::<NavMeshOffMeshLinkDescriptor>(value);
            if !link.activated || link.agent_type != agent_type {
                return None;
            }
            Some(NavMeshLinkAsset {
                id: 0,
                owner_entity: node.id,
                lane_index: 0,
                capacity: NavMeshLinkCapacity::Unbounded,
                motion: link.motion,
                arc_height: link.arc_height.max(0.0),
                start: link_endpoint_world_position(
                    world,
                    node.id,
                    link.start_entity,
                    link.start_local_point,
                ),
                end: link_endpoint_world_position(
                    world,
                    node.id,
                    link.end_entity,
                    link.end_local_point,
                ),
                width: link.width,
                bidirectional: link.bidirectional,
                area: link.area_type,
                cost_override: link.cost_override,
                traversal_mode: link.traversal_mode,
            })
        })
        .collect()
}

fn collect_off_mesh_bridges(world: &World, agent_type: &str) -> Vec<NavMeshLinkAsset> {
    world
        .node_records()
        .into_iter()
        .flat_map(|node| {
            let Some(value) =
                world.dynamic_component(node.id, NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE)
            else {
                return Vec::new();
            };
            let bridge = parse_component::<NavMeshOffMeshBridgeDescriptor>(value);
            if !bridge.activated || bridge.agent_type != agent_type {
                return Vec::new();
            }
            expand_bridge_lanes(world, node.id, &bridge)
        })
        .collect()
}

fn expand_bridge_lanes(
    world: &World,
    owner: u64,
    bridge: &NavMeshOffMeshBridgeDescriptor,
) -> Vec<NavMeshLinkAsset> {
    let start = Vec3::from_array(link_endpoint_world_position(
        world,
        owner,
        bridge.start_entity,
        bridge.start_local_point,
    ));
    let end = Vec3::from_array(link_endpoint_world_position(
        world,
        owner,
        bridge.end_entity,
        bridge.end_local_point,
    ));
    let lane_count = bridge.lane_count.clamp(1, MAX_OFF_MESH_BRIDGE_LANES) as usize;
    let width = bridge.width.max(0.0);
    let lane_width = if lane_count > 0 {
        width / lane_count as Real
    } else {
        width
    };
    let side = bridge_side_axis(start, end);

    (0..lane_count)
        .map(|lane| {
            let offset = if lane_count == 1 || width <= Real::EPSILON {
                Vec3::ZERO
            } else {
                let center_offset = lane as Real + 0.5 - lane_count as Real * 0.5;
                side * (center_offset * lane_width)
            };
            NavMeshLinkAsset {
                id: 0,
                owner_entity: owner,
                lane_index: lane as u32,
                capacity: NavMeshLinkCapacity::Shared {
                    group: owner,
                    limit: lane_count as u32,
                },
                motion: bridge.motion,
                arc_height: bridge.arc_height.max(0.0),
                start: (start + offset).to_array(),
                end: (end + offset).to_array(),
                width: lane_width,
                bidirectional: bridge.bidirectional,
                area: bridge.area_type,
                cost_override: bridge.cost_override,
                traversal_mode: bridge.traversal_mode,
            }
        })
        .collect()
}

fn bridge_side_axis(start: Vec3, end: Vec3) -> Vec3 {
    let direction = end - start;
    let horizontal = Vec3::new(direction.x, 0.0, direction.z);
    if horizontal.length_squared() <= Real::EPSILON {
        Vec3::X
    } else {
        Vec3::new(-horizontal.z, 0.0, horizontal.x).normalize_or_zero()
    }
}

fn link_endpoint_world_position(
    world: &World,
    owner: u64,
    endpoint_entity: Option<u64>,
    local_point: [Real; 3],
) -> [Real; 3] {
    let transform_entity = endpoint_entity.unwrap_or(owner);
    world
        .world_transform(transform_entity)
        .unwrap_or_default()
        .matrix()
        .transform_point3(Vec3::from_array(local_point))
        .to_array()
}

fn count_component_instances(world: &World, component_type: &str) -> usize {
    world
        .node_records()
        .into_iter()
        .filter(|node| world.dynamic_component(node.id, component_type).is_some())
        .count()
}
