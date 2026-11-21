use std::collections::{hash_map::Entry, HashMap};

use zircon_plugin_navigation_recast::{
    RecastNavigationObstacle, RecastNavigationObstacleShape, RecastTileCache,
    RecastTileCacheObstacleHandle,
};
use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavMeshHandle, NavMeshObstacleDescriptor, NavMeshObstacleShape, NavPathQuery, NavPathResult,
    NavigationError, NavigationErrorKind, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Real, Vec3};
use zircon_runtime::scene::components::{NodeKind, SceneNode};
use zircon_runtime::scene::World;

use crate::component_json::parse_component;
use crate::manager::DefaultNavigationManager;

pub(crate) fn has_obstacle_worlds(manager: &DefaultNavigationManager) -> bool {
    !manager.lock_state().obstacle_worlds.is_empty()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum CarvedLinkPolicy {
    GroundOnly,
    AutomaticOnly,
}

// 查询前同步当前代理连接策略对应的 TileCache；不把首个代理的策略固化给整个网格。
pub(crate) fn find_path_with_runtime_obstacles(
    manager: &DefaultNavigationManager,
    handle: NavMeshHandle,
    query_asset: &NavMeshAsset,
    query: &NavPathQuery,
    obstacles: &[RuntimeObstacle],
) -> Result<NavPathResult, NavigationError> {
    // The tick projects each loaded asset to either no links or its fixed automatic subset.
    let (policy, other) = if query_asset.off_mesh_links.is_empty() {
        (
            CarvedLinkPolicy::GroundOnly,
            CarvedLinkPolicy::AutomaticOnly,
        )
    } else {
        (
            CarvedLinkPolicy::AutomaticOnly,
            CarvedLinkPolicy::GroundOnly,
        )
    };
    let key = (handle, policy);
    let has_carving = obstacles.iter().any(|obstacle| obstacle.carve);
    let mut state = manager.lock_state();
    if !has_carving {
        // Retire both variants. If the selected one was live, keep the final synchronized
        // cache query rather than switching backends on the last carve-removal frame.
        let cached = state.obstacle_worlds.remove(&key);
        state.obstacle_worlds.remove(&(handle, other));
        if let Some(mut obstacle_world) = cached {
            obstacle_world.synchronize(obstacles)?;
            return Ok(obstacle_world.find_path(query));
        }
        drop(state);
        return manager.backend.find_path(query_asset, query);
    }
    let obstacle_world = match state.obstacle_worlds.entry(key) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => entry.insert(NavigationObstacleWorld::from_asset(query_asset)?),
    };
    obstacle_world.synchronize(obstacles)?;
    Ok(obstacle_world.find_path(query))
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Debug)]
pub(crate) struct NavigationObstacleWorld {
    cache: RecastTileCache,
    bindings: HashMap<u64, (RuntimeObstacle, RecastTileCacheObstacleHandle)>,
}

impl NavigationObstacleWorld {
    pub(crate) fn from_asset(asset: &NavMeshAsset) -> Result<Self, NavigationError> {
        let cache = RecastTileCache::from_asset(asset).ok_or_else(|| {
            NavigationError::new(
                NavigationErrorKind::InvalidConfiguration,
                "navmesh cannot initialize a Detour TileCache obstacle world",
            )
        })?;
        Ok(Self {
            cache,
            bindings: HashMap::new(),
        })
    }

    pub(crate) fn synchronize(
        &mut self,
        obstacles: &[RuntimeObstacle],
    ) -> Result<(), NavigationError> {
        let desired = obstacles
            .iter()
            .filter(|obstacle| obstacle.carve)
            .map(|obstacle| (obstacle.entity, obstacle.clone()))
            .collect::<HashMap<_, _>>();
        let removed_or_changed = self
            .bindings
            .iter()
            .filter(|(entity, (current, _))| desired.get(entity) != Some(current))
            .map(|(entity, (_, handle))| (*entity, *handle))
            .collect::<Vec<_>>();
        let mut removed = false;
        for (entity, handle) in removed_or_changed {
            self.cache.remove_obstacle(handle).map_err(obstacle_error)?;
            self.bindings.remove(&entity);
            removed = true;
        }
        // TileCache removal is queued. Flush it before replacement additions so a full cache
        // releases the removed slots before new obstacle refs are allocated.
        if removed {
            self.cache.update().map_err(obstacle_error)?;
        }
        let mut added = false;
        for (entity, obstacle) in desired {
            if self.bindings.contains_key(&entity) {
                continue;
            }
            let handle = self
                .cache
                .add_obstacle(recast_obstacle(&obstacle))
                .ok_or_else(|| obstacle_error("Detour TileCache could not add obstacle"))?;
            self.bindings.insert(entity, (obstacle, handle));
            added = true;
        }
        if added {
            self.cache.update().map_err(obstacle_error)?;
        }
        Ok(())
    }

    pub(crate) fn find_path(&self, query: &NavPathQuery) -> NavPathResult {
        self.cache.find_path(query)
    }
}

fn obstacle_error(message: &'static str) -> NavigationError {
    NavigationError::new(NavigationErrorKind::InvalidConfiguration, message)
}

fn recast_obstacle(obstacle: &RuntimeObstacle) -> RecastNavigationObstacle {
    match obstacle.shape {
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
    }
}

pub(crate) fn collect_runtime_obstacles(world: &World) -> Vec<RuntimeObstacle> {
    // Obstacle carving and avoidance must observe direct dynamic component writes immediately.
    world
        .node_records()
        .into_iter()
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
        NodeKind::Empty => 0.0,
        NodeKind::Camera
        | NodeKind::AmbientLight
        | NodeKind::DirectionalLight
        | NodeKind::PointLight
        | NodeKind::RectLight
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
