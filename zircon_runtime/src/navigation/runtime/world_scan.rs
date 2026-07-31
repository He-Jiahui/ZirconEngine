use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

use crate::core::framework::navigation::{
    NavMeshAgentDescriptor, NavMeshObstacleDescriptor, NavMeshObstacleShape,
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use crate::core::math::{Real, Vec3};
use crate::scene::World;

pub(super) const MAX_NAVIGATION_AVOIDANCE_NEIGHBORS: usize = 64;

const NAVIGATION_AVOIDANCE_CELL_SIZE: Real = 2.0;
const MAX_NAVIGATION_AVOIDANCE_CELL_RADIUS: i32 = 4;
const MAX_NAVIGATION_AVOIDANCE_CELL_VISITS_PER_INDEX: usize = 81;
pub(super) const MAX_NAVIGATION_AVOIDANCE_CELL_VISITS: usize =
    2 * MAX_NAVIGATION_AVOIDANCE_CELL_VISITS_PER_INDEX;
pub(super) const MAX_NAVIGATION_AVOIDANCE_CANDIDATE_VISITS: usize =
    2 * MAX_NAVIGATION_AVOIDANCE_NEIGHBORS;

#[derive(Clone, Debug)]
pub(super) struct RuntimeObstacle {
    pub(super) entity: u64,
    pub(super) center: Vec3,
    pub(super) radius: Real,
    pub(super) avoidance_enabled: bool,
}

#[derive(Debug)]
pub(super) struct RuntimeAgent {
    pub(super) entity: u64,
    pub(super) descriptor: NavMeshAgentDescriptor,
}

#[derive(Debug)]
pub(super) struct NavigationWorldProjection {
    pub(super) generation: u64,
    pub(super) agents: Vec<RuntimeAgent>,
    pub(super) agent_positions: Vec<(u64, Vec3, Real)>,
    pub(super) obstacles: Vec<RuntimeObstacle>,
    pub(super) agent_component_rows: usize,
    pub(super) obstacle_component_rows: usize,
    pub(super) agent_position_lookups: u64,
    pub(super) avoidance_cell_visits: usize,
    pub(super) avoidance_candidate_visits: usize,
    agent_position_rows: HashMap<u64, usize>,
    avoidance_index: NavigationAvoidanceIndex,
    avoidance_epoch: usize,
    avoidance_obstacle_scratch: Vec<RuntimeObstacle>,
    avoidance_agent_scratch: Vec<(u64, Vec3, Real)>,
}

impl NavigationWorldProjection {
    pub(super) fn update_agent_position(&mut self, entity: u64, position: Vec3) {
        self.agent_position_lookups = self.agent_position_lookups.saturating_add(1);
        let Some(&index) = self.agent_position_rows.get(&entity) else {
            return;
        };
        let previous = self.agent_positions[index].1;
        self.agent_positions[index].1 = position;
        self.avoidance_index.move_agent(index, previous, position);
    }

    pub(super) fn begin_avoidance_frame(&mut self) {
        self.avoidance_epoch = self.avoidance_epoch.wrapping_add(1);
    }

    pub(super) fn local_avoidance_rows(
        &mut self,
        entity: u64,
        position: Vec3,
        agent_radius: Real,
    ) -> (&[RuntimeObstacle], &[(u64, Vec3, Real)]) {
        let rotation = avoidance_rotation(self.avoidance_epoch, entity);
        self.avoidance_obstacle_scratch.clear();
        let obstacle_work = {
            let index = &self.avoidance_index;
            let obstacles = &self.obstacles;
            let scratch = &mut self.avoidance_obstacle_scratch;
            index.for_each_nearby_obstacle(position, agent_radius, rotation, |row| {
                scratch.push(obstacles[row].clone());
                true
            })
        };

        self.avoidance_agent_scratch.clear();
        let agent_work = {
            let index = &self.avoidance_index;
            let agents = &self.agent_positions;
            let scratch = &mut self.avoidance_agent_scratch;
            index.for_each_nearby_agent(position, agent_radius, rotation, |row| {
                let candidate = agents[row];
                if candidate.0 != entity {
                    scratch.push(candidate);
                    true
                } else {
                    false
                }
            })
        };
        self.avoidance_cell_visits = obstacle_work.0.saturating_add(agent_work.0);
        self.avoidance_candidate_visits = obstacle_work.1.saturating_add(agent_work.1);
        debug_assert!(self.avoidance_cell_visits <= MAX_NAVIGATION_AVOIDANCE_CELL_VISITS);
        debug_assert!(self.avoidance_candidate_visits <= MAX_NAVIGATION_AVOIDANCE_CANDIDATE_VISITS);

        (
            self.avoidance_obstacle_scratch.as_slice(),
            self.avoidance_agent_scratch.as_slice(),
        )
    }
}

#[derive(Debug)]
struct NavigationAvoidanceIndex {
    agents: BTreeMap<(i32, i32), Vec<usize>>,
    obstacles: BTreeMap<(i32, i32), Vec<usize>>,
    max_agent_radius: Real,
    max_obstacle_radius: Real,
}

impl NavigationAvoidanceIndex {
    fn new(agents: &[(u64, Vec3, Real)], obstacles: &[RuntimeObstacle]) -> Self {
        let mut agent_cells = BTreeMap::<(i32, i32), Vec<usize>>::new();
        let mut obstacle_cells = BTreeMap::<(i32, i32), Vec<usize>>::new();
        let mut max_agent_radius: Real = 0.05;
        let mut max_obstacle_radius: Real = 0.05;

        for (index, (_, position, radius)) in agents.iter().enumerate() {
            agent_cells
                .entry(navigation_avoidance_cell(*position))
                .or_default()
                .push(index);
            max_agent_radius = max_agent_radius.max(*radius);
        }
        for (index, obstacle) in obstacles.iter().enumerate() {
            obstacle_cells
                .entry(navigation_avoidance_cell(obstacle.center))
                .or_default()
                .push(index);
            max_obstacle_radius = max_obstacle_radius.max(obstacle.radius);
        }

        Self {
            agents: agent_cells,
            obstacles: obstacle_cells,
            max_agent_radius,
            max_obstacle_radius,
        }
    }

    fn for_each_nearby_agent(
        &self,
        position: Vec3,
        agent_radius: Real,
        rotation: usize,
        visit: impl FnMut(usize) -> bool,
    ) -> (usize, usize) {
        self.for_each_nearby(
            &self.agents,
            position,
            agent_radius.max(0.05) + self.max_agent_radius + 0.25,
            rotation,
            visit,
        )
    }

    fn move_agent(&mut self, row: usize, previous: Vec3, next: Vec3) {
        let previous_cell = navigation_avoidance_cell(previous);
        let next_cell = navigation_avoidance_cell(next);
        if previous_cell == next_cell {
            return;
        }
        let remove_previous_cell = if let Some(rows) = self.agents.get_mut(&previous_cell) {
            rows.retain(|candidate| *candidate != row);
            rows.is_empty()
        } else {
            false
        };
        if remove_previous_cell {
            self.agents.remove(&previous_cell);
        }
        let rows = self.agents.entry(next_cell).or_default();
        rows.push(row);
        rows.sort_unstable();
    }

    fn for_each_nearby_obstacle(
        &self,
        position: Vec3,
        agent_radius: Real,
        rotation: usize,
        visit: impl FnMut(usize) -> bool,
    ) -> (usize, usize) {
        self.for_each_nearby(
            &self.obstacles,
            position,
            agent_radius.max(0.05) + self.max_obstacle_radius + 0.5,
            rotation,
            visit,
        )
    }

    fn for_each_nearby(
        &self,
        cells: &BTreeMap<(i32, i32), Vec<usize>>,
        position: Vec3,
        extent: Real,
        rotation: usize,
        mut visit: impl FnMut(usize) -> bool,
    ) -> (usize, usize) {
        let (cell_x, cell_z) = navigation_avoidance_cell(position);
        let range = (extent.max(0.05) / NAVIGATION_AVOIDANCE_CELL_SIZE)
            .ceil()
            .max(1.0) as i32;
        let range = range.min(MAX_NAVIGATION_AVOIDANCE_CELL_RADIUS);
        let cell_span = (range * 2 + 1) as usize;
        let cell_count = cell_span.saturating_mul(cell_span);
        let first_cell_x = cell_x.saturating_sub(range);
        let first_cell_z = cell_z.saturating_sub(range);
        let mut cell_visits = 0;
        let mut candidate_visits = 0;
        'cells: for offset in 0..cell_count {
            let cell_offset = (offset + rotation % cell_count) % cell_count;
            let x = first_cell_x.saturating_add((cell_offset / cell_span) as i32);
            let z = first_cell_z.saturating_add((cell_offset % cell_span) as i32);
            cell_visits += 1;
            let Some(rows) = cells.get(&(x, z)) else {
                continue;
            };
            let row_rotation = rotation % rows.len();
            for row_offset in 0..rows.len() {
                if candidate_visits >= MAX_NAVIGATION_AVOIDANCE_NEIGHBORS {
                    break 'cells;
                }
                if visit(rows[(row_offset + row_rotation) % rows.len()]) {
                    candidate_visits += 1;
                }
            }
        }
        debug_assert!(cell_visits <= MAX_NAVIGATION_AVOIDANCE_CELL_VISITS_PER_INDEX);
        debug_assert!(candidate_visits <= MAX_NAVIGATION_AVOIDANCE_NEIGHBORS);
        (cell_visits, candidate_visits)
    }
}

fn navigation_avoidance_cell(position: Vec3) -> (i32, i32) {
    (
        (position.x / NAVIGATION_AVOIDANCE_CELL_SIZE).floor() as i32,
        (position.z / NAVIGATION_AVOIDANCE_CELL_SIZE).floor() as i32,
    )
}

fn avoidance_rotation(epoch: usize, entity: u64) -> usize {
    // Fold the stable 64-bit entity ID so the rotation does not depend on pointer width.
    let folded_entity = (entity ^ (entity >> 32)) as u32;
    epoch.wrapping_add(folded_entity as usize)
}

pub(super) fn collect_navigation_world_projection(world: &World) -> NavigationWorldProjection {
    let mut agents = Vec::new();
    let mut agent_positions = Vec::new();
    let mut obstacles = Vec::new();
    let mut component_rows = Vec::new();

    world.dynamic_component_rows(NAV_MESH_AGENT_COMPONENT_TYPE, &mut component_rows);
    let agent_component_rows = component_rows.len();
    for (entity, value) in component_rows.drain(..) {
        let Ok(agent) = NavMeshAgentDescriptor::deserialize(value) else {
            continue;
        };
        if let Some(transform) = world.world_transform(entity) {
            agent_positions.push((entity, transform.translation, agent.radius.max(0.05)));
        }
        agents.push(RuntimeAgent {
            entity,
            descriptor: agent,
        });
    }

    world.dynamic_component_rows(NAV_MESH_OBSTACLE_COMPONENT_TYPE, &mut component_rows);
    let obstacle_component_rows = component_rows.len();
    for (entity, value) in component_rows.drain(..) {
        let Ok(obstacle) = NavMeshObstacleDescriptor::deserialize(value) else {
            continue;
        };
        let Some(transform) = world
            .world_transform(entity)
            .or_else(|| world.node_record(entity).map(|node| node.transform))
        else {
            continue;
        };
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
        obstacles.push(RuntimeObstacle {
            entity,
            center,
            radius,
            avoidance_enabled: obstacle.avoidance_enabled,
        });
    }

    let mut agent_position_rows = HashMap::with_capacity(agent_positions.len());
    for (row, (entity, _, _)) in agent_positions.iter().enumerate() {
        agent_position_rows.insert(*entity, row);
    }
    let avoidance_index = NavigationAvoidanceIndex::new(&agent_positions, &obstacles);
    NavigationWorldProjection {
        generation: world.world_generation(),
        agents,
        agent_positions,
        obstacles,
        agent_component_rows,
        obstacle_component_rows,
        agent_position_lookups: 0,
        avoidance_cell_visits: 0,
        avoidance_candidate_visits: 0,
        agent_position_rows,
        avoidance_index,
        avoidance_epoch: 0,
        avoidance_obstacle_scratch: Vec::with_capacity(MAX_NAVIGATION_AVOIDANCE_NEIGHBORS),
        avoidance_agent_scratch: Vec::with_capacity(MAX_NAVIGATION_AVOIDANCE_NEIGHBORS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_avoidance_rotation_is_deterministic_and_agent_distinct_in_one_frame() {
        let agents = (0..=MAX_NAVIGATION_AVOIDANCE_NEIGHBORS)
            .map(|index| (index as u64, Vec3::ZERO, 0.05))
            .collect::<Vec<_>>();
        let index = NavigationAvoidanceIndex::new(&agents, &[]);
        let epoch = 41;

        let rows_for = |entity| {
            let mut rows = Vec::new();
            index.for_each_nearby_agent(
                Vec3::ZERO,
                0.05,
                avoidance_rotation(epoch, entity),
                |row| {
                    rows.push(row);
                    true
                },
            );
            rows
        };

        let first = rows_for(1);
        let second = rows_for(2);

        assert_eq!(first.len(), MAX_NAVIGATION_AVOIDANCE_NEIGHBORS);
        assert_ne!(
            first, second,
            "agents sharing one dense cell must receive distinct fair candidate rotations"
        );
        assert_eq!(first, rows_for(1));
    }

    #[test]
    fn dense_avoidance_rotation_reaches_neighboring_cells_across_epochs() {
        let mut agents = (0..=MAX_NAVIGATION_AVOIDANCE_NEIGHBORS)
            .map(|index| (index as u64, Vec3::new(-0.2, 0.0, -0.2), 0.05))
            .collect::<Vec<_>>();
        let neighboring_entity = 10_000;
        agents.push((neighboring_entity, Vec3::new(0.2, 0.0, 0.2), 0.05));
        let index = NavigationAvoidanceIndex::new(&agents, &[]);

        let visited_neighboring_cell = (0..9).any(|epoch| {
            let mut rows = Vec::new();
            index.for_each_nearby_agent(Vec3::ZERO, 0.05, avoidance_rotation(epoch, 1), |row| {
                rows.push(agents[row].0);
                true
            });
            rows.contains(&neighboring_entity)
        });

        assert!(
            visited_neighboring_cell,
            "a dense earlier cell must not starve a neighboring-cell candidate across rotations"
        );
    }

    #[test]
    fn dense_avoidance_rotation_remains_bounded_after_epoch_wraparound() {
        let agents = (0..=MAX_NAVIGATION_AVOIDANCE_NEIGHBORS)
            .map(|index| (index as u64, Vec3::ZERO, 0.05))
            .collect::<Vec<_>>();
        let index = NavigationAvoidanceIndex::new(&agents, &[]);
        let mut rows = Vec::new();

        index.for_each_nearby_agent(Vec3::ZERO, 0.05, usize::MAX, |row| {
            rows.push(row);
            true
        });

        assert_eq!(rows.len(), MAX_NAVIGATION_AVOIDANCE_NEIGHBORS);
    }
}
