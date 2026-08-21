mod writeback;

use std::collections::{HashMap, HashSet};

use zircon_plugin_navigation_recast::{
    RecastCrowd, RecastCrowdAgentHandle, RecastCrowdAgentState, RecastCrowdConfig,
};
use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE, NavAgentTickReport,
    NavAgentWritebackMode, NavMeshAgentDescriptor, NavMeshHandle, NavPathQuery,
    NavigationAgentDebugState, NavigationDebugCapture, NavigationError, NavigationManager,
};
use zircon_runtime::core::math::{Real, Vec3};
use zircon_runtime::navigation::NavRepathBudget;
use zircon_runtime::scene::World;

use crate::component_json::parse_component;
use crate::manager::DefaultNavigationManager;
use crate::runtime_obstacles::has_obstacle_worlds;

const NAV_CROWD_MAX_AGENTS: u32 = 256;
const NAV_CROWD_MAX_AGENT_RADIUS: Real = 8.0;
const DETOUR_TARGET_FAILED: u8 = 1;

#[derive(Clone, Debug)]
struct CrowdAgentBinding {
    handle: RecastCrowdAgentHandle,
    movement_settings: NavMeshAgentDescriptor,
    last_destination: Option<[Real; 3]>,
    last_position: [Real; 3],
    blocked: bool,
}

#[derive(Debug)]
pub(super) struct NavigationCrowdRuntime {
    crowd: RecastCrowd,
    agents: HashMap<u64, CrowdAgentBinding>,
    repath_cursor: usize,
}

pub(super) fn tick_world_agents(
    manager: &DefaultNavigationManager,
    world: &mut World,
    dt_seconds: Real,
) -> Result<NavAgentTickReport, NavigationError> {
    if dt_seconds <= 0.0 || !dt_seconds.is_finite() {
        return Ok(NavAgentTickReport::default());
    }
    let loaded = manager.loaded_assets();
    if loaded.is_empty() {
        return crate::manager::tick::tick_world_agents_legacy(manager, world, dt_seconds);
    }
    let default_handle = loaded[0].0;
    let assets = loaded.into_iter().collect::<HashMap<_, _>>();
    let (agents, has_runtime_obstacles) = collect_agent_tick_inputs(world);
    let mut report = NavAgentTickReport {
        scanned_agents: agents.len(),
        ..NavAgentTickReport::default()
    };
    let mut groups = HashMap::<NavMeshHandle, Vec<(u64, NavMeshAgentDescriptor)>>::new();
    for (entity, agent) in agents {
        groups
            .entry(agent.nav_mesh.unwrap_or(default_handle))
            .or_default()
            .push((entity, agent));
    }
    if has_runtime_obstacles
        || has_obstacle_worlds(manager)
        || groups
            .keys()
            .filter_map(|handle| assets.get(handle))
            .any(|asset| !asset.off_mesh_links.is_empty())
    {
        manager.lock_state().crowds.clear();
        return crate::manager::tick::tick_world_agents_legacy(manager, world, dt_seconds);
    }

    let positions = groups
        .values()
        .flatten()
        .filter_map(|(entity, _)| {
            world
                .world_transform(*entity)
                .map(|transform| (*entity, transform.translation.to_array()))
        })
        .collect::<HashMap<_, _>>();
    let mut budget = world
        .get_resource::<NavRepathBudget>()
        .copied()
        .unwrap_or_default();
    budget.begin_frame();
    let mut handles = groups.keys().copied().collect::<Vec<_>>();
    handles.sort_by_key(|handle| handle.0);

    let mut writebacks = Vec::<(u64, NavMeshAgentDescriptor, RecastCrowdAgentState)>::new();
    {
        let mut runtime_state = manager.lock_state();
        runtime_state
            .crowds
            .retain(|handle, _| assets.contains_key(handle));
        let owners = groups
            .iter()
            .flat_map(|(handle, agents)| agents.iter().map(|(entity, _)| (*entity, *handle)))
            .collect::<HashMap<_, _>>();
        remove_agents_from_previous_crowds(&mut runtime_state.crowds, &owners, &mut report);
        if !handles.is_empty() {
            let start = runtime_state.crowd_handle_cursor % handles.len();
            handles.rotate_left(start);
            runtime_state.crowd_handle_cursor = (start + 1) % handles.len();
        }
        for handle in handles {
            let group = groups.remove(&handle).unwrap_or_default();
            let Some(asset) = assets.get(&handle) else {
                report.blocked_agents += group.len();
                report.diagnostics.push(format!(
                    "{} agents reference unloaded nav mesh {:?}",
                    group.len(),
                    handle
                ));
                continue;
            };
            ensure_crowd(&mut runtime_state.crowds, handle, asset)?;
            let runtime = runtime_state
                .crowds
                .get_mut(&handle)
                .expect("crowd was ensured");
            let active_entities = group
                .iter()
                .map(|(entity, _)| *entity)
                .collect::<HashSet<_>>();
            remove_stale_agents(runtime, &active_entities)?;
            synchronize_agents(runtime, &group, &positions, &mut report);
            apply_repath_budget(runtime, &group, &positions, &mut budget, &mut report)?;
            runtime.crowd.update(dt_seconds)?;
            let states = runtime.crowd.read_states()?;
            let entities_by_handle = runtime
                .agents
                .iter()
                .map(|(entity, binding)| (binding.handle, *entity))
                .collect::<HashMap<_, _>>();
            let agents_by_entity = group.into_iter().collect::<HashMap<_, _>>();
            for state in states {
                let Some(entity) = entities_by_handle.get(&state.handle).copied() else {
                    continue;
                };
                let mut failed_handle = None;
                if let Some(binding) = runtime.agents.get_mut(&entity) {
                    binding.last_position = state.position;
                    let failed = state.target_state == DETOUR_TARGET_FAILED || state.partial_path;
                    if failed && !binding.blocked {
                        report.blocked_agents += 1;
                        if let Some(destination) = agents_by_entity
                            .get(&entity)
                            .and_then(|agent| agent.destination)
                        {
                            report.no_path_agents.push((entity, destination));
                        }
                        report.diagnostics.push(format!(
                            "agent {entity} has no path to its complete Crowd target"
                        ));
                    }
                    binding.blocked = failed;
                    failed_handle = failed.then_some(binding.handle);
                }
                if let Some(handle) = failed_handle {
                    runtime.crowd.clear_target(handle)?;
                }
                if state.target_state != DETOUR_TARGET_FAILED && !state.partial_path {
                    if let Some(agent) = agents_by_entity.get(&entity) {
                        writebacks.push((entity, agent.clone(), state));
                    }
                }
            }
        }
        runtime_state.stats.active_agents = report.scanned_agents;
    }

    for (entity, agent, state) in &writebacks {
        writeback::write_agent_state(world, *entity, agent, state, dt_seconds, &mut report);
        if let Some(destination) = agent.destination {
            let remaining =
                (Vec3::from_array(destination) - Vec3::from_array(state.position)).length();
            if remaining <= agent.stopping_distance.max(0.0) {
                report.arrived_agents.push((*entity, destination));
            }
        }
    }
    if world
        .get_resource::<NavigationDebugCapture>()
        .is_some_and(|capture| capture.enabled)
    {
        for (entity, agent, state) in &writebacks {
            let path = agent.destination.and_then(|destination| {
                manager
                    .find_path(NavPathQuery {
                        nav_mesh: agent.nav_mesh,
                        start: state.position,
                        end: destination,
                        agent_type: agent.agent_type.clone(),
                        area_mask: agent.area_mask,
                    })
                    .ok()
            });
            report.debug_agents.push(NavigationAgentDebugState {
                entity: *entity,
                position: state.position,
                destination: agent.destination,
                desired_velocity: state.desired_velocity,
                avoidance_velocity: state.avoidance_velocity,
                path_status: path.as_ref().map(|path| path.status),
                path: path
                    .map(|path| {
                        path.points
                            .into_iter()
                            .map(|point| point.position)
                            .collect()
                    })
                    .unwrap_or_default(),
            });
        }
    }
    if let Some(resource) = world.get_resource_mut::<NavRepathBudget>() {
        *resource = budget;
    }
    Ok(report)
}

fn ensure_crowd(
    crowds: &mut HashMap<NavMeshHandle, NavigationCrowdRuntime>,
    nav_mesh: NavMeshHandle,
    asset: &NavMeshAsset,
) -> Result<(), NavigationError> {
    if crowds.contains_key(&nav_mesh) {
        return Ok(());
    }
    let crowd = RecastCrowd::from_asset(
        asset,
        RecastCrowdConfig {
            max_agents: NAV_CROWD_MAX_AGENTS,
            max_agent_radius: NAV_CROWD_MAX_AGENT_RADIUS,
        },
    )?;
    crowds.insert(
        nav_mesh,
        NavigationCrowdRuntime {
            crowd,
            agents: HashMap::new(),
            repath_cursor: 0,
        },
    );
    Ok(())
}

fn remove_stale_agents(
    runtime: &mut NavigationCrowdRuntime,
    active_entities: &HashSet<u64>,
) -> Result<(), NavigationError> {
    let stale = runtime
        .agents
        .keys()
        .filter(|entity| !active_entities.contains(entity))
        .copied()
        .collect::<Vec<_>>();
    for entity in stale {
        if let Some(binding) = runtime.agents.remove(&entity) {
            runtime.crowd.remove_agent(binding.handle)?;
        }
    }
    Ok(())
}

fn remove_agents_from_previous_crowds(
    crowds: &mut HashMap<NavMeshHandle, NavigationCrowdRuntime>,
    owners: &HashMap<u64, NavMeshHandle>,
    report: &mut NavAgentTickReport,
) {
    for (handle, runtime) in crowds {
        let stale = runtime
            .agents
            .keys()
            .filter(|entity| owners.get(entity) != Some(handle))
            .copied()
            .collect::<Vec<_>>();
        for entity in stale {
            if let Some(binding) = runtime.agents.remove(&entity) {
                if let Err(error) = runtime.crowd.remove_agent(binding.handle) {
                    report.diagnostics.push(format!(
                        "agent {entity} stale Crowd binding cleanup failed: {error}"
                    ));
                }
            }
        }
    }
}

fn synchronize_agents(
    runtime: &mut NavigationCrowdRuntime,
    agents: &[(u64, NavMeshAgentDescriptor)],
    positions: &HashMap<u64, [Real; 3]>,
    report: &mut NavAgentTickReport,
) {
    for (entity, agent) in agents {
        let Some(position) = positions.get(entity).copied() else {
            continue;
        };
        let movement_settings = movement_settings(agent);
        let replace = runtime
            .agents
            .get(entity)
            .is_some_and(|binding| binding.movement_settings != movement_settings);
        if replace {
            if let Some(binding) = runtime.agents.remove(entity) {
                if let Err(error) = runtime.crowd.remove_agent(binding.handle) {
                    report.diagnostics.push(format!(
                        "agent {entity} Crowd parameter replacement cleanup failed: {error}"
                    ));
                }
            }
        }
        if !runtime.agents.contains_key(entity) {
            match runtime.crowd.add_agent(position, agent) {
                Ok(handle) => {
                    runtime.agents.insert(
                        *entity,
                        CrowdAgentBinding {
                            handle,
                            movement_settings,
                            last_destination: None,
                            last_position: position,
                            blocked: false,
                        },
                    );
                }
                Err(error) => {
                    report.blocked_agents += 1;
                    report
                        .diagnostics
                        .push(format!("agent {entity} could not join Crowd: {error}"));
                }
            }
        } else if matches!(agent.writeback_mode, NavAgentWritebackMode::DesiredVelocity)
            || runtime
                .agents
                .get(entity)
                .is_some_and(|binding| binding.blocked)
        {
            let handle = runtime
                .agents
                .get(entity)
                .expect("agent binding exists")
                .handle;
            if let Err(error) = runtime.crowd.sync_agent_position(handle, position) {
                report.blocked_agents += 1;
                report.diagnostics.push(format!(
                    "agent {entity} controller position could not synchronize: {error}"
                ));
                if let Err(remove_error) = runtime.crowd.remove_agent(handle) {
                    report.diagnostics.push(format!(
                        "agent {entity} failed to retire unsynchronized Crowd binding: {remove_error}"
                    ));
                }
                runtime.agents.remove(entity);
            }
        }
    }
}

fn apply_repath_budget(
    runtime: &mut NavigationCrowdRuntime,
    agents: &[(u64, NavMeshAgentDescriptor)],
    positions: &HashMap<u64, [Real; 3]>,
    budget: &mut NavRepathBudget,
    report: &mut NavAgentTickReport,
) -> Result<(), NavigationError> {
    if agents.is_empty() {
        runtime.repath_cursor = 0;
        return Ok(());
    }
    let start_index = runtime.repath_cursor.min(agents.len() - 1);
    let mut next_cursor = start_index;
    for offset in 0..agents.len() {
        let index = (start_index + offset) % agents.len();
        let (entity, agent) = &agents[index];
        let Some(binding) = runtime.agents.get_mut(entity) else {
            report.blocked_agents += 1;
            continue;
        };
        if !agent.update_position
            && matches!(agent.writeback_mode, NavAgentWritebackMode::Transform)
        {
            if binding.last_destination.take().is_some() {
                runtime.crowd.clear_target(binding.handle)?;
            }
            binding.blocked = false;
            continue;
        }
        let Some(destination) = agent.destination else {
            if binding.last_destination.take().is_some() {
                runtime.crowd.clear_target(binding.handle)?;
            }
            binding.blocked = false;
            continue;
        };
        let target_changed = binding.last_destination != Some(destination);
        if !target_changed && !(agent.auto_repath && binding.blocked) {
            continue;
        }
        if !budget.try_consume() {
            break;
        }
        next_cursor = (index + 1) % agents.len();
        let Some(start) = positions.get(entity).copied() else {
            report.blocked_agents += 1;
            continue;
        };
        binding.last_destination = Some(destination);
        match runtime
            .crowd
            .set_target(binding.handle, stopping_target(start, destination, agent))
        {
            Ok(()) => binding.blocked = false,
            Err(error) => {
                binding.blocked = true;
                report.blocked_agents += 1;
                report.no_path_agents.push((*entity, destination));
                report
                    .diagnostics
                    .push(format!("agent {entity} Crowd target rejected: {error}"));
            }
        }
    }
    runtime.repath_cursor = next_cursor;
    Ok(())
}

fn stopping_target(
    start: [Real; 3],
    destination: [Real; 3],
    agent: &NavMeshAgentDescriptor,
) -> [Real; 3] {
    if !agent.auto_braking {
        return destination;
    }
    let start = Vec3::from_array(start);
    let destination = Vec3::from_array(destination);
    let delta = destination - start;
    let distance = delta.length();
    let stopping_distance = agent.stopping_distance.max(0.0).min(distance);
    (destination - delta.normalize_or_zero() * stopping_distance).to_array()
}

fn movement_settings(agent: &NavMeshAgentDescriptor) -> NavMeshAgentDescriptor {
    let mut settings = agent.clone();
    settings.destination = None;
    settings
}

fn collect_agent_tick_inputs(world: &World) -> (Vec<(u64, NavMeshAgentDescriptor)>, bool) {
    let mut agents = Vec::new();
    let mut has_runtime_obstacles = false;
    for node in world.node_records() {
        if let Some(value) = world.dynamic_component(node.id, NAV_MESH_AGENT_COMPONENT_TYPE) {
            agents.push((node.id, parse_component::<NavMeshAgentDescriptor>(value)));
        }
        has_runtime_obstacles |= world
            .dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
            .is_some();
    }
    (agents, has_runtime_obstacles)
}

#[cfg(test)]
mod input_sampling_tests {
    use std::{hint::black_box, time::Instant};

    use serde_json::json;
    use zircon_runtime::core::framework::navigation::{
        NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE, NavMeshAgentDescriptor,
    };
    use zircon_runtime::scene::{NodeKind, World};

    use super::collect_agent_tick_inputs;
    use crate::component_json::parse_component;

    const BENCHMARK_NODE_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn single_pass_agent_tick_inputs_preserve_agents_and_obstacle_detection() {
        let world = benchmark_world(64);
        let legacy = legacy_collect_agent_tick_inputs(&world);
        let optimized = collect_agent_tick_inputs(&world);

        assert_eq!(optimized, legacy);
    }

    #[test]
    fn agent_tick_inputs_use_one_world_projection() {
        let source = include_str!("agent.rs");
        let tick = source
            .split("pub(super) fn tick_world_agents")
            .nth(1)
            .and_then(|body| body.split("fn ensure_crowd").next())
            .expect("agent tick source");
        let collector = source
            .split("fn collect_agent_tick_inputs")
            .nth(1)
            .and_then(|body| body.split("#[cfg(test)]").next())
            .expect("agent tick input collector source");

        assert!(tick.contains("collect_agent_tick_inputs(world)"));
        assert!(!tick.contains("collect_runtime_obstacles(world)"));
        assert_eq!(collector.matches(".node_records()").count(), 1);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn single_pass_agent_tick_inputs_release_benchmark_evidence() {
        let world = benchmark_world(BENCHMARK_NODE_COUNT);
        assert_eq!(
            collect_agent_tick_inputs(&world),
            legacy_collect_agent_tick_inputs(&world)
        );

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_collect_agent_tick_inputs(&world),
            || collect_agent_tick_inputs(&world),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins14_single_pass_agent_tick_inputs nodes={} samples={} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_world_projections=2 optimized_world_projections=1 legacy_projected_nodes={} optimized_projected_nodes={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            BENCHMARK_NODE_COUNT,
            BENCHMARK_SAMPLE_COUNT,
            BENCHMARK_NODE_COUNT * 2,
            BENCHMARK_NODE_COUNT,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95,
        );
        assert!(
            optimized_p95 * 10 <= legacy_p95 * 7,
            "single-pass input P95 {optimized_p95}ns must be no more than 70% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn benchmark_world(node_count: usize) -> World {
        let mut world = World::empty();
        for index in 0..node_count {
            let entity = world.spawn_node(NodeKind::Empty);
            if index % 2 == 0 {
                world
                    .set_dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE, json!({}))
                    .unwrap();
            }
            if index % 3 == 0 {
                world
                    .set_dynamic_component(entity, NAV_MESH_OBSTACLE_COMPONENT_TYPE, json!({}))
                    .unwrap();
            }
        }
        world
    }

    fn legacy_collect_agent_tick_inputs(
        world: &World,
    ) -> (Vec<(u64, NavMeshAgentDescriptor)>, bool) {
        let agents = world
            .node_records()
            .into_iter()
            .filter_map(|node| {
                let value = world.dynamic_component(node.id, NAV_MESH_AGENT_COMPONENT_TYPE)?;
                Some((node.id, parse_component::<NavMeshAgentDescriptor>(value)))
            })
            .collect();
        let has_obstacles = world.node_records().into_iter().any(|node| {
            world
                .dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
                .is_some()
        });
        (agents, has_obstacles)
    }

    fn benchmark_paired_samples<L, O>(
        mut legacy: impl FnMut() -> L,
        mut optimized: impl FnMut() -> O,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
        let started = Instant::now();
        let result = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(&result);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        assert!(!ordered.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (ordered.len() * percentile).div_ceil(100) - 1;
        ordered[index]
    }
}
