//! Exercise collection in a real bake, then query the selected floors in native TileCache.

#[path = "../src/tests/test_support.rs"]
mod test_support;

use zircon_plugin_navigation_recast::RecastTileCache;
use zircon_plugin_navigation_runtime::{
    navigation_component_descriptors, DefaultNavigationManager,
};
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeRequest, NavMeshCollectMode, NavMeshSurfaceDescriptor, NavMeshUseGeometry,
    NavPathQuery, NavPathResult, NavPathStatus, NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Quat, Transform, Vec2, Vec3};
use zircon_runtime::scene::components::{ColliderComponent, ColliderShape, NodeKind};
use zircon_runtime::scene::{EntityId, World};

const CENTER: Vec3 = Vec3::new(16.0, 0.0, 0.0);
const OFFSET: Vec3 = Vec3::new(0.0, 0.0, 16.0);
const SIZE: [f32; 3] = [40.0, 8.0, 16.0];
const TARGET_X: f32 = 32.0;
const OUTSIDE_X: f32 = 128.0;

#[derive(Clone, Copy)]
enum Placement {
    RigidFull,
    RigidSubset,
    Shear,
}

fn spawn(world: &mut World, transform: Transform) -> EntityId {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world.update_transform(entity, transform).unwrap();
    entity
}

fn surface_node(world: &mut World, placement: Placement) -> EntityId {
    let (node, expected_center, column_dot) = match placement {
        Placement::RigidFull => (
            spawn(world, Transform::from_translation(CENTER - OFFSET)),
            CENTER,
            0.0,
        ),
        Placement::RigidSubset => {
            // An independent rigid control collects only the shared sentinel.
            let subset_center = Vec3::Z * 4.0;
            (
                spawn(world, Transform::from_translation(subset_center - OFFSET)),
                subset_center,
                0.0,
            )
        }
        Placement::Shear => {
            let angle = std::f32::consts::FRAC_PI_4;
            let parent = spawn(
                world,
                Transform {
                    translation: Vec3::NEG_Z * 32.0,
                    rotation: Quat::from_rotation_y(-angle),
                    scale: Vec3::new(3.0, 1.0, 1.0),
                },
            );
            let child = spawn(
                world,
                Transform {
                    rotation: Quat::from_rotation_y(angle),
                    ..Transform::default()
                },
            );
            assert!(world.set_parent_checked(child, Some(parent)).unwrap());
            // Scalar Rxz(-45) * diag(3,1) * Rxz(45) has rows (2,1),(1,2).
            // Its Z column times 16 plus (0,0,-32) gives (16,0,0).
            (child, CENTER, 4.0)
        }
    };
    let matrix = world.world_matrix(node).unwrap();
    assert!((matrix.x_axis.dot(matrix.z_axis) - column_dot).abs() < 1e-5);
    assert!((matrix.transform_point3(OFFSET) - expected_center).length() < 1e-5);
    if matches!(placement, Placement::Shear) {
        // Diagnostic only; neither expected membership nor native queries use projected TRS.
        let projected = world
            .world_transform(node)
            .unwrap()
            .matrix()
            .transform_point3(OFFSET);
        println!(
            "NAV_BAKE_VOLUME_PROJECTION x={} z={} gap={}",
            projected.x,
            projected.z,
            (projected - expected_center).length()
        );
    }
    node
}

fn add_floor(world: &mut World, x: f32) {
    let entity = spawn(world, Transform::from_translation(Vec3::new(x, -0.5, 0.0)));
    world
        .set_collider(
            entity,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::new(2.0, 0.5, 2.0),
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();
}

fn path(cache: &RecastTileCache, x: f32) -> NavPathResult {
    cache.find_path(&NavPathQuery::new([x - 1.0, 0.0, 0.0], [x + 1.0, 0.0, 0.0]))
}

fn check_inside(path: &NavPathResult, x: f32) {
    assert_eq!(
        path.status,
        NavPathStatus::Complete,
        "floor at x={x}: {path:?}"
    );
    assert!(path.visited_nodes > 0 && path.points.len() >= 2 && path.length > 1.5);
    let start = path.points.first().unwrap().position;
    let end = path.points.last().unwrap().position;
    assert!(Vec2::new(start[0] - (x - 1.0), start[2]).length() < 0.25);
    assert!(Vec2::new(end[0] - (x + 1.0), end[2]).length() < 0.25);
    assert!(path
        .points
        .iter()
        .all(|point| point.off_mesh_link_id.is_none()));
}

fn require_membership(label: &str, placement: Placement, target_included: bool) {
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = surface_node(&mut world, placement);
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            serde_json::to_value(NavMeshSurfaceDescriptor {
                collect_mode: NavMeshCollectMode::Volume,
                volume_center: OFFSET.to_array(),
                volume_size: SIZE,
                use_geometry: NavMeshUseGeometry::PhysicsColliders,
                generate_links: false,
                ..Default::default()
            })
            .unwrap(),
        )
        .unwrap();
    for x in [0.0, TARGET_X, OUTSIDE_X] {
        add_floor(&mut world, x);
    }
    let (_level_owner, level) = test_support::level_from_world(world);
    let manager = test_support::navigation_manager();
    let report = manager
        .bake_surface(
            &level,
            NavMeshBakeRequest {
                surface_entity: Some(surface),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(report.surfaces, 1);
    assert!(matches!(
        (report.source_vertices, report.source_triangles),
        (4, 2) | (8, 4)
    ));
    assert!(report.baked_polygons > 0 && report.tiles > 0);
    let sources = report
        .diagnostics
        .iter()
        .find(|entry| entry.entity == Some(surface) && entry.message.starts_with("collected "))
        .expect("actual source collection diagnostic");
    assert!(report
        .diagnostics
        .iter()
        .all(|entry| !entry.message.contains("render mesh fallback")));
    let asset = report.asset.unwrap();
    assert!(!asset.is_empty() && asset.off_mesh_links.is_empty());
    let max_x = asset
        .vertices
        .iter()
        .map(|p| p[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let native =
        RecastTileCache::from_asset(&asset).expect("actual native query; no Rust fallback");
    let sentinel = path(&native, 0.0);
    let target = path(&native, TARGET_X);
    let outside = path(&native, OUTSIDE_X);
    println!("NAV_BAKE_VOLUME label={label} vertices={} triangles={} polygons={} max_x={max_x} sentinel={:?} target={:?} outside={:?} diagnostic={:?}",
        report.source_vertices, report.source_triangles, report.baked_polygons,
        sentinel.status, target.status, outside.status, sources.message);
    check_inside(&sentinel, 0.0);
    assert_eq!(outside.status, NavPathStatus::NoPath);
    if target_included {
        assert_eq!(
            target.status,
            NavPathStatus::Complete,
            "the source at x=32 within the analytic collection volume must be walkable"
        );
        check_inside(&target, TARGET_X);
        assert!(max_x > TARGET_X && max_x < TARGET_X + 2.1);
        assert_eq!((report.source_vertices, report.source_triangles), (8, 4));
    } else {
        assert_eq!(target.status, NavPathStatus::NoPath);
        assert!(max_x > 0.0 && max_x < 2.1);
        assert_eq!((report.source_vertices, report.source_triangles), (4, 2));
    }
}

#[test]
fn rigid_volume_collects_both_analytic_inside_floors() {
    require_membership("rigid", Placement::RigidFull, true);
}

#[test]
fn rigid_subset_keeps_only_the_shared_native_sentinel() {
    require_membership("subset", Placement::RigidSubset, false);
}

#[test]
fn sheared_volume_collects_both_analytic_inside_floors() {
    require_membership("shear", Placement::Shear, true);
}
