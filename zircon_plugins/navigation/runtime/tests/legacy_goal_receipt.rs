//! A real intermediate route target is not a final navigation completion.

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

const A: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const B: Vec3 = Vec3::new(7.0, 0.0, 0.0);
const START: Vec3 = Vec3::new(0.94, 0.0, 0.08);
const GOAL: Vec3 = Vec3::new(8.0, 0.0, 0.3);
const CARVER: Vec3 = Vec3::new(-0.5, 0.0, 0.0);
const BOX_SIZE: Vec3 = Vec3::new(0.4, 2.0, 3.0);
const DT: f32 = 0.125;

#[test]
fn link_registration_is_not_arrival_and_landing_still_leads_to_the_goal() {
    let trace = trial("entry", START, GOAL, 0.2, 0.12, true, 24);
    assert_eq!(trace.positions[0], START.to_array());
    assert_eq!(
        trace.events,
        vec![
            (2, OffMeshTraverseEventKind::Started),
            (11, OffMeshTraverseEventKind::Completed)
        ]
    );
    assert_eq!(trace.positions[1], A.to_array());
    assert_eq!(trace.positions[10], B.to_array());
    assert!(
        trace.reached.iter().any(|frame| *frame > 11),
        "true goal receipt after the actual traversal is required: {trace:?}"
    );
    assert!(trace.no_path.is_empty());
    assert!(
        trace.early.is_empty(),
        "intermediate registration cannot report goal arrival: {trace:?}"
    );
}

#[test]
fn continuing_approach_cannot_report_the_final_destination() {
    // Farther from the link than its .1125 entry trigger, but inside the legacy
    // steering stopping distance. The receipt must not report a distant goal.
    let start = Vec3::new(0.75, 0.0, 0.3);
    let trace = trial("approach", start, GOAL, 0.05, 0.45, true, 4);
    assert!(trace.events.is_empty() && trace.reached.is_empty() && trace.no_path.is_empty());
    assert!(
        trace.early.is_empty(),
        "ongoing approach cannot report goal arrival: {trace:?}"
    );
}

#[test]
fn same_island_goal_still_has_a_real_arrival_receipt() {
    let start = Vec3::new(0.0, 0.0, 0.3);
    let goal = Vec3::new(0.75, 0.0, 0.3);
    let trace = trial("ground", start, goal, 0.2, 0.12, false, 8);
    assert!(trace.moved > 0 && !trace.reached.is_empty() && trace.early.is_empty());
    assert!(trace.events.is_empty() && trace.no_path.is_empty());
}

#[test]
fn masked_link_still_reports_no_path_not_goal_arrival() {
    let trace = trial("masked", START, GOAL, 0.2, 0.12, false, 4);
    assert_eq!(trace.no_path, vec![1, 2, 3, 4]);
    assert!(trace.events.is_empty() && trace.early.is_empty() && trace.reached.is_empty());
    assert_eq!(trace.moved, 0);
    assert!(trace.positions.iter().all(|p| *p == START.to_array()));
}

#[derive(Debug, Default)]
struct Trace {
    positions: Vec<[f32; 3]>,
    events: Vec<(usize, OffMeshTraverseEventKind)>,
    early: Vec<usize>,
    reached: Vec<usize>,
    no_path: Vec<usize>,
    moved: usize,
}

fn trial(
    label: &str,
    start: Vec3,
    goal: Vec3,
    radius: f32,
    tolerance: f32,
    jumps: bool,
    frames: usize,
) -> Trace {
    let manager = test_support::navigation_manager();
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    world.register_event::<OffMeshTraverseEvent>();
    let owner = spawn(&mut world, Vec3::ZERO);
    let asset = two_convex_islands(owner);
    qualify_native(label, &asset, start, goal, jumps);
    let handle = manager.load_nav_mesh(asset).unwrap();
    let carver = spawn(&mut world, CARVER);
    world
        .set_dynamic_component(
            carver,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            serde_json::to_value(NavMeshObstacleDescriptor {
                carve: true,
                size: BOX_SIZE.to_array(),
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
                radius,
                area_mask: if jumps {
                    u64::MAX
                } else {
                    u64::MAX & !(1_u64 << AREA_JUMP)
                },
                speed: 6.0,
                acceleration: 64.0,
                stopping_distance: tolerance,
                auto_braking: false,
                avoidance_quality: NavAvoidanceQuality::None,
                update_rotation: false,
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    let mut trace = Trace::default();
    for frame in 1..=frames {
        let report = manager.tick_world_agents(&mut world, DT).unwrap();
        assert_eq!(report.scanned_agents, 1);
        assert_eq!(
            report.blocked_agents,
            usize::from(label == "masked"),
            "{:?}",
            report.diagnostics
        );
        let pos = world
            .world_matrix(agent)
            .unwrap()
            .transform_point3(Vec3::ZERO);
        assert!(pos.is_finite());
        trace.positions.push(pos.to_array());
        trace.moved += report.moved_agents;
        if !report.arrived_agents.is_empty() {
            assert_eq!(report.arrived_agents, vec![(agent, goal.to_array())]);
            if pos.distance(goal) <= tolerance {
                trace.reached.push(frame);
            } else {
                trace.early.push(frame);
            }
        }
        if !report.no_path_agents.is_empty() {
            assert_eq!(report.no_path_agents, vec![(agent, goal.to_array())]);
            trace.no_path.push(frame);
        }
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
            assert_eq!((event.start, event.end), (A.to_array(), B.to_array()));
            trace.events.push((frame, event.kind));
        }
        // Stop only at genuine arrival and keep the entire prefix for comparison.
        if !trace.reached.is_empty() {
            break;
        }
    }
    let event_frames: Vec<_> = trace.events.iter().map(|e| e.0).collect();
    println!("LEGACY_GOAL_MOVE label={label} start_goal={} early={:?} reached={:?} no_path={:?} events={event_frames:?} moved={} positions={:?}",
        start.distance(goal), trace.early, trace.reached, trace.no_path, trace.moved, trace.positions);
    trace
}

fn spawn(world: &mut World, point: Vec3) -> EntityId {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .update_transform(entity, Transform::from_translation(point))
        .unwrap();
    entity
}

fn two_convex_islands(owner: EntityId) -> NavMeshAsset {
    let mut asset = support::two_island_navmesh(true);
    let [mut left, mut right] = [asset.polygons[0].clone(), asset.polygons[2].clone()];
    left.index_count = 6;
    right.index_count = 6;
    asset.polygons = vec![left, right];
    asset.tiles[0].polygon_count = 2;
    asset.off_mesh_links[0].owner_entity = owner;
    asset.off_mesh_links[0].bidirectional = false;
    asset
}

fn path(cache: &RecastTileCache, start: Vec3, end: Vec3, jumps: bool) -> NavPathResult {
    let mut query = NavPathQuery::new(start.to_array(), end.to_array());
    if !jumps {
        query.area_mask &= !(1_u64 << AREA_JUMP);
    }
    cache.find_path(&query)
}

fn assert_endpoints(result: &NavPathResult, start: Vec3, goal: Vec3) {
    assert_eq!(result.status, NavPathStatus::Complete, "{result:?}");
    assert!(result.length > 0.0 && result.points.len() >= 2);
    assert!(Vec3::from_array(result.points[0].position).distance(start) < 0.1);
    assert!(Vec3::from_array(result.points.last().unwrap().position).distance(goal) < 0.1);
}

fn qualify_native(label: &str, asset: &NavMeshAsset, start: Vec3, goal: Vec3, jumps: bool) {
    let mut cache = RecastTileCache::from_asset(asset).expect("actual native TileCache owner");
    let side = Vec3::new(-0.85, 0.0, 0.3);
    let other = Vec3::new(0.0, 0.0, 0.3);
    assert_endpoints(&path(&cache, side, other, false), side, other);
    let o = cache
        .add_obstacle(RecastNavigationObstacle::box_obstacle(
            CARVER.to_array(),
            (BOX_SIZE * 0.5).to_array(),
        ))
        .unwrap();
    cache.update().unwrap();
    assert_eq!(
        path(&cache, side, other, false).status,
        NavPathStatus::NoPath
    );
    let ordered = path(&cache, start, goal, jumps);
    let expected_masked = if label == "ground" {
        NavPathStatus::Complete
    } else {
        NavPathStatus::NoPath
    };
    assert_eq!(path(&cache, start, goal, false).status, expected_masked);
    if jumps {
        assert_endpoints(&ordered, start, goal);
        let (i, p) = ordered
            .points
            .iter()
            .enumerate()
            .find(|(_, p)| {
                p.off_mesh_link_id == Some(1) && p.flags.iter().any(|f| f == "off_mesh_link")
            })
            .unwrap_or_else(|| panic!("expected actual native marker: {ordered:?}"));
        assert!(i < 2 && ordered.visited_nodes == 3);
        assert!(Vec3::from_array(p.position).distance(A) < 0.12);
        assert!(Vec3::from_array(ordered.points[i + 1].position).distance(B) < 0.12);
        println!(
            "LEGACY_GOAL_PORTAL label={label} index={i} entry={:?} exit={:?}",
            p.position,
            ordered.points[i + 1].position
        );
    } else if label == "ground" {
        assert_endpoints(&ordered, start, goal);
        assert!(ordered
            .points
            .iter()
            .all(|point| point.off_mesh_link_id.is_none()));
    } else {
        assert_eq!(ordered.status, NavPathStatus::NoPath);
        assert_endpoints(
            &path(&cache, start, start + Vec3::Z * 0.25, false),
            start,
            start + Vec3::Z * 0.25,
        );
        assert_endpoints(
            &path(&cache, goal, goal + Vec3::Z * 0.25, false),
            goal,
            goal + Vec3::Z * 0.25,
        );
    }
    cache.remove_obstacle(o).unwrap();
    cache.update().unwrap();
    assert_endpoints(&path(&cache, side, other, false), side, other);
    println!("LEGACY_GOAL_NATIVE label={label} status={:?} masked={expected_masked:?} corridor={} cut=NoPath restored=Complete",
        ordered.status, ordered.visited_nodes);
}
