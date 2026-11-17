//! Query the actual Recast output of the same attached floor from both bake inputs.

#[path = "../src/tests/test_support.rs"]
mod test_support;

use zircon_plugin_navigation_runtime::{
    navigation_component_descriptors, DefaultNavigationManager,
};
use zircon_runtime::asset::{AssetUri, MeshAsset, MeshVertex, ModelPrimitiveAsset};
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeRequest, NavMeshSurfaceDescriptor, NavMeshUseGeometry, NavRaycastQuery,
    NavRaycastResult, NavSampleHit, NavSampleQuery, NavigationManager, AREA_WALKABLE,
    DEFAULT_AGENT_TYPE, DEFAULT_AREA_MASK, NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Quat, Transform, Vec2, Vec3};
use zircon_runtime::core::resource::{
    MeshMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use zircon_runtime::scene::components::{ColliderComponent, ColliderShape, MeshRenderer, NodeKind};
use zircon_runtime::scene::{EntityId, World};

const HALF: Vec3 = Vec3::new(24.0, 0.5, 8.0);
const OFFSET: Vec3 = Vec3::new(3.0, 0.25, -2.0);
const ORIGIN: Vec3 = Vec3::new(5.0, 2.0, -3.0);

fn spawn(world: &mut World, transform: Transform) -> EntityId {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world.update_transform(entity, transform).unwrap();
    entity
}

fn attach_render_floor(
    manager: &test_support::TestNavigationManager,
    world: &mut World,
    floor: EntityId,
) {
    let locator = AssetUri::parse("mem://nav-affine-test/top-quad.zmesh").unwrap();
    let id = ResourceId::from_locator(&locator);
    let vertices = [
        Vec3::new(-HALF.x, HALF.y, -HALF.z),
        Vec3::new(HALF.x, HALF.y, -HALF.z),
        Vec3::new(HALF.x, HALF.y, HALF.z),
        Vec3::new(-HALF.x, HALF.y, HALF.z),
    ]
    .map(|point| MeshVertex::new(point + OFFSET, Vec3::Y, Vec2::ZERO));
    let primitive = ModelPrimitiveAsset {
        vertices: vertices.to_vec(),
        indices: vec![0, 1, 2, 0, 2, 3],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };
    let assets = manager.project_assets();
    let mesh = MeshAsset::from_model_primitive(locator.clone(), &primitive);
    assets
        .assets::<MeshAsset>()
        .insert(ResourceRecord::new(id, ResourceKind::Mesh, locator), mesh)
        .unwrap();
    let lease = assets
        .acquire_mesh_asset(id)
        .expect("the bake must have a real, resident typed mesh");
    assert_eq!(lease.to_model_primitive().unwrap().vertices.len(), 4);
    let renderer = MeshRenderer {
        mesh: Some(ResourceHandle::<MeshMarker>::new(id)),
        ..MeshRenderer::default()
    };
    world.insert(floor, renderer).unwrap();
    assert!(world.collider(floor).is_none());
}

fn point_on_top(x: f32, z: f32, scale_x: f32) -> Vec3 {
    // Analytic A = diag(scale_x,1,1) * Ry(45deg). The independent
    // query oracle does not use the World TRS view or its reconstruction.
    let p = Vec3::new(x, HALF.y, z) + OFFSET;
    let c = std::f32::consts::FRAC_1_SQRT_2;
    ORIGIN + Vec3::new(scale_x * c * (p.x + p.z), p.y, c * (p.z - p.x))
}

fn sample(manager: &DefaultNavigationManager, point: Vec3) -> Option<NavSampleHit> {
    NavigationManager::sample_position(
        manager,
        NavSampleQuery {
            nav_mesh: None,
            position: point.to_array(),
            extents: [0.2, 0.75, 0.2],
            agent_type: DEFAULT_AGENT_TYPE.to_owned(),
            area_mask: DEFAULT_AREA_MASK,
        },
    )
    .unwrap()
}

fn raycast(manager: &DefaultNavigationManager, start: Vec3, end: Vec3) -> NavRaycastResult {
    NavigationManager::raycast(
        manager,
        NavRaycastQuery {
            nav_mesh: None,
            start: start.to_array(),
            end: end.to_array(),
            agent_type: DEFAULT_AGENT_TYPE.to_owned(),
            area_mask: DEFAULT_AREA_MASK,
        },
    )
    .unwrap()
}

fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

fn require_baked_world_footprint(source: NavMeshUseGeometry, scale_x: f32) {
    let manager = test_support::navigation_manager();
    assert_eq!(NavigationManager::stats(&*manager).loaded_nav_meshes, 0);
    let mut world = World::empty();
    let surface = spawn(&mut world, Transform::default());
    world
        .register_component_type(
            navigation_component_descriptors()
                .into_iter()
                .find(|descriptor| descriptor.type_id == NAV_MESH_SURFACE_COMPONENT_TYPE)
                .unwrap(),
        )
        .unwrap();
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            serde_json::to_value(NavMeshSurfaceDescriptor {
                use_geometry: source,
                generate_links: false,
                ..NavMeshSurfaceDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    let parent = spawn(
        &mut world,
        Transform {
            translation: ORIGIN,
            scale: Vec3::new(scale_x, 1.0, 1.0),
            ..Transform::default()
        },
    );
    let floor = spawn(
        &mut world,
        Transform {
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
            ..Transform::default()
        },
    );
    assert!(world.set_parent_checked(floor, Some(parent)).unwrap());
    let matrix = world.world_matrix(floor).unwrap();
    assert!(matrix.determinant() > 0.0);
    let xz_dot = matrix.x_axis.dot(matrix.z_axis);
    if scale_x > 1.0 {
        assert!(
            (xz_dot - 1.5).abs() < 1.0e-5,
            "the hierarchy must really shear the quad"
        );
    } else {
        assert!(xz_dot.abs() < 1.0e-5);
    }
    match source {
        NavMeshUseGeometry::PhysicsColliders => {
            world
                .set_collider(
                    floor,
                    Some(ColliderComponent {
                        shape: ColliderShape::Box { half_extents: HALF },
                        local_transform: Transform::from_translation(OFFSET),
                        ..ColliderComponent::default()
                    }),
                )
                .unwrap();
            assert!(world.get::<MeshRenderer>(floor).is_none());
        }
        NavMeshUseGeometry::RenderMeshes => attach_render_floor(&manager, &mut world, floor),
    }
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
        (1, 4, 2),
        "{report:?}"
    );
    assert!(report.baked_vertices > 0 && report.baked_polygons > 0 && report.tiles > 0);
    assert!(!report.asset.as_ref().unwrap().is_empty());
    assert!(
        report
            .diagnostics
            .iter()
            .all(|row| !row.message.contains("fallback")),
        "{report:?}"
    );
    assert_eq!(NavigationManager::stats(&*manager).loaded_nav_meshes, 1);

    let center = point_on_top(0.0, 0.0, scale_x);
    let center_sample =
        sample(&manager, center).expect("the actual baked floor is walkable at its center");
    let center_on_mesh = Vec3::from_array(center_sample.position);
    assert_eq!(center_sample.area, AREA_WALKABLE);
    assert!(horizontal_distance(center, center_on_mesh) < 0.2 && center_sample.distance < 0.35);
    let outside = point_on_top(0.0, 100.0, scale_x);
    assert!(
        sample(&manager, outside).is_none(),
        "a point well outside the global footprint must miss"
    );
    let outside_ray = raycast(&manager, center_on_mesh, outside);
    assert!(
        outside_ray.hit && outside_ray.distance > 1.0,
        "the outside ray must exit a real floor: {outside_ray:?}"
    );

    // At least three local units from the nearest correct edge. With inherited
    // shear this lies outside the old orthogonalized floor, away from .2-cell
    // rasterization noise. A nearest poly candidate can still snap to its edge.
    let probe = point_on_top(20.0, 5.0, scale_x);
    let probe_sample = sample(&manager, probe);
    let probe_xz_gap = probe_sample
        .as_ref()
        .map(|hit| horizontal_distance(probe, Vec3::from_array(hit.position)));
    let probe_ray = raycast(&manager, center_on_mesh, probe);
    eprintln!("NAV_BAKE_AFFINE source={source:?} scale_x={scale_x} source_vertices={} source_triangles={} baked_polygons={} probe={probe:?} sample_gap={probe_xz_gap:?} ray_hit={} ray_distance={} outside_hit={}",
        report.source_vertices, report.source_triangles, report.baked_polygons, probe_ray.hit, probe_ray.distance, outside_ray.hit);
    assert!(
        probe_xz_gap.is_some_and(|gap| gap < 0.2) && !probe_ray.hit,
        "full affine floor must reach the same world point without an early boundary: target={probe:?}, sample={probe_sample:?}, ray={probe_ray:?}"
    );
    assert!(probe_sample.unwrap().distance < 0.35);
    assert!(horizontal_distance(probe, Vec3::from_array(probe_ray.position)) < 0.2);
}

#[test]
fn collider_box_rigid_parent_keeps_the_walkable_control() {
    require_baked_world_footprint(NavMeshUseGeometry::PhysicsColliders, 1.0);
}

#[test]
fn collider_box_shear_keeps_the_actual_world_walkable_footprint() {
    require_baked_world_footprint(NavMeshUseGeometry::PhysicsColliders, 2.0);
}

#[test]
fn resident_render_mesh_rigid_parent_keeps_the_walkable_control() {
    require_baked_world_footprint(NavMeshUseGeometry::RenderMeshes, 1.0);
}

#[test]
fn resident_render_mesh_shear_keeps_the_actual_world_walkable_footprint() {
    require_baked_world_footprint(NavMeshUseGeometry::RenderMeshes, 2.0);
}
