//! Check real generated world endpoints and native Detour landing, not just link counts.

#[path = "../src/tests/test_support.rs"]
mod test_support;

use zircon_plugin_navigation_recast::RecastTileCache;
use zircon_plugin_navigation_runtime::{navigation_component_descriptors, DefaultNavigationManager};
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeRequest, NavMeshLinkCapacity, NavMeshOffMeshBridgeDescriptor,
    NavMeshOffMeshLinkDescriptor, NavMeshSurfaceDescriptor, NavMeshUseGeometry, NavPathQuery,
    NavPathStatus, AREA_JUMP, NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE,
    NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE, NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Quat, Transform, Vec2, Vec3};
use zircon_runtime::scene::components::{ColliderComponent, ColliderShape, NodeKind};
use zircon_runtime::scene::{EntityId, World};

const SEPARATION: f32 = 24.0;
const LOCAL_POINT: [f32; 3] = [8.0, 0.0, 0.0];
const ROOT_2: f32 = std::f32::consts::SQRT_2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Witness {
    Link,
    Bridge,
    LinkDisabled,
}

fn spawn(world: &mut World, authored: Transform) -> EntityId {
    let node = world.spawn_node(NodeKind::Empty).unwrap();
    world.update_transform(node, authored).unwrap();
    node
}

fn anchor(world: &mut World, scale_x: f32, x_offset: f32) -> EntityId {
    // Keep both authored variants' analytic landing points on the same floors.
    let parent = spawn(
        world,
        Transform {
            translation: Vec3::X * (x_offset + (2.0 - scale_x) * 4.0 * ROOT_2),
            scale: Vec3::new(scale_x, 1.0, 1.0),
            ..Transform::default()
        },
    );
    let child = spawn(
        world,
        Transform {
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
            ..Transform::default()
        },
    );
    assert!(world.set_parent_checked(child, Some(parent)).unwrap());
    let matrix = world.world_matrix(child).unwrap();
    let column_dot = matrix.x_axis.dot(matrix.z_axis);
    assert!((column_dot - 0.5 * (scale_x * scale_x - 1.0)).abs() < 1.0e-5);
    child
}

fn add_rigid_floor(world: &mut World, top_center: Vec3) {
    let floor = spawn(
        world,
        Transform::from_translation(top_center - Vec3::Y * 0.5),
    );
    world
        .set_collider(
            floor,
            Some(ColliderComponent {
                shape: ColliderShape::Box {
                    half_extents: Vec3::new(2.0, 0.5, 1.75),
                },
                ..ColliderComponent::default()
            }),
        )
        .unwrap();
}

fn native_landing_gap(actual: [f32; 3], expected: Vec3) -> f32 {
    Vec2::new(actual[0] - expected.x, actual[2] - expected.z).length()
}

fn witness(label: &str, scale_x: f32, kind: Witness) {
    let manager = test_support::navigation_manager();
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = spawn(&mut world, Transform::default());
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            serde_json::to_value(NavMeshSurfaceDescriptor {
                use_geometry: NavMeshUseGeometry::PhysicsColliders,
                generate_links: kind != Witness::LinkDisabled,
                ..NavMeshSurfaceDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();

    // Independent scalar oracle, not WorldMatrix or the resolver under test.
    let start = Vec3::new(8.0 * ROOT_2, 0.0, -4.0 * ROOT_2);
    let end = start + Vec3::X * SEPARATION;
    add_rigid_floor(&mut world, start);
    add_rigid_floor(&mut world, end);
    let owner = anchor(&mut world, scale_x, 0.0);
    let other = anchor(&mut world, scale_x, SEPARATION);
    let (component, value) = if kind == Witness::Bridge {
        (
            NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE,
            serde_json::to_value(NavMeshOffMeshBridgeDescriptor {
                start_entity: None,
                end_entity: Some(other),
                start_local_point: LOCAL_POINT,
                end_local_point: LOCAL_POINT,
                width: 1.2,
                lane_count: 2,
                ..NavMeshOffMeshBridgeDescriptor::default()
            })
            .unwrap(),
        )
    } else {
        (
            NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
            serde_json::to_value(NavMeshOffMeshLinkDescriptor {
                start_entity: None,
                end_entity: Some(other),
                start_local_point: LOCAL_POINT,
                end_local_point: LOCAL_POINT,
                width: 0.35,
                ..NavMeshOffMeshLinkDescriptor::default()
            })
            .unwrap(),
        )
    };
    world
        .set_dynamic_component(owner, component, value)
        .unwrap();
    let (_level_owner, level) = test_support::level_from_world(world);
    let report = manager
        .bake_surface(
            &level,
            NavMeshBakeRequest {
                surface_entity: Some(surface),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        (
            report.surfaces,
            report.source_vertices,
            report.source_triangles
        ),
        (1, 8, 4)
    );
    assert!(report.baked_polygons >= 2 && report.tiles > 0);
    let asset = report.asset.expect("real baked floor and link output");
    assert!(!asset.is_empty());
    let query =
        RecastTileCache::from_asset(&asset).expect("actual native TileCache; no Rust fallback");
    for center in [start, end] {
        let to = center + Vec3::X * 0.35;
        let local = query.find_path(&NavPathQuery::new(center.to_array(), to.to_array()));
        assert_eq!(
            local.status,
            NavPathStatus::Complete,
            "both islands must exist independently: {local:?}"
        );
        assert!(native_landing_gap(local.points.last().unwrap().position, to) < 0.30);
        assert!(local.points.iter().all(|p| p.off_mesh_link_id.is_none()));
    }
    let across = query.find_path(&NavPathQuery::new(start.to_array(), end.to_array()));
    let crossed_ids: Vec<_> = across
        .points
        .iter()
        .filter_map(|point| {
            point
                .flags
                .iter()
                .any(|flag| flag == "off_mesh_link")
                .then_some(point.off_mesh_link_id)
                .flatten()
        })
        .collect();
    if kind == Witness::LinkDisabled {
        eprintln!(
            "NAV_OFF_MESH_AFFINE label={label} links={} polys={} native={:?} ids={crossed_ids:?}",
            asset.off_mesh_links.len(),
            report.baked_polygons,
            across.status
        );
        assert!(asset.off_mesh_links.is_empty());
        assert_eq!(across.status, NavPathStatus::NoPath);
        assert!(crossed_ids.is_empty());
        return;
    }

    let expected_lanes = if kind == Witness::Bridge { 2 } else { 1 };
    assert_eq!(asset.off_mesh_links.len(), expected_lanes);
    let mut max_error = 0.0_f32;
    for (lane, link) in asset.off_mesh_links.iter().enumerate() {
        let (offset, width, capacity) = if kind == Witness::Bridge {
            (
                Vec3::Z * if lane == 0 { -0.3 } else { 0.3 },
                0.6,
                NavMeshLinkCapacity::Shared {
                    group: owner,
                    limit: 2,
                },
            )
        } else {
            (Vec3::ZERO, 0.35, NavMeshLinkCapacity::Unbounded)
        };
        assert_eq!(link.owner_entity, owner);
        assert_eq!(link.lane_index, lane as u32);
        assert!(link.id != 0 && link.bidirectional && link.area == AREA_JUMP);
        assert_eq!(link.capacity, capacity);
        assert!((link.width - width).abs() < 1.0e-6);
        let (actual_start, actual_end) = (Vec3::from_array(link.start), Vec3::from_array(link.end));
        assert!(actual_start.is_finite() && actual_end.is_finite());
        max_error = max_error
            .max(actual_start.distance(start + offset))
            .max(actual_end.distance(end + offset));
        eprintln!("NAV_OFF_MESH_LANE label={label} lane={lane} id={} start={actual_start:?} end={actual_end:?}", link.id);
    }
    eprintln!("NAV_OFF_MESH_AFFINE label={label} max_error={max_error} links={} polys={} native={:?} ids={crossed_ids:?}", asset.off_mesh_links.len(), report.baked_polygons, across.status);
    assert!(
        max_error < 1.0e-4,
        "authored owner and external anchor require affine endpoint coordinates"
    );
    assert_eq!(across.status, NavPathStatus::Complete);
    assert!(across.points.len() >= 2);
    assert!(native_landing_gap(across.points.first().unwrap().position, start) < 0.30);
    assert!(native_landing_gap(across.points.last().unwrap().position, end) < 0.30);
    assert!(
        !crossed_ids.is_empty()
            && crossed_ids
                .iter()
                .all(|id| asset.off_mesh_links.iter().any(|link| link.id == *id)),
        "the actual complete Detour path must use a published link: {across:?}"
    );
}

#[test]
fn rigid_owner_and_external_anchor_keep_a_complete_native_path() {
    witness("rigid", 1.0, Witness::Link);
}

#[test]
fn shear_owner_and_external_anchor_keep_a_complete_native_path() {
    witness("link", 2.0, Witness::Link);
}

#[test]
fn shear_bridge_publishes_both_affine_lanes_and_connects_the_islands() {
    witness("bridge", 2.0, Witness::Bridge);
}

#[test]
fn disabling_links_keeps_the_two_real_islands_disconnected() {
    witness("disabled", 2.0, Witness::LinkDisabled);
}
