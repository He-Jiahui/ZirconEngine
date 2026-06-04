use crate::runtime_obstacles::{collect_runtime_obstacles, node_intersects_obstacle};
use zircon_runtime::core::framework::navigation::{
    NavMeshModifierMode, NavMeshSurfaceDescriptor, NavMeshUseGeometry,
};
use zircon_runtime::core::math::{Mat4, Real, Vec3};
use zircon_runtime::scene::components::{ColliderShape, NodeKind, SceneNode};
use zircon_runtime::scene::World;

use super::filter::{node_matches_surface_collection, should_exclude_from_bake};
use super::modifier::{direct_modifier, effective_modifier};

#[derive(Clone, Debug, Default)]
pub(super) struct BakeGeometry {
    pub(super) vertices: Vec<[Real; 3]>,
    pub(super) indices: Vec<u32>,
    pub(super) triangle_areas: Vec<u8>,
    pub(super) source_entities: usize,
    pub(super) skipped_navigation_components: usize,
    pub(super) removed_by_modifier: usize,
    pub(super) modified_by_area_override: usize,
    pub(super) carved_by_obstacle: usize,
}

impl BakeGeometry {
    pub(super) fn source_triangles(&self) -> usize {
        self.indices.len() / 3
    }

    fn push_quad_from_matrix(&mut self, matrix: Mat4, half_extents: Vec3, area: u8) {
        let top_y = half_extents.y.max(0.0);
        let corners = [
            Vec3::new(-half_extents.x, top_y, -half_extents.z),
            Vec3::new(half_extents.x, top_y, -half_extents.z),
            Vec3::new(half_extents.x, top_y, half_extents.z),
            Vec3::new(-half_extents.x, top_y, half_extents.z),
        ];
        let base = self.vertices.len() as u32;
        self.vertices.extend(
            corners
                .into_iter()
                .map(|corner| matrix.transform_point3(corner).to_array()),
        );
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        self.triangle_areas.extend_from_slice(&[area, area]);
    }

    fn push_disc_from_matrix(&mut self, matrix: Mat4, radius: Real, local_y: Real, area: u8) {
        const SEGMENTS: u32 = 12;
        let radius = radius.max(0.05);
        let center = self.vertices.len() as u32;
        self.vertices.push(
            matrix
                .transform_point3(Vec3::new(0.0, local_y, 0.0))
                .to_array(),
        );
        for index in 0..SEGMENTS {
            let angle = (index as Real / SEGMENTS as Real) * std::f32::consts::TAU;
            let point = Vec3::new(angle.cos() * radius, local_y, angle.sin() * radius);
            self.vertices
                .push(matrix.transform_point3(point).to_array());
        }
        for index in 0..SEGMENTS {
            let next = if index + 1 == SEGMENTS {
                center + 1
            } else {
                center + index + 2
            };
            self.indices
                .extend_from_slice(&[center, center + index + 1, next]);
            self.triangle_areas.push(area);
        }
    }
}

pub(super) fn collect_bake_geometry(
    world: &World,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
    agent_type: &str,
) -> BakeGeometry {
    let mut geometry = BakeGeometry::default();
    let carved_obstacles = collect_runtime_obstacles(world)
        .into_iter()
        .filter(|obstacle| obstacle.carve)
        .collect::<Vec<_>>();
    let surface_area_override = surface_entity
        .and_then(|entity| direct_modifier(world, entity, agent_type))
        .filter(|modifier| modifier.override_area)
        .map(|modifier| modifier.area);
    for node in world.node_records() {
        if should_exclude_from_bake(world, node.id) {
            geometry.skipped_navigation_components += 1;
            continue;
        }
        if !node_matches_surface_collection(world, &node, surface_entity, surface) {
            continue;
        }

        let modifier = effective_modifier(world, node.id, agent_type);
        if matches!(
            modifier.as_ref().map(|modifier| modifier.mode),
            Some(NavMeshModifierMode::Remove)
        ) {
            geometry.removed_by_modifier += 1;
            continue;
        }
        if node_intersects_obstacle(world, &node, &carved_obstacles) {
            geometry.carved_by_obstacle += 1;
            continue;
        }
        let area_override = modifier
            .as_ref()
            .filter(|modifier| modifier.override_area)
            .map(|modifier| modifier.area)
            .or(surface_area_override);
        let area = area_override.unwrap_or(surface.default_area);
        if area_override.is_some() {
            geometry.modified_by_area_override += 1;
        }

        let before = geometry.source_triangles();
        match surface.use_geometry {
            NavMeshUseGeometry::RenderMeshes => {
                collect_render_node_geometry(world, &node, &mut geometry, area)
            }
            NavMeshUseGeometry::PhysicsColliders => {
                collect_collider_geometry(world, &node, &mut geometry, area)
            }
        }
        if geometry.source_triangles() > before {
            geometry.source_entities += 1;
        }
    }
    geometry
}

fn collect_render_node_geometry(
    world: &World,
    node: &SceneNode,
    geometry: &mut BakeGeometry,
    area: u8,
) {
    if node.mesh.is_none() && !matches!(node.kind, NodeKind::Cube | NodeKind::Mesh) {
        return;
    }
    let Some(transform) = world.world_transform(node.id) else {
        return;
    };
    geometry.push_quad_from_matrix(transform.matrix(), Vec3::splat(0.5), area);
}

fn collect_collider_geometry(
    world: &World,
    node: &SceneNode,
    geometry: &mut BakeGeometry,
    area: u8,
) {
    let Some(collider) = node.collider.as_ref() else {
        return;
    };
    if collider.sensor {
        return;
    }
    let Some(transform) = world.world_transform(node.id) else {
        return;
    };
    let matrix = transform.matrix() * collider.local_transform.matrix();
    match &collider.shape {
        ColliderShape::Box { half_extents } => {
            geometry.push_quad_from_matrix(matrix, *half_extents, area);
        }
        ColliderShape::Sphere { radius } => {
            geometry.push_disc_from_matrix(matrix, *radius, 0.0, area);
        }
        ColliderShape::Capsule {
            radius,
            half_height,
        } => {
            geometry.push_disc_from_matrix(matrix, *radius, *half_height, area);
        }
    }
}
