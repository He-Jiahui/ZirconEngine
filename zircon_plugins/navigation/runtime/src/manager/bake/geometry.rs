use crate::runtime_obstacles::{collect_runtime_obstacles, node_intersects_obstacle};
use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::navigation::{
    NavMeshModifierMode, NavMeshSurfaceDescriptor, NavMeshUseGeometry, NavigationError,
    NavigationErrorKind,
};
use zircon_runtime::core::math::{
    is_finite_mat4, is_finite_quat, is_finite_vec3, Mat4, Real, Transform, Vec3,
};
use zircon_runtime::scene::components::{ColliderShape, SceneNode};
use zircon_runtime::scene::World;

use super::area_volume::{collect_area_volumes, volume_area_override, BakeAreaVolume};
use super::filter::{node_matches_surface_collection, should_exclude_from_bake};
use super::geometry_provider::collect_render_node_geometry;
use super::modifier::{direct_modifier, effective_modifier};
use super::source_selection::should_fallback_to_render_mesh;

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct BakeGeometry {
    pub(super) vertices: Vec<[Real; 3]>,
    pub(super) indices: Vec<u32>,
    pub(super) triangle_areas: Vec<u8>,
    pub(super) source_entities: usize,
    pub(super) skipped_navigation_components: usize,
    pub(super) removed_by_modifier: usize,
    pub(super) modified_by_area_override: usize,
    pub(super) carved_by_obstacle: usize,
    pub(super) render_mesh_fallback: bool,
    pub(super) unbound_render_nodes: usize,
    pub(super) render_lod_levels_not_selected: usize,
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

// 先按表面收集规则筛选世界节点，再应用障碍雕刻与区域修饰，避免把导航控制实体烘进网格。
pub(super) fn collect_bake_geometry(
    asset_manager: &ProjectAssetManager,
    world: &World,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
    agent_type: &str,
) -> Result<BakeGeometry, NavigationError> {
    let area_volumes = collect_area_volumes(world, agent_type);
    let mut geometry = collect_bake_geometry_for_source(
        asset_manager,
        world,
        surface_entity,
        surface,
        agent_type,
        surface.use_geometry,
        &area_volumes,
    )?;
    if should_fallback_to_render_mesh(surface.use_geometry, &geometry) {
        geometry = collect_bake_geometry_for_source(
            asset_manager,
            world,
            surface_entity,
            surface,
            agent_type,
            NavMeshUseGeometry::RenderMeshes,
            &area_volumes,
        )?;
        geometry.render_mesh_fallback = true;
    }
    Ok(geometry)
}

fn collect_bake_geometry_for_source(
    asset_manager: &ProjectAssetManager,
    world: &World,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
    agent_type: &str,
    geometry_source: NavMeshUseGeometry,
    area_volumes: &[BakeAreaVolume],
) -> Result<BakeGeometry, NavigationError> {
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
            .or_else(|| volume_area_override(world, &node, area_volumes))
            .or(surface_area_override);
        let area = area_override.unwrap_or(surface.default_area);

        let before = geometry.source_triangles();
        match geometry_source {
            NavMeshUseGeometry::RenderMeshes => {
                let Some(transform) = world.world_transform(node.id) else {
                    if node.mesh.is_some() {
                        return Err(NavigationError::new(
                            zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration,
                            format!("renderable entity {} has no world transform", node.id),
                        ));
                    }
                    continue;
                };
                collect_render_node_geometry(
                    asset_manager,
                    &node,
                    transform.matrix(),
                    &mut geometry,
                    area,
                )?;
            }
            NavMeshUseGeometry::PhysicsColliders => {
                collect_collider_geometry(world, &node, &mut geometry, area)?;
            }
        }
        if geometry.source_triangles() > before {
            geometry.source_entities += 1;
            if area_override.is_some() {
                geometry.modified_by_area_override += 1;
            }
        }
    }
    Ok(geometry)
}

fn collect_collider_geometry(
    world: &World,
    node: &SceneNode,
    geometry: &mut BakeGeometry,
    area: u8,
) -> Result<(), NavigationError> {
    let Some(collider) = node.collider.as_ref() else {
        return Ok(());
    };
    if collider.sensor {
        return Ok(());
    }
    let Some(transform) = world.world_transform(node.id) else {
        return Ok(());
    };
    let matrix = transform.matrix() * collider.local_transform.matrix();
    collect_collider_shape_geometry(matrix, &collider.shape, geometry, area)
}

/// Generate one complete collider shape into scratch storage before appending it to the bake.
/// This keeps a malformed compound or overflowed transform from leaving partial geometry behind.
fn collect_collider_shape_geometry(
    matrix: Mat4,
    shape: &ColliderShape,
    geometry: &mut BakeGeometry,
    area: u8,
) -> Result<(), NavigationError> {
    validate_collider_matrix(matrix)?;
    let mut generated = BakeGeometry::default();
    append_collider_shape_geometry(matrix, shape, &mut generated, area)?;
    validate_generated_collider_geometry(&generated)?;
    append_generated_collider_geometry(geometry, generated)
}

fn append_collider_shape_geometry(
    matrix: Mat4,
    shape: &ColliderShape,
    geometry: &mut BakeGeometry,
    area: u8,
) -> Result<(), NavigationError> {
    validate_collider_matrix(matrix)?;
    match shape {
        ColliderShape::Box { half_extents } => {
            validate_collider_vec3(*half_extents, "box half extents")?;
            geometry.push_quad_from_matrix(matrix, *half_extents, area);
        }
        ColliderShape::Sphere { radius } => {
            validate_collider_scalar(*radius, "sphere radius")?;
            geometry.push_disc_from_matrix(matrix, *radius, 0.0, area);
        }
        ColliderShape::Capsule {
            radius,
            half_height,
        } => {
            validate_collider_scalar(*radius, "capsule radius")?;
            validate_collider_scalar(*half_height, "capsule half height")?;
            geometry.push_disc_from_matrix(matrix, *radius, *half_height, area);
        }
        ColliderShape::Cylinder {
            radius,
            half_height,
        } => {
            validate_collider_scalar(*radius, "cylinder radius")?;
            validate_collider_scalar(*half_height, "cylinder half height")?;
            geometry.push_disc_from_matrix(matrix, *radius, *half_height, area);
        }
        ColliderShape::ConvexHull { points } => {
            for point in points {
                validate_collider_vec3(*point, "convex hull point")?;
            }
            let Some(first) = points.first().copied() else {
                return Ok(());
            };
            let (min, max) = points
                .iter()
                .copied()
                .fold((first, first), |(min, max), point| {
                    (min.min(point), max.max(point))
                });
            let center = (min + max) * 0.5;
            let half_extents = (max - min) * 0.5;
            geometry.push_quad_from_matrix(
                matrix * Mat4::from_translation(center),
                half_extents,
                area,
            );
        }
        ColliderShape::Compound { children } => {
            for (transform, child) in children {
                validate_collider_transform(*transform)?;
                append_collider_shape_geometry(matrix * transform.matrix(), child, geometry, area)?;
            }
        }
        ColliderShape::TriangleMesh { .. } | ColliderShape::HeightField { .. } => {
            // Asset-backed collider geometry is collected by its owning asset bake path.
        }
    }
    Ok(())
}

fn validate_collider_transform(transform: Transform) -> Result<(), NavigationError> {
    if !is_finite_vec3(transform.translation)
        || !is_finite_quat(transform.rotation)
        || !is_finite_vec3(transform.scale)
    {
        return Err(collider_geometry_error(
            "collider child transform contains a non-finite value",
        ));
    }
    Ok(())
}

fn validate_collider_matrix(matrix: Mat4) -> Result<(), NavigationError> {
    if is_finite_mat4(matrix) {
        return Ok(());
    }
    Err(collider_geometry_error(
        "collider transform produced a non-finite matrix",
    ))
}

fn validate_collider_vec3(value: Vec3, label: &str) -> Result<(), NavigationError> {
    if is_finite_vec3(value) {
        return Ok(());
    }
    Err(collider_geometry_error(format!(
        "collider {label} contains a non-finite value"
    )))
}

fn validate_collider_scalar(value: Real, label: &str) -> Result<(), NavigationError> {
    if value.is_finite() {
        return Ok(());
    }
    Err(collider_geometry_error(format!(
        "collider {label} contains a non-finite value"
    )))
}

fn validate_generated_collider_geometry(geometry: &BakeGeometry) -> Result<(), NavigationError> {
    if geometry.vertices.len() > u32::MAX as usize {
        return Err(collider_geometry_error(
            "generated collider geometry exceeds the Recast vertex index range",
        ));
    }
    if geometry
        .vertices
        .iter()
        .any(|vertex| !vertex.iter().all(|component| component.is_finite()))
    {
        return Err(collider_geometry_error(
            "generated collider geometry contains a non-finite transformed vertex",
        ));
    }
    if geometry.indices.len() % 3 != 0 {
        return Err(collider_geometry_error(
            "generated collider geometry index count is not divisible by three",
        ));
    }
    if geometry.triangle_areas.len() != geometry.indices.len() / 3 {
        return Err(collider_geometry_error(
            "generated collider geometry has mismatched triangle area metadata",
        ));
    }
    if geometry
        .indices
        .iter()
        .any(|index| *index as usize >= geometry.vertices.len())
    {
        return Err(collider_geometry_error(
            "generated collider geometry contains an index outside its vertex range",
        ));
    }
    Ok(())
}

fn append_generated_collider_geometry(
    geometry: &mut BakeGeometry,
    generated: BakeGeometry,
) -> Result<(), NavigationError> {
    validate_generated_collider_geometry(&generated)?;
    let base = u32::try_from(geometry.vertices.len()).map_err(|_| {
        collider_geometry_error("collider geometry exceeds the Recast vertex index range")
    })?;
    let mut indices = Vec::with_capacity(generated.indices.len());
    for index in generated.indices {
        indices.push(base.checked_add(index).ok_or_else(|| {
            collider_geometry_error("collider geometry exceeds the Recast vertex index range")
        })?);
    }
    geometry.vertices.extend(generated.vertices);
    geometry.indices.extend(indices);
    geometry.triangle_areas.extend(generated.triangle_areas);
    Ok(())
}

fn collider_geometry_error(message: impl Into<String>) -> NavigationError {
    NavigationError::new(NavigationErrorKind::InvalidConfiguration, message)
}

#[cfg(test)]
#[path = "tests/geometry_collider_tests.rs"]
mod geometry_collider_tests;
