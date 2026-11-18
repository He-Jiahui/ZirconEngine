//! Native-backed regression of actual world entry, arc and landing writes.

#[path = "../src/tests/support.rs"]
mod support;
#[path = "../src/tests/test_support.rs"]
mod test_support;

use zircon_plugin_navigation_recast::{RecastNavigationObstacle, RecastTileCache};
use zircon_plugin_navigation_runtime::navigation_component_descriptors;
use zircon_runtime::core::framework::navigation::{
    NavAvoidanceQuality, NavMeshAgentDescriptor, NavMeshAsset, NavMeshHandle,
    NavMeshObstacleDescriptor, NavPathQuery, NavPathResult, NavPathStatus, NavigationManager,
    OffMeshTraverseEvent, OffMeshTraverseEventKind, AREA_JUMP, NAV_MESH_AGENT_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Mat4, Quat, Transform, Vec3};
use zircon_runtime::scene::components::{Mobility, NodeKind};
use zircon_runtime::scene::{EntityId, World};

const DT: f32 = 0.125;
const EPS: f32 = 1e-4;
const A: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const B: Vec3 = Vec3::new(7.0, 0.0, 0.0);
const S: Vec3 = Vec3::new(0.94, 0.0, 0.08);
const GOAL: Vec3 = Vec3::new(8.0, 0.0, 0.3);
const OFFSET: Vec3 = Vec3::new(10.0, 0.0, -3.0);
const CARVER: Vec3 = Vec3::new(-0.5, 0.0, 0.0);
const CARVER_SIZE: Vec3 = Vec3::new(0.4, 2.0, 3.0);

#[test]
fn translated_parent_writes_direct_stage_in_world_space() {
    let trace = run_jump("translation", Placement::Translated);
    assert!(
        trace.good(),
        "translated direct writeback must match the actual world arc: {trace:?}"
    );
}

#[test]
fn sheared_parent_writes_direct_stage_without_changing_child_basis() {
    let trace = run_jump("shear", Placement::Sheared);
    assert!(trace.good(), "sheared direct writeback must match the actual world arc and preserve signed basis: {trace:?}");
}

#[test]
fn unparented_arc_and_endpoint_events_stay_exact() {
    let trace = run_jump("root", Placement::Root);
    assert!(trace.good(), "root arc must remain exact: {trace:?}");
}

#[test]
fn rejected_direct_writes_do_not_publish_success_events() {
    for (label, rejected_frame) in [("reject_started", 2), ("reject_completed", 11)] {
        let mut f = Fixture::new(label, Placement::Root);
        f.arm();
        for frame in 2..rejected_frame {
            let report = f.tick();
            assert_eq!(report.blocked_agents, 0);
            if frame == 2 {
                f.event(&report.off_mesh_events, OffMeshTraverseEventKind::Started);
            } else {
                assert!(report.off_mesh_events.is_empty());
            }
        }
        f.world.set_mobility(f.agent, Mobility::Static).unwrap();
        let before = f.matrix();
        let report = f.tick();
        assert_eq!(report.blocked_agents, 1);
        assert_eq!(report.moved_agents, 0);
        assert!(report.off_mesh_events.is_empty());
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.contains("off-mesh traversal failed")));
        assert_eq!(f.matrix(), before);
        println!("DIRECT_AFFINE_REJECT label={label} frame={rejected_frame} blocked=1 moved=0 events=0 unchanged=true");
    }
}

#[derive(Clone, Copy)]
enum Placement {
    Root,
    Translated,
    Sheared,
}

impl Placement {
    // Independent reference for P^(-1) as an ordinary point. P = diag(2,1,1)*Ry45.
    fn expected_local(self, p: Vec3) -> Vec3 {
        let r = std::f32::consts::FRAC_1_SQRT_2;
        match self {
            Self::Root => p,
            Self::Translated => p - OFFSET,
            Self::Sheared => Vec3::new((0.5 * p.x - p.z) * r, p.y, (0.5 * p.x + p.z) * r),
        }
    }
}

struct Fixture {
    manager: test_support::TestNavigationManager,
    world: World,
    handle: NavMeshHandle,
    agent: EntityId,
    owner: EntityId,
    ancestors: Vec<(EntityId, Mat4)>,
}

impl Fixture {
    fn new(label: &str, placement: Placement) -> Self {
        let manager = test_support::navigation_manager();
        let mut world = World::empty();
        for descriptor in navigation_component_descriptors() {
            world.register_component_type(descriptor).unwrap();
        }
        world.register_event::<OffMeshTraverseEvent>();
        let owner = spawn(&mut world, Transform::IDENTITY);
        let mut asset = support::two_island_navmesh(true);
        // Keep the original indices but represent each convex island as one polygon.
        let [mut left, mut right] = [asset.polygons[0].clone(), asset.polygons[2].clone()];
        left.index_count = 6;
        right.index_count = 6;
        asset.polygons = vec![left, right];
        asset.tiles[0].polygon_count = 2;
        asset.off_mesh_links[0].owner_entity = owner;
        asset.off_mesh_links[0].bidirectional = false;
        qualify_native(label, &asset);
        let handle = manager.load_nav_mesh(asset).unwrap();
        let carver = spawn(&mut world, Transform::from_translation(CARVER));
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

        let parents = match placement {
            Placement::Root => vec![],
            Placement::Translated => vec![spawn(&mut world, Transform::from_translation(OFFSET))],
            Placement::Sheared => {
                let g = spawn(
                    &mut world,
                    Transform {
                        scale: Vec3::new(2.0, 1.0, 1.0),
                        ..Transform::IDENTITY
                    },
                );
                let p = spawn(
                    &mut world,
                    Transform {
                        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
                        ..Transform::IDENTITY
                    },
                );
                world.set_parent_checked(p, Some(g)).unwrap();
                let m = world.world_matrix(p).unwrap();
                assert!((m.determinant() - 2.0).abs() < EPS);
                assert!(
                    (m.x_axis.dot(m.z_axis) - 1.5).abs() < EPS,
                    "must create the D*Ry shear"
                );
                vec![g, p]
            }
        };
        let authored_rotation = Quat::from_rotation_z(std::f32::consts::PI / 12.0);
        let sx = if matches!(placement, Placement::Sheared) {
            -1.25
        } else {
            1.25
        };
        let agent = spawn(
            &mut world,
            Transform {
                translation: placement.expected_local(S),
                rotation: authored_rotation,
                scale: Vec3::new(sx, 1.0, 0.75),
            },
        );
        if let Some(&parent) = parents.last() {
            world.set_parent_checked(agent, Some(parent)).unwrap();
        }
        assert!(
            world
                .world_matrix(agent)
                .unwrap()
                .transform_point3(Vec3::ZERO)
                .distance(S)
                < EPS
        );
        world
            .set_dynamic_component(
                agent,
                NAV_MESH_AGENT_COMPONENT_TYPE,
                serde_json::to_value(NavMeshAgentDescriptor {
                    nav_mesh: Some(handle),
                    destination: Some(GOAL.to_array()),
                    update_rotation: false,
                    avoidance_quality: NavAvoidanceQuality::None,
                    radius: 0.2,
                    speed: 6.0,
                    acceleration: 64.0,
                    stopping_distance: 0.12,
                    ..NavMeshAgentDescriptor::default()
                })
                .unwrap(),
            )
            .unwrap();
        let ancestors = parents
            .into_iter()
            .map(|p| (p, world.world_matrix(p).unwrap()))
            .collect();
        Self {
            manager,
            world,
            handle,
            agent,
            owner,
            ancestors,
        }
    }

    fn matrix(&self) -> Mat4 {
        self.world.world_matrix(self.agent).unwrap()
    }

    fn tick(&mut self) -> zircon_runtime::core::framework::navigation::NavAgentTickReport {
        let report = self.manager.tick_world_agents(&mut self.world, DT).unwrap();
        assert_eq!(report.scanned_agents, 1);
        assert!(report.no_path_agents.is_empty());
        for &(ancestor, expected) in &self.ancestors {
            assert_eq!(self.world.world_matrix(ancestor), Some(expected));
        }
        report
    }

    fn arm(&mut self) {
        let before = self.matrix();
        let local = self.world.local_transform(self.agent).unwrap();
        let report = self.tick();
        assert_eq!(
            report.moved_agents, 0,
            "ordinary movement must not contaminate this direct write"
        );
        assert_eq!(report.blocked_agents, 0, "{:?}", report.diagnostics);
        assert!(report.off_mesh_events.is_empty());
        assert_eq!(self.matrix(), before);
        assert_eq!(self.world.local_transform(self.agent), Some(local));
    }

    fn event(&self, events: &[OffMeshTraverseEvent], kind: OffMeshTraverseEventKind) {
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.kind, kind);
        assert_eq!(
            (
                event.agent_entity,
                event.nav_mesh,
                event.owner_entity,
                event.link_id
            ),
            (self.agent, self.handle, self.owner, 1)
        );
        assert_eq!((event.start, event.end), (A.to_array(), B.to_array()));
    }
}

#[derive(Debug, Default)]
struct Trace {
    start: Vec3,
    apex: Vec3,
    end: Vec3,
    max_point: f32,
    max_local: f32,
    max_basis: f32,
    max_step: f32,
    authored_same: bool,
    det_end: f32,
}

impl Trace {
    fn good(&self) -> bool {
        self.max_point < EPS
            && self.max_local < EPS
            && self.max_basis < EPS
            && self.max_step < EPS
            && self.authored_same
    }
}

fn run_jump(label: &str, placement: Placement) -> Trace {
    let mut f = Fixture::new(label, placement);
    let basis = f.matrix();
    let authored = f.world.local_transform(f.agent).unwrap();
    if matches!(placement, Placement::Sheared) {
        assert!((basis.determinant() + 1.875).abs() < EPS);
    }
    f.arm();
    let mut trace = Trace {
        authored_same: true,
        ..Trace::default()
    };
    let mut previous = basis.transform_point3(Vec3::ZERO);
    let mut previous_expected = S;
    for frame in 2..=11 {
        let report = f.tick();
        assert_eq!(report.blocked_agents, 0, "{:?}", report.diagnostics);
        match frame {
            2 => f.event(&report.off_mesh_events, OffMeshTraverseEventKind::Started),
            11 => f.event(&report.off_mesh_events, OffMeshTraverseEventKind::Completed),
            _ => assert!(report.off_mesh_events.is_empty()),
        }
        let t = (frame - 2).min(8) as f32 / 8.0;
        let expected = Vec3::new(1.0 + 6.0 * t, 4.0 * t * (1.0 - t), 0.0);
        let current = f.matrix();
        let p = current.transform_point3(Vec3::ZERO);
        let local = f.world.local_transform(f.agent).unwrap();
        assert!(p.is_finite());
        trace.max_point = trace.max_point.max(p.distance(expected));
        trace.max_local = trace.max_local.max(
            local
                .translation
                .distance(placement.expected_local(expected)),
        );
        trace.max_step = trace
            .max_step
            .max(((p - previous) - (expected - previous_expected)).length());
        for (after, before) in [
            (current.x_axis, basis.x_axis),
            (current.y_axis, basis.y_axis),
            (current.z_axis, basis.z_axis),
        ] {
            trace.max_basis = trace.max_basis.max((after - before).length());
        }
        trace.authored_same &= local.scale.distance(authored.scale) < EPS
            && (local.rotation.dot(authored.rotation).abs() - 1.0).abs() < EPS;
        previous = p;
        previous_expected = expected;
        match frame {
            2 => trace.start = p,
            6 => trace.apex = p,
            11 => {
                trace.end = p;
                trace.det_end = current.determinant();
            }
            _ => {}
        }
    }
    println!("DIRECT_AFFINE_MOVE label={label} start={:?} apex={:?} end={:?} max_point={} max_local={} max_basis={} max_step={} authored={} det_end={} started=2 completed=11",
        trace.start.to_array(), trace.apex.to_array(), trace.end.to_array(), trace.max_point, trace.max_local,
        trace.max_basis, trace.max_step, trace.authored_same, trace.det_end);
    trace
}

fn spawn(world: &mut World, transform: Transform) -> EntityId {
    let id = world.spawn_node(NodeKind::Empty).unwrap();
    world.update_transform(id, transform).unwrap();
    id
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

fn qualify_native(label: &str, asset: &NavMeshAsset) {
    let mut c = RecastTileCache::from_asset(asset).expect("native TileCache owner");
    let left = Vec3::new(-0.85, 0.0, 0.3);
    let right = Vec3::new(0.0, 0.0, 0.3);
    assert_endpoints(&path(&c, left, right, false), left, right);
    let o = c
        .add_obstacle(RecastNavigationObstacle::box_obstacle(
            CARVER.to_array(),
            (CARVER_SIZE * 0.5).to_array(),
        ))
        .unwrap();
    c.update().unwrap();
    assert_eq!(path(&c, left, right, false).status, NavPathStatus::NoPath);
    assert_eq!(path(&c, S, GOAL, false).status, NavPathStatus::NoPath);
    assert_endpoints(&path(&c, S, S + Vec3::Z * 0.3, false), S, S + Vec3::Z * 0.3);
    assert_endpoints(
        &path(&c, GOAL, GOAL + Vec3::Z * 0.3, false),
        GOAL,
        GOAL + Vec3::Z * 0.3,
    );
    let ordered = path(&c, S, GOAL, true);
    assert_endpoints(&ordered, S, GOAL);
    let (i, marker) = ordered
        .points
        .iter()
        .enumerate()
        .find(|(_, p)| {
            p.off_mesh_link_id == Some(1) && p.flags.iter().any(|f| f == "off_mesh_link")
        })
        .unwrap_or_else(|| panic!("expected native marker: {ordered:?}"));
    assert!(i < 2 && ordered.visited_nodes == 3, "{ordered:?}");
    let next = ordered.points[i + 1].position;
    assert!(Vec3::from_array(marker.position).distance(A) < 0.12);
    assert!(Vec3::from_array(next).distance(B) < 0.12);
    assert_eq!(path(&c, GOAL, S, true).status, NavPathStatus::NoPath);
    c.remove_obstacle(o).unwrap();
    c.update().unwrap();
    assert_endpoints(&path(&c, left, right, false), left, right);
    println!("DIRECT_AFFINE_NATIVE label={label} corridor={} index={i} marked={:?} next={next:?} masked=NoPath cut=NoPath restored=Complete reverse=NoPath",
        ordered.visited_nodes, marker.position);
}
