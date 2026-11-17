//! Exercise per-agent traversal policy on the same live carved navigation handle.

#[path = "../src/tests/support.rs"]
mod support;
#[path = "../src/tests/test_support.rs"]
mod test_support;

use zircon_plugin_navigation_runtime::{
    navigation_component_descriptors, DefaultNavigationManager,
};
use zircon_runtime::core::framework::navigation::{
    NavAvoidanceQuality, NavLinkTraversalMode, NavMeshAgentDescriptor, NavMeshHandle,
    NavMeshObstacleDescriptor, NavPathQuery, NavPathStatus, NavigationManager,
    OffMeshTraverseEvent, OffMeshTraverseEventKind, NAV_MESH_AGENT_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{EntityId, World};

const START: Vec3 = Vec3::new(0.0, 0.0, 0.3);
const GOAL: Vec3 = Vec3::new(8.0, 0.0, 0.3);
const LINK_ENTRY: [f32; 3] = [1.0, 0.0, 0.0];
const LINK_EXIT: [f32; 3] = [7.0, 0.0, 0.0];
const FRAMES: usize = 48;
const DT: f32 = 0.125;

#[test]
fn carved_cache_does_not_hide_links_after_opt_out() {
    let [first, second] = observe_order("deny_allow", true, false);
    assert!(
        first.blocked_at_start(START),
        "first-use opt-out control: {first:?}"
    );
    assert!(
        second.crossed_link(),
        "current allowed agent must not inherit the first opt-out: {second:?}"
    );
}

#[test]
fn carved_cache_respects_opt_out_after_allowed_agent() {
    let [first, second] = observe_order("allow_deny", true, true);
    assert!(
        first.crossed_link(),
        "first-use enabled crossing control: {first:?}"
    );
    assert!(
        second.blocked_at_start(START),
        "current opt-out must not inherit the first allowed link: {second:?}"
    );
}

#[test]
fn uncarved_link_policy_is_order_independent() {
    for (order, allow) in [("deny_allow", false), ("allow_deny", true)] {
        let [first, second] = observe_order(order, false, allow);
        let (allowed, denied) = if allow {
            (first, second)
        } else {
            (second, first)
        };
        assert!(allowed.crossed_link(), "uncarved allowed path: {allowed:?}");
        assert!(
            denied.blocked_at_start(START),
            "uncarved opt-out: {denied:?}"
        );
    }
}

#[test]
fn manual_link_remains_explicit_with_live_carving() {
    let manager = test_support::navigation_manager();
    let mut world = world();
    let owner = spawn(&mut world, Vec3::ZERO);
    let mut asset = support::two_island_navmesh(true);
    asset.off_mesh_links[0].owner_entity = owner;
    asset.off_mesh_links[0].traversal_mode = NavLinkTraversalMode::Manual;
    let handle = manager.load_nav_mesh(asset).unwrap();
    add_carver(&mut world);
    assert_explicit_link(&manager, handle);
    let agent = spawn_agent(&mut world, handle, START, GOAL, true);
    let run = observe(&manager, &mut world, handle, owner, agent, GOAL);
    run.log("manual", true, "only", true);
    assert!(
        run.blocked_at_start(START),
        "manual link was auto-traversed: {run:?}"
    );
}

fn world() -> World {
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    world.register_event::<OffMeshTraverseEvent>();
    world
}

fn spawn(world: &mut World, position: Vec3) -> EntityId {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .update_transform(entity, Transform::from_translation(position))
        .unwrap();
    entity
}

fn add_carver(world: &mut World) -> EntityId {
    // A real cut through the outer left edge, not in the tested forward crossing.
    let entity = spawn(world, Vec3::new(-0.5, 0.0, 0.0));
    let carver = NavMeshObstacleDescriptor {
        size: [0.4, 2.0, 3.0],
        carve: true,
        avoidance_enabled: false,
        ..NavMeshObstacleDescriptor::default()
    };
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            serde_json::to_value(carver).unwrap(),
        )
        .unwrap();
    entity
}

fn spawn_agent(
    world: &mut World,
    handle: NavMeshHandle,
    start: Vec3,
    goal: Vec3,
    allow: bool,
) -> EntityId {
    let entity = spawn(world, start);
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                nav_mesh: Some(handle),
                destination: Some(goal.to_array()),
                auto_traverse_links: allow,
                avoidance_quality: NavAvoidanceQuality::None,
                update_rotation: false,
                speed: 8.0,
                acceleration: 64.0,
                stopping_distance: 0.05,
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    entity
}

fn retire(world: &mut World, agent: EntityId) {
    assert!(world
        .remove_dynamic_component(agent, NAV_MESH_AGENT_COMPONENT_TYPE)
        .unwrap());
}

fn assert_explicit_link(manager: &DefaultNavigationManager, handle: NavMeshHandle) {
    let mut query = NavPathQuery::new(START.to_array(), GOAL.to_array());
    query.nav_mesh = Some(handle);
    let path = manager.find_path(query).unwrap();
    assert_eq!(path.status, NavPathStatus::Complete);
    assert!(path.length > 7.9 && path.visited_nodes >= 3);
    assert!((Vec3::from_array(path.points.first().unwrap().position) - START).length() < 1e-4);
    assert!((Vec3::from_array(path.points.last().unwrap().position) - GOAL).length() < 1e-4);
    assert!(path
        .points
        .iter()
        .any(|p| p.off_mesh_link_id == Some(1) && p.flags.iter().any(|f| f == "off_mesh_link")));
}

fn observe_order(order: &str, carved: bool, first_allow: bool) -> [Trace; 2] {
    let manager = test_support::navigation_manager();
    let mut world = world();
    let owner = spawn(&mut world, Vec3::ZERO);
    let mut asset = support::two_island_navmesh(true);
    asset.off_mesh_links[0].owner_entity = owner;
    let handle = manager.load_nav_mesh(asset).unwrap();
    let carver = carved.then(|| add_carver(&mut world));
    assert_explicit_link(&manager, handle);

    let first = spawn_agent(&mut world, handle, START, GOAL, first_allow);
    let first_trace = observe(&manager, &mut world, handle, owner, first, GOAL);
    first_trace.log(order, carved, "first", first_allow);
    retire(&mut world, first);
    assert_eq!(
        manager
            .tick_world_agents(&mut world, DT)
            .unwrap()
            .scanned_agents,
        0
    );

    let second = spawn_agent(&mut world, handle, START, GOAL, !first_allow);
    assert_ne!(first, second);
    let second_trace = observe(&manager, &mut world, handle, owner, second, GOAL);
    second_trace.log(order, carved, "second", !first_allow);
    retire(&mut world, second);

    if let Some(carver) = carver {
        // Verify actual carving and removal on this manager and handle *after* both seeds.
        // The strip isolates a small left pocket but leaves the two test endpoints alone.
        let pocket = Vec3::new(-0.85, 0.0, 0.3);
        let side_agent = spawn_agent(&mut world, handle, pocket, START, false);
        let cut = observe(&manager, &mut world, handle, owner, side_agent, START);
        cut.log(order, carved, "cut", false);
        assert!(
            cut.blocked_at_start(pocket),
            "live carving side route: {cut:?}"
        );
        assert!(world
            .remove_dynamic_component(carver, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
            .unwrap());
        let restored = observe(&manager, &mut world, handle, owner, side_agent, START);
        restored.log(order, carved, "restored", false);
        assert!(
            restored.ground_arrived(START),
            "removing the last carver must restore the ground route: {restored:?}"
        );
    }
    [first_trace, second_trace]
}

#[derive(Debug, Default)]
struct Trace {
    end: Vec3,
    moved: usize,
    blocked: usize,
    no_path: usize,
    max_y: f32,
    airborne_between_ends: bool,
    landed: bool,
    events: Vec<OffMeshTraverseEventKind>,
}

impl Trace {
    fn blocked_at_start(&self, start: Vec3) -> bool {
        self.end == start
            && self.moved == 0
            && self.blocked == FRAMES
            && self.no_path == FRAMES
            && self.events.is_empty()
    }

    fn ground_arrived(&self, goal: Vec3) -> bool {
        self.end.distance(goal) < 0.08
            && self.moved > 0
            && self.blocked == 0
            && self.no_path == 0
            && self.events.is_empty()
    }

    fn crossed_link(&self) -> bool {
        self.end.distance(GOAL) < 0.08
            && self.moved > 0
            && self.blocked == 0
            && self.no_path == 0
            && self.events
                == [
                    OffMeshTraverseEventKind::Started,
                    OffMeshTraverseEventKind::Completed,
                ]
            && self.max_y > 0.8
            && self.airborne_between_ends
            && self.landed
    }

    fn log(&self, order: &str, carved: bool, phase: &str, allow: bool) {
        println!("CARVED_LINK_POLICY order={order} carved={carved} phase={phase} allow={allow} end={:?} moved={} blocked={} no_path={} peak={} airborne={} landed={} events={:?}",
            self.end.to_array(), self.moved, self.blocked, self.no_path, self.max_y, self.airborne_between_ends, self.landed, self.events);
    }
}

fn observe(
    manager: &DefaultNavigationManager,
    world: &mut World,
    handle: NavMeshHandle,
    owner: EntityId,
    agent: EntityId,
    goal: Vec3,
) -> Trace {
    let mut trace = Trace::default();
    for _ in 0..FRAMES {
        let report = manager.tick_world_agents(world, DT).unwrap();
        assert_eq!(report.scanned_agents, 1);
        assert!(
            report.no_path_agents.is_empty() || report.no_path_agents == [(agent, goal.to_array())]
        );
        trace.moved += report.moved_agents;
        trace.blocked += report.blocked_agents;
        trace.no_path += report.no_path_agents.len();
        trace.end = world
            .world_matrix(agent)
            .unwrap()
            .transform_point3(Vec3::ZERO);
        trace.max_y = trace.max_y.max(trace.end.y);
        trace.airborne_between_ends |= (1.1..6.9).contains(&trace.end.x) && trace.end.y > 0.25;
        for event in report.off_mesh_events {
            assert_eq!(
                (
                    event.agent_entity,
                    event.nav_mesh,
                    event.owner_entity,
                    event.link_id
                ),
                (agent, handle, owner, 1)
            );
            assert_eq!((event.start, event.end), (LINK_ENTRY, LINK_EXIT));
            if event.kind == OffMeshTraverseEventKind::Completed {
                trace.landed |= trace.end.distance(Vec3::from_array(LINK_EXIT)) < 1e-4;
            }
            trace.events.push(event.kind);
        }
    }
    trace
}
