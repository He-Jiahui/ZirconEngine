use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Mutex;

use crate::asset::{NavMeshAsset, NavigationSettingsAsset};
use crate::core::framework::navigation::{
    NavAgentTickReport, NavMeshAgentDescriptor, NavMeshBakeReport, NavMeshBakeRequest,
    NavMeshHandle, NavMeshObstacleDescriptor, NavMeshObstacleShape, NavPathPoint, NavPathQuery,
    NavPathResult, NavPathStatus, NavRaycastQuery, NavRaycastResult, NavSampleHit, NavSampleQuery,
    NavigationError, NavigationErrorKind, NavigationManager, NavigationRuntimeStats, AREA_WALKABLE,
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use crate::core::math::{Real, Transform, Vec3};
use crate::scene::World;

#[derive(Debug)]
pub struct BuiltinNavigationManager {
    state: Mutex<BuiltinNavigationState>,
}

impl BuiltinNavigationManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(BuiltinNavigationState::default()),
        }
    }
}

impl Default for BuiltinNavigationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationManager for BuiltinNavigationManager {
    fn bake_surface(
        &self,
        _world: &World,
        _request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError> {
        Err(NavigationError::new(
            NavigationErrorKind::BackendFailure,
            "built-in navigation can load baked navmeshes but does not bake surfaces",
        ))
    }

    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError> {
        if asset.is_empty() {
            return Err(NavigationError::missing_nav_mesh(
                "cannot load an empty navmesh asset",
            ));
        }
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        let handle = NavMeshHandle(state.next_handle);
        state.next_handle += 1;
        state.loaded.insert(handle, BakedNavMesh::new(asset));
        state.stats.loaded_nav_meshes = state.loaded.len();
        Ok(handle)
    }

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError> {
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        state.settings = settings;
        Ok(())
    }

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError> {
        let state = self.state.lock().expect("navigation state lock poisoned");
        let mesh = state.selected_mesh(query.nav_mesh)?;
        Ok(mesh.find_path(query))
    }

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        let state = self.state.lock().expect("navigation state lock poisoned");
        let mesh = state.selected_mesh(query.nav_mesh)?;
        Ok(mesh.sample_position(query))
    }

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError> {
        let state = self.state.lock().expect("navigation state lock poisoned");
        let mesh = state.selected_mesh(query.nav_mesh)?;
        Ok(mesh.raycast(query))
    }

    fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        if dt_seconds <= 0.0 || !dt_seconds.is_finite() {
            return Ok(NavAgentTickReport::default());
        }

        let agents = collect_agents(world);
        let agent_positions = collect_agent_positions(world, &agents);
        let obstacles = collect_obstacles(world);
        let mut report = NavAgentTickReport {
            scanned_agents: agents.len(),
            ..NavAgentTickReport::default()
        };
        {
            let mut state = self.state.lock().expect("navigation state lock poisoned");
            state.stats.active_agents = agents.len();
            state.stats.active_obstacles = obstacles.len();
        }

        for (entity, agent) in agents {
            self.tick_agent(
                world,
                entity,
                agent,
                &obstacles,
                &agent_positions,
                dt_seconds,
                &mut report,
            );
        }
        Ok(report)
    }

    fn tick_world_agent(
        &self,
        world: &mut World,
        entity: u64,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        if dt_seconds <= 0.0 || !dt_seconds.is_finite() {
            return Ok(NavAgentTickReport::default());
        }

        let Some(agent) = collect_agent(world, entity) else {
            return Ok(NavAgentTickReport::default());
        };
        let agents = collect_agents(world);
        let agent_positions = collect_agent_positions(world, &agents);
        let obstacles = collect_obstacles(world);
        let mut report = NavAgentTickReport {
            scanned_agents: 1,
            ..NavAgentTickReport::default()
        };
        {
            let mut state = self.state.lock().expect("navigation state lock poisoned");
            state.stats.active_agents = agents.len();
            state.stats.active_obstacles = obstacles.len();
        }

        self.tick_agent(
            world,
            entity,
            agent,
            &obstacles,
            &agent_positions,
            dt_seconds,
            &mut report,
        );
        Ok(report)
    }

    fn stats(&self) -> NavigationRuntimeStats {
        self.state
            .lock()
            .expect("navigation state lock poisoned")
            .stats
            .clone()
    }
}

impl BuiltinNavigationManager {
    fn tick_agent(
        &self,
        world: &mut World,
        entity: u64,
        agent: NavMeshAgentDescriptor,
        obstacles: &[RuntimeObstacle],
        agent_positions: &[(u64, Vec3, Real)],
        dt_seconds: Real,
        report: &mut NavAgentTickReport,
    ) {
        let Some(destination) = agent.destination else {
            return;
        };
        if !agent.update_position {
            return;
        }
        let Some(transform) = world.world_transform(entity) else {
            report.blocked_agents += 1;
            report
                .diagnostics
                .push(format!("agent {entity} has no world transform"));
            return;
        };
        let current = transform.translation;
        let destination = Vec3::from_array(destination);
        let path_target = {
            let state = self.state.lock().expect("navigation state lock poisoned");
            match state.selected_mesh(None) {
                Ok(mesh) => match mesh.find_path(NavPathQuery {
                    nav_mesh: None,
                    start: current.to_array(),
                    end: destination.to_array(),
                    agent_type: agent.agent_type.clone(),
                    area_mask: agent.area_mask,
                }) {
                    result if result.status != NavPathStatus::NoPath => result
                        .points
                        .get(1)
                        .or_else(|| result.points.last())
                        .map(|point| Vec3::from_array(point.position))
                        .unwrap_or(destination),
                    _ => {
                        report.blocked_agents += 1;
                        report
                            .diagnostics
                            .push(format!("agent {entity} has no path on loaded navmesh"));
                        return;
                    }
                },
                Err(_) => destination,
            }
        };
        let movement_target = avoidance_adjusted_target(
            entity,
            current,
            path_target,
            &agent,
            obstacles,
            agent_positions,
        );
        let delta = movement_target - current;
        let distance = distance_xz(current, movement_target);
        if distance <= agent.stopping_distance.max(0.0) {
            return;
        }
        let max_step = agent.speed.max(0.0) * dt_seconds;
        if max_step <= Real::EPSILON {
            return;
        }
        let direction = Vec3::new(delta.x, 0.0, delta.z).normalize_or_zero();
        let mut next = current + direction * max_step.min(distance);
        if let Some(sampled) = {
            let state = self.state.lock().expect("navigation state lock poisoned");
            state.selected_mesh(None).ok().and_then(|mesh| {
                mesh.sample_position(NavSampleQuery {
                    nav_mesh: None,
                    position: next.to_array(),
                    extents: [
                        agent.radius.max(0.25),
                        agent.height.max(0.5),
                        agent.radius.max(0.25),
                    ],
                    agent_type: agent.agent_type.clone(),
                    area_mask: agent.area_mask,
                })
            })
        } {
            next = Vec3::from_array(sampled.position);
        }
        let rotation = if agent.update_rotation && direction.length_squared() > Real::EPSILON {
            rotation_from_direction(direction)
        } else {
            transform.rotation
        };
        let updated = Transform {
            translation: next,
            rotation,
            ..transform
        };
        match world.update_transform(entity, updated) {
            Ok(true) | Ok(false) => {
                report.moved_agents += 1;
            }
            Err(error) => {
                report.blocked_agents += 1;
                report
                    .diagnostics
                    .push(format!("agent {entity} could not move: {error}"));
            }
        }
    }
}

#[derive(Debug)]
struct BuiltinNavigationState {
    next_handle: u64,
    loaded: HashMap<NavMeshHandle, BakedNavMesh>,
    settings: NavigationSettingsAsset,
    stats: NavigationRuntimeStats,
}

impl Default for BuiltinNavigationState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            loaded: HashMap::new(),
            settings: NavigationSettingsAsset::default(),
            stats: NavigationRuntimeStats::default(),
        }
    }
}

impl BuiltinNavigationState {
    fn selected_mesh(
        &self,
        query_handle: Option<NavMeshHandle>,
    ) -> Result<&BakedNavMesh, NavigationError> {
        let handle = query_handle
            .or_else(|| self.loaded.keys().copied().min_by_key(|handle| handle.0))
            .ok_or_else(|| NavigationError::missing_nav_mesh("no nav mesh is loaded"))?;
        self.loaded.get(&handle).ok_or_else(|| {
            NavigationError::missing_nav_mesh(format!("nav mesh {:?} is not loaded", handle))
        })
    }
}

#[derive(Clone, Debug)]
struct BakedNavMesh {
    asset: NavMeshAsset,
    polygons: Vec<BakedPolygon>,
    adjacency: Vec<Vec<usize>>,
}

impl BakedNavMesh {
    fn new(asset: NavMeshAsset) -> Self {
        let polygons = asset
            .polygons
            .iter()
            .map(|polygon| BakedPolygon::from_asset(&asset, polygon))
            .collect::<Vec<_>>();
        let adjacency = build_adjacency(&polygons);
        Self {
            asset,
            polygons,
            adjacency,
        }
    }

    fn find_path(&self, query: NavPathQuery) -> NavPathResult {
        if self.polygons.is_empty() {
            return NavPathResult::no_path();
        }
        let start = Vec3::from_array(query.start);
        let end = Vec3::from_array(query.end);
        let Some(start_polygon) = self.best_polygon(start, query.area_mask) else {
            return NavPathResult::no_path();
        };
        let Some(end_polygon) = self.best_polygon(end, query.area_mask) else {
            return NavPathResult::no_path();
        };
        if start_polygon == end_polygon {
            let points = vec![
                self.path_point(start, start_polygon),
                self.path_point(self.polygons[start_polygon].project_point(end), end_polygon),
            ];
            return path_result(NavPathStatus::Complete, points, 1);
        }
        let Some(poly_path) = self.graph_path(start_polygon, end_polygon, end) else {
            let nearest = self.nearest_reachable_polygon(start_polygon, end, query.area_mask);
            let points = vec![
                self.path_point(start, start_polygon),
                self.path_point(self.polygons[nearest].center, nearest),
            ];
            return path_result(NavPathStatus::Partial, points, self.polygons.len());
        };
        let mut points = Vec::with_capacity(poly_path.len() + 1);
        points.push(self.path_point(start, start_polygon));
        for polygon_index in poly_path
            .iter()
            .skip(1)
            .take(poly_path.len().saturating_sub(2))
        {
            points.push(self.path_point(self.polygons[*polygon_index].center, *polygon_index));
        }
        points.push(self.path_point(self.polygons[end_polygon].project_point(end), end_polygon));
        path_result(
            NavPathStatus::Complete,
            deduplicate_path_points(points),
            poly_path.len(),
        )
    }

    fn sample_position(&self, query: NavSampleQuery) -> Option<NavSampleHit> {
        let position = Vec3::from_array(query.position);
        let max_distance = Vec3::from_array(query.extents).abs().length().max(0.05);
        self.polygons
            .iter()
            .enumerate()
            .filter(|(_, polygon)| area_allowed(polygon.area, query.area_mask))
            .map(|(_, polygon)| {
                let projected = polygon.project_point(position);
                let distance = distance_xz(position, projected);
                (polygon, projected, distance)
            })
            .filter(|(_, _, distance)| *distance <= max_distance)
            .min_by(|left, right| left.2.total_cmp(&right.2))
            .map(|(polygon, projected, distance)| NavSampleHit {
                position: projected.to_array(),
                distance,
                area: polygon.area,
            })
    }

    fn raycast(&self, query: NavRaycastQuery) -> NavRaycastResult {
        let end = Vec3::from_array(query.end);
        let hit = self
            .best_polygon(end, query.area_mask)
            .map(|polygon| self.polygons[polygon].contains_xz(end))
            .unwrap_or(false);
        NavRaycastResult {
            hit: !hit,
            position: query.end,
            normal: [0.0, 1.0, 0.0],
            distance: distance_xz(Vec3::from_array(query.start), end),
        }
    }

    fn graph_path(&self, start: usize, end: usize, end_position: Vec3) -> Option<Vec<usize>> {
        let mut open = BinaryHeap::new();
        let mut best_cost = vec![Real::INFINITY; self.polygons.len()];
        let mut previous = vec![None; self.polygons.len()];
        best_cost[start] = 0.0;
        open.push(PathOpenNode {
            polygon: start,
            estimated_total: distance_xz(self.polygons[start].center, end_position),
        });
        while let Some(node) = open.pop() {
            if node.polygon == end {
                break;
            }
            let current_cost = best_cost[node.polygon];
            for next in &self.adjacency[node.polygon] {
                let step_cost = distance_xz(
                    self.polygons[node.polygon].center,
                    self.polygons[*next].center,
                ) * self.area_cost(self.polygons[*next].area);
                let tentative = current_cost + step_cost;
                if tentative >= best_cost[*next] {
                    continue;
                }
                best_cost[*next] = tentative;
                previous[*next] = Some(node.polygon);
                open.push(PathOpenNode {
                    polygon: *next,
                    estimated_total: tentative
                        + distance_xz(self.polygons[*next].center, end_position),
                });
            }
        }
        if !best_cost[end].is_finite() {
            return None;
        }
        let mut path = vec![end];
        let mut cursor = end;
        while cursor != start {
            cursor = previous[cursor]?;
            path.push(cursor);
        }
        path.reverse();
        Some(path)
    }

    fn nearest_reachable_polygon(&self, start: usize, end: Vec3, area_mask: u64) -> usize {
        let mut stack = vec![start];
        let mut visited = vec![false; self.polygons.len()];
        let mut best = start;
        let mut best_distance = distance_xz(self.polygons[start].center, end);
        while let Some(current) = stack.pop() {
            if visited[current] {
                continue;
            }
            visited[current] = true;
            let distance = distance_xz(self.polygons[current].center, end);
            if distance < best_distance {
                best = current;
                best_distance = distance;
            }
            for next in &self.adjacency[current] {
                if !visited[*next] && area_allowed(self.polygons[*next].area, area_mask) {
                    stack.push(*next);
                }
            }
        }
        best
    }

    fn best_polygon(&self, position: Vec3, area_mask: u64) -> Option<usize> {
        self.polygons
            .iter()
            .enumerate()
            .filter(|(_, polygon)| area_allowed(polygon.area, area_mask))
            .map(|(index, polygon)| {
                let projected = polygon.project_point(position);
                let penalty = if polygon.contains_xz(position) {
                    0.0
                } else {
                    10_000.0
                };
                (index, distance_xz(position, projected) + penalty)
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(index, _)| index)
    }

    fn path_point(&self, position: Vec3, polygon: usize) -> NavPathPoint {
        NavPathPoint {
            position: position.to_array(),
            area: self.polygons[polygon].area,
            flags: Vec::new(),
        }
    }

    fn area_cost(&self, area: u8) -> Real {
        self.asset
            .area_costs
            .iter()
            .find(|cost| cost.area == area)
            .map(|cost| cost.cost.max(0.01))
            .unwrap_or(1.0)
    }
}

#[derive(Clone, Debug)]
struct BakedPolygon {
    area: u8,
    center: Vec3,
    min: Vec3,
    max: Vec3,
    index_set: Vec<u32>,
}

impl BakedPolygon {
    fn from_asset(asset: &NavMeshAsset, polygon: &crate::asset::NavMeshPolygonAsset) -> Self {
        let start = polygon.first_index as usize;
        let end = start
            .saturating_add(polygon.index_count as usize)
            .min(asset.indices.len());
        let index_set = asset.indices[start.min(asset.indices.len())..end].to_vec();
        let mut vertices = index_set
            .iter()
            .filter_map(|index| asset.vertices.get(*index as usize).copied())
            .map(Vec3::from_array)
            .collect::<Vec<_>>();
        vertices.sort_by(|left, right| {
            left.x
                .total_cmp(&right.x)
                .then(left.z.total_cmp(&right.z))
                .then(left.y.total_cmp(&right.y))
        });
        vertices.dedup_by(|left, right| {
            (left.x - right.x).abs() < 0.001 && (left.z - right.z).abs() < 0.001
        });
        let (min, max) = bounds_for_vertices(&vertices);
        let center = if vertices.is_empty() {
            Vec3::ZERO
        } else {
            vertices
                .iter()
                .copied()
                .fold(Vec3::ZERO, |sum, vertex| sum + vertex)
                / vertices.len() as Real
        };
        Self {
            area: polygon.area,
            center,
            min,
            max,
            index_set,
        }
    }

    fn contains_xz(&self, point: Vec3) -> bool {
        point.x >= self.min.x - 0.001
            && point.x <= self.max.x + 0.001
            && point.z >= self.min.z - 0.001
            && point.z <= self.max.z + 0.001
    }

    fn project_point(&self, point: Vec3) -> Vec3 {
        let x = point.x.clamp(self.min.x, self.max.x);
        let z = point.z.clamp(self.min.z, self.max.z);
        let y = self.center.y;
        Vec3::new(x, y, z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PathOpenNode {
    polygon: usize,
    estimated_total: Real,
}

impl Eq for PathOpenNode {}

impl Ord for PathOpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .estimated_total
            .total_cmp(&self.estimated_total)
            .then_with(|| other.polygon.cmp(&self.polygon))
    }
}

impl PartialOrd for PathOpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
struct RuntimeObstacle {
    entity: u64,
    center: Vec3,
    radius: Real,
    avoidance_enabled: bool,
}

fn build_adjacency(polygons: &[BakedPolygon]) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); polygons.len()];
    for left in 0..polygons.len() {
        for right in (left + 1)..polygons.len() {
            if polygons_share_edge(&polygons[left], &polygons[right]) {
                adjacency[left].push(right);
                adjacency[right].push(left);
            }
        }
    }
    adjacency
}

fn polygons_share_edge(left: &BakedPolygon, right: &BakedPolygon) -> bool {
    let shared_indices = left
        .index_set
        .iter()
        .filter(|index| right.index_set.contains(index))
        .count();
    if shared_indices >= 2 {
        return true;
    }
    rectangles_touch_or_overlap(left, right)
}

fn rectangles_touch_or_overlap(left: &BakedPolygon, right: &BakedPolygon) -> bool {
    let x_overlap = left.max.x >= right.min.x - 0.001 && right.max.x >= left.min.x - 0.001;
    let z_overlap = left.max.z >= right.min.z - 0.001 && right.max.z >= left.min.z - 0.001;
    x_overlap && z_overlap
}

fn bounds_for_vertices(vertices: &[Vec3]) -> (Vec3, Vec3) {
    let Some(first) = vertices.first().copied() else {
        return (Vec3::ZERO, Vec3::ZERO);
    };
    let mut min = first;
    let mut max = first;
    for vertex in vertices.iter().copied().skip(1) {
        min = min.min(vertex);
        max = max.max(vertex);
    }
    (min, max)
}

fn area_allowed(area: u8, mask: u64) -> bool {
    area == AREA_WALKABLE || (mask & (1_u64 << area.min(63))) != 0
}

fn path_result(
    status: NavPathStatus,
    points: Vec<NavPathPoint>,
    visited_nodes: usize,
) -> NavPathResult {
    let length = points
        .windows(2)
        .map(|window| {
            distance_xz(
                Vec3::from_array(window[0].position),
                Vec3::from_array(window[1].position),
            )
        })
        .sum();
    NavPathResult {
        status,
        points,
        length,
        visited_nodes,
    }
}

fn deduplicate_path_points(points: Vec<NavPathPoint>) -> Vec<NavPathPoint> {
    let mut result = Vec::new();
    for point in points {
        let duplicate = result.last().is_some_and(|previous: &NavPathPoint| {
            distance_xz(
                Vec3::from_array(previous.position),
                Vec3::from_array(point.position),
            ) < 0.05
        });
        if !duplicate {
            result.push(point);
        }
    }
    result
}

fn collect_agents(world: &World) -> Vec<(u64, NavMeshAgentDescriptor)> {
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

fn collect_agent(world: &World, entity: u64) -> Option<NavMeshAgentDescriptor> {
    let value = world.dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)?;
    serde_json::from_value::<NavMeshAgentDescriptor>(value.clone()).ok()
}

fn collect_agent_positions(
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

fn collect_obstacles(world: &World) -> Vec<RuntimeObstacle> {
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

fn avoidance_adjusted_target(
    entity: u64,
    current: Vec3,
    target: Vec3,
    agent: &NavMeshAgentDescriptor,
    obstacles: &[RuntimeObstacle],
    agents: &[(u64, Vec3, Real)],
) -> Vec3 {
    if matches!(
        agent.avoidance_quality,
        crate::core::framework::navigation::NavAvoidanceQuality::None
    ) {
        return target;
    }
    let desired_delta = Vec3::new(target.x - current.x, 0.0, target.z - current.z);
    let desired_distance = desired_delta.length();
    if desired_distance <= Real::EPSILON {
        return target;
    }
    let mut avoidance = Vec3::ZERO;
    for obstacle in obstacles
        .iter()
        .filter(|obstacle| obstacle.avoidance_enabled && obstacle.entity != entity)
    {
        let distance = distance_xz(current, obstacle.center);
        let limit = obstacle.radius + agent.radius.max(0.05) + 0.5;
        if distance > 0.001 && distance < limit {
            let away = Vec3::new(
                current.x - obstacle.center.x,
                0.0,
                current.z - obstacle.center.z,
            )
            .normalize_or_zero();
            avoidance += away * (limit - distance);
        }
    }
    for (other_entity, other_position, other_radius) in agents {
        if *other_entity == entity {
            continue;
        }
        let distance = distance_xz(current, *other_position);
        let limit = agent.radius.max(0.05) + *other_radius + 0.25;
        if distance > 0.001 && distance < limit {
            let away = Vec3::new(
                current.x - other_position.x,
                0.0,
                current.z - other_position.z,
            )
            .normalize_or_zero();
            avoidance += away * (limit - distance);
        }
    }
    if avoidance.length_squared() <= Real::EPSILON {
        return target;
    }
    let direction = avoidance.normalize_or_zero();
    if direction.length_squared() <= Real::EPSILON {
        current
    } else {
        current + direction * desired_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::NavMeshAsset;
    use crate::core::framework::navigation::{
        NavAvoidanceQuality, NavMeshObstacleDescriptor, NavMeshObstacleShape, DEFAULT_AGENT_TYPE,
        DEFAULT_AREA_MASK,
    };
    use crate::scene::components::NodeKind;

    #[test]
    fn tick_world_agent_moves_only_selected_agent_and_avoids_local_colliders() {
        let manager = BuiltinNavigationManager::new();
        manager
            .load_nav_mesh(NavMeshAsset::simple_quad(DEFAULT_AGENT_TYPE, 6.0))
            .unwrap();
        let mut world = World::empty();
        let selected = spawn_test_agent(
            &mut world,
            Vec3::new(-0.35, 0.0, 0.08),
            Vec3::new(4.0, 0.0, 0.0),
        );
        let other = spawn_test_agent(
            &mut world,
            Vec3::new(-0.42, 0.0, 0.16),
            Vec3::new(4.0, 0.0, 0.0),
        );
        let obstacle = spawn_test_obstacle(&mut world, Vec3::ZERO, 0.35);
        let selected_before = world.world_transform(selected).unwrap().translation;
        let other_before = world.world_transform(other).unwrap().translation;
        let obstacle_position = world.world_transform(obstacle).unwrap().translation;
        let obstacle_distance_before = distance_xz(selected_before, obstacle_position);
        let other_distance_before = distance_xz(selected_before, other_before);

        let report = manager.tick_world_agent(&mut world, selected, 0.1).unwrap();

        let selected_after = world.world_transform(selected).unwrap().translation;
        let other_after = world.world_transform(other).unwrap().translation;
        assert_eq!(report.scanned_agents, 1);
        assert_eq!(report.moved_agents, 1);
        assert_eq!(
            other_after, other_before,
            "targeted navigation ticks must not advance every agent in the scene"
        );
        assert!(
            distance_xz(selected_after, obstacle_position) > obstacle_distance_before,
            "selected agent should steer away from local obstacle instead of moving deeper into it: before={selected_before:?} after={selected_after:?}"
        );
        assert!(
            distance_xz(selected_after, other_before) > other_distance_before,
            "selected agent should also separate from nearby agents: before={selected_before:?} after={selected_after:?} other={other_before:?}"
        );
    }

    fn spawn_test_agent(world: &mut World, position: Vec3, destination: Vec3) -> u64 {
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .update_transform(entity, Transform::from_translation(position))
            .unwrap();
        world
            .set_dynamic_component(
                entity,
                NAV_MESH_AGENT_COMPONENT_TYPE,
                serde_json::to_value(NavMeshAgentDescriptor {
                    agent_type: DEFAULT_AGENT_TYPE.to_string(),
                    radius: 0.3,
                    height: 1.7,
                    speed: 2.0,
                    stopping_distance: 0.02,
                    avoidance_quality: NavAvoidanceQuality::High,
                    area_mask: DEFAULT_AREA_MASK,
                    destination: Some(destination.to_array()),
                    ..NavMeshAgentDescriptor::default()
                })
                .unwrap(),
            )
            .unwrap();
        entity
    }

    fn spawn_test_obstacle(world: &mut World, position: Vec3, radius: Real) -> u64 {
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .update_transform(entity, Transform::from_translation(position))
            .unwrap();
        world
            .set_dynamic_component(
                entity,
                NAV_MESH_OBSTACLE_COMPONENT_TYPE,
                serde_json::to_value(NavMeshObstacleDescriptor {
                    shape: NavMeshObstacleShape::Capsule,
                    radius,
                    avoidance_enabled: true,
                    ..NavMeshObstacleDescriptor::default()
                })
                .unwrap(),
            )
            .unwrap();
        entity
    }
}

fn rotation_from_direction(direction: Vec3) -> crate::core::math::Quat {
    crate::core::math::Quat::from_rotation_y(direction.x.atan2(-direction.z))
}

fn distance_xz(left: Vec3, right: Vec3) -> Real {
    let delta = left - right;
    (delta.x * delta.x + delta.z * delta.z).sqrt()
}
