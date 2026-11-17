//! Follow the actual entry of a native off-mesh route, not the nearer authored endpoint.

#[path = "../src/tests/support.rs"]
mod support;
#[path = "../src/tests/test_support.rs"]
mod test_support;

use zircon_plugin_navigation_recast::{RecastNavigationObstacle, RecastTileCache};
use zircon_plugin_navigation_runtime::{
    navigation_component_descriptors, DefaultNavigationManager,
};
use zircon_runtime::core::framework::navigation::{
    NavAvoidanceQuality, NavMeshAgentDescriptor, NavMeshAsset, NavMeshObstacleDescriptor,
    NavPathQuery, NavPathResult, NavPathStatus, NavigationManager, OffMeshTraverseEvent,
    OffMeshTraverseEventKind, AREA_JUMP, NAV_MESH_AGENT_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{EntityId, World};

const A: Vec3 = Vec3::new(1.0, 0.0, -5.0);
const B: Vec3 = Vec3::new(7.0, 0.0, 5.0);
const CARVER: Vec3 = Vec3::new(0.0, 0.0, 6.0);
const CARVER_SIZE: Vec3 = Vec3::new(3.0, 2.0, 0.4);
const DT: f32 = 0.125;

#[test]
fn bidirectional_forward_preserves_chosen_entry() {
    let trace = trial("far_forward", Vec3::new(0.0, 0.0, 5.0), A, B, true);
    assert!(
        trace.follows_entry(A, B),
        "marked forward entry must guide the actual agent: {trace:?}"
    );
}

#[test]
fn bidirectional_reverse_preserves_chosen_entry() {
    let trace = trial("far_reverse", Vec3::new(8.0, 0.0, -5.0), B, A, true);
    assert!(
        trace.follows_entry(B, A),
        "marked reverse entry must guide the actual agent: {trace:?}"
    );
}

#[test]
fn nearer_entry_controls_keep_both_directions() {
    assert!(trial("near_forward", Vec3::new(0.0, 0.0, -3.0), A, B, true).follows_entry(A, B));
    assert!(trial("near_reverse", Vec3::new(8.0, 0.0, 3.0), B, A, true).follows_entry(B, A));
}

#[test]
fn one_way_control_keeps_forward_and_rejects_reverse() {
    assert!(trial("one_way", Vec3::new(0.0, 0.0, 5.0), A, B, false).follows_entry(A, B));
}

fn trial(label: &str, start: Vec3, entry: Vec3, exit: Vec3, bidirectional: bool) -> Trace {
    let manager = test_support::navigation_manager();
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    world.register_event::<OffMeshTraverseEvent>();
    let owner = spawn(&mut world, Vec3::ZERO);
    let asset = long_islands(owner, bidirectional);
    let goal = exit + if exit.x > entry.x { Vec3::X } else { -Vec3::X };
    let closer_to_exit = start.distance_squared(exit) < start.distance_squared(entry);
    assert_eq!(closer_to_exit, !label.starts_with("near"));
    qualify_native(label, &asset, start, goal, entry, exit, bidirectional);

    let handle = manager.load_nav_mesh(asset).unwrap();
    let carver = spawn(&mut world, CARVER);
    world
        .set_dynamic_component(
            carver,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            serde_json::to_value(NavMeshObstacleDescriptor {
                size: CARVER_SIZE.to_array(),
                carve: true,
                avoidance_enabled: false,
                ..NavMeshObstacleDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    let agent = spawn(&mut world, start);
    world
        .set_dynamic_component(
            agent,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                nav_mesh: Some(handle),
                destination: Some(goal.to_array()),
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
    let mut first = Vec3::ZERO;
    let mut gap_before_start = false;
    let mut airborne = false;
    let mut peak = 0.0_f32;
    let mut started = None;
    for frame in 0..48 {
        let report = manager.tick_world_agents(&mut world, DT).unwrap();
        assert_eq!(report.scanned_agents, 1);
        assert_eq!(
            report.blocked_agents, 0,
            "unexpected route or writeback error: {:?}",
            report.diagnostics
        );
        assert!(report.no_path_agents.is_empty());
        let position = world
            .world_matrix(agent)
            .unwrap()
            .transform_point3(Vec3::ZERO);
        if frame == 0 {
            first = position - start;
            assert!(first.length() > 0.2);
            assert!(report.off_mesh_events.is_empty());
        }
        let in_gap = (1.01..6.99).contains(&position.x);
        gap_before_start |= started.is_none() && report.off_mesh_events.is_empty() && in_gap;
        airborne |= started.is_some() && in_gap && position.y > 0.5;
        peak = peak.max(position.y);
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
            let a = Vec3::from_array(event.start);
            let b = Vec3::from_array(event.end);
            assert!((a, b) == (A, B) || (a, b) == (B, A));
            match event.kind {
                OffMeshTraverseEventKind::Started => {
                    assert!(started.replace((a, b)).is_none());
                    assert!(
                        position.distance(a) < 1e-4,
                        "Started must be written at the entry"
                    );
                }
                OffMeshTraverseEventKind::Completed => {
                    assert_eq!(
                        started,
                        Some((a, b)),
                        "first pair must be ordered and agree"
                    );
                    assert!(
                        position.distance(b) < 1e-4,
                        "Completed must be written at the landing"
                    );
                    assert!(
                        airborne && peak > 0.8,
                        "the actual parabolic stage must run"
                    );
                    let trace = Trace {
                        first,
                        dot: first.normalize_or_zero().dot((entry - start).normalize()),
                        gap_before_start,
                        event_entry: a,
                        landed: position,
                        peak,
                    };
                    println!("DIRECTED_PORTAL_MOVE label={label} first={:?} dot={} gap={} entry={:?} landed={:?} peak={}",
                        trace.first.to_array(), trace.dot, trace.gap_before_start, trace.event_entry.to_array(), trace.landed.to_array(), trace.peak);
                    // This is the first genuine crossing only. Requerying from a wrong landing
                    // exactly at a portal would measure the independent coincident-marker bug.
                    return trace;
                }
            }
        }
    }
    panic!("first real crossing did not finish: label={label} first={first:?} started={started:?} peak={peak}");
}

#[derive(Debug)]
struct Trace {
    first: Vec3,
    dot: f32,
    gap_before_start: bool,
    event_entry: Vec3,
    landed: Vec3,
    peak: f32,
}

impl Trace {
    fn follows_entry(&self, entry: Vec3, exit: Vec3) -> bool {
        self.dot > 0.99
            && !self.gap_before_start
            && self.event_entry == entry
            && self.landed.distance(exit) < 1e-4
    }
}

fn spawn(world: &mut World, position: Vec3) -> EntityId {
    let id = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .update_transform(id, Transform::from_translation(position))
        .unwrap();
    id
}

fn long_islands(owner: EntityId, bidirectional: bool) -> NavMeshAsset {
    let mut asset = support::two_island_navmesh(true);
    for v in &mut asset.vertices {
        v[2] *= 7.0;
    }
    let [mut left, mut right] = [asset.polygons[0].clone(), asset.polygons[2].clone()];
    left.index_count = 6;
    right.index_count = 6;
    asset.polygons = vec![left, right];
    asset.tiles[0].polygon_count = 2;
    asset.tiles[0].bounds_min[2] = -7.0;
    asset.tiles[0].bounds_max[2] = 7.0;
    let link = &mut asset.off_mesh_links[0];
    link.owner_entity = owner;
    link.start = A.to_array();
    link.end = B.to_array();
    link.bidirectional = bidirectional;
    asset
}

fn path(cache: &RecastTileCache, start: Vec3, end: Vec3, jump: bool) -> NavPathResult {
    let mut q = NavPathQuery::new(start.to_array(), end.to_array());
    if !jump {
        q.area_mask &= !(1_u64 << AREA_JUMP);
    }
    cache.find_path(&q)
}

fn assert_endpoints(result: &NavPathResult, start: Vec3, end: Vec3) {
    assert_eq!(result.status, NavPathStatus::Complete, "{result:?}");
    assert!(result.length > 0.0);
    assert!(Vec3::from_array(result.points.first().unwrap().position).distance(start) < 0.1);
    assert!(Vec3::from_array(result.points.last().unwrap().position).distance(end) < 0.1);
}

fn qualify_native(
    label: &str,
    asset: &NavMeshAsset,
    start: Vec3,
    goal: Vec3,
    entry: Vec3,
    exit: Vec3,
    bidirectional: bool,
) {
    // The same asset and real nonintrusive box as the manager; explicit native owner, no Rust graph.
    let mut cache = RecastTileCache::from_asset(asset).expect("native TileCache owner");
    let high = Vec3::new(0.0, 0.0, 6.7);
    let low = Vec3::new(0.0, 0.0, 5.3);
    assert_endpoints(&path(&cache, high, low, false), high, low);
    let obstacle = cache
        .add_obstacle(RecastNavigationObstacle::box_obstacle(
            CARVER.to_array(),
            (CARVER_SIZE * 0.5).to_array(),
        ))
        .expect("native box");
    cache.update().unwrap();
    assert_eq!(path(&cache, high, low, false).status, NavPathStatus::NoPath);
    for (p, q) in [
        (high, high + Vec3::X * 0.25),
        (low, low + Vec3::X * 0.25),
        (start, start + Vec3::Z * 0.25),
        (goal, goal - Vec3::Z * 0.25),
    ] {
        assert_endpoints(&path(&cache, p, q, false), p, q);
    }
    assert_eq!(
        path(&cache, start, goal, false).status,
        NavPathStatus::NoPath
    );
    let ordered = path(&cache, start, goal, true);
    assert_endpoints(&ordered, start, goal);
    assert!(ordered.visited_nodes >= 3);
    let (index, marked) = ordered
        .points
        .iter()
        .enumerate()
        .find(|(_, p)| {
            p.off_mesh_link_id == Some(1) && p.flags.iter().any(|f| f == "off_mesh_link")
        })
        .expect("the actual structured native path must mark the selected portal");
    assert!(
        index < 2,
        "the native portal must be selected by the runtime's existing take(2) policy: {ordered:?}"
    );
    assert!(
        Vec3::from_array(marked.position).distance(entry) < 0.12,
        "marked wrong native side: {ordered:?}"
    );
    let next = ordered.points[index + 1].position;
    assert!(if exit == A {
        next[0] < 1.2
    } else {
        next[0] > 6.8
    });
    let reverse = if !bidirectional {
        let opposite = path(&cache, goal, start, true);
        assert_eq!(opposite.status, NavPathStatus::NoPath);
        "NoPath"
    } else {
        "not_tested_here"
    };
    cache.remove_obstacle(obstacle).unwrap();
    cache.update().unwrap();
    assert_endpoints(&path(&cache, high, low, false), high, low);
    println!("DIRECTED_PORTAL_NATIVE label={label} corridor={} marker_index={index} marked={:?} next={next:?} length={} masked=NoPath local=Complete cut=NoPath restored=Complete reverse={reverse}",
        ordered.visited_nodes, marked.position, ordered.length);
}
