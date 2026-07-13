use std::collections::HashMap;
use std::ffi::CStr;
use std::slice;
use std::sync::Arc;

use zircon_runtime::core::framework::navigation::{
    NavAreaId, NavigationError, NavigationErrorKind,
};
use zircon_runtime::core::framework::navigation::{
    NavMeshAsset, NavMeshPolygonAsset, NavMeshTileAsset,
};
use zircon_runtime::core::math::Real;

use crate::ffi::{
    self, ZrNavRecastBakeResult, ZrNavRecastBakeSettings, ZrNavRecastBakeTileRequest,
};
use crate::RecastBackend;

#[derive(Clone, Debug, PartialEq)]
pub struct RecastBakeInput {
    pub agent_type: String,
    pub source_vertices: usize,
    pub source_triangles: usize,
    pub half_extent: Real,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecastBakeMeshInput {
    pub agent_type: String,
    pub vertices: Vec<[Real; 3]>,
    pub indices: Vec<u32>,
    pub triangle_areas: Vec<NavAreaId>,
    pub default_area: NavAreaId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecastTiledBakeInput {
    pub mesh: RecastBakeMeshInput,
    pub tile_size: Real,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecastTileSpec {
    pub id: u32,
    pub x: i32,
    pub z: i32,
    pub bounds_min: [Real; 3],
    pub bounds_max: [Real; 3],
}

#[derive(Clone, Debug)]
pub struct RecastTiledBakePlan {
    mesh: Arc<RecastBakeMeshInput>,
    flat_vertices: Arc<[Real]>,
    triangle_areas: Arc<[NavAreaId]>,
    tiles: Arc<[RecastTileSpec]>,
    tile_size: Real,
}

impl RecastTiledBakePlan {
    pub fn tiles(&self) -> &[RecastTileSpec] {
        &self.tiles
    }

    pub fn tile_size(&self) -> Real {
        self.tile_size
    }

    pub fn with_tiles(&self, tiles: Vec<RecastTileSpec>) -> Self {
        Self {
            mesh: Arc::clone(&self.mesh),
            flat_vertices: Arc::clone(&self.flat_vertices),
            triangle_areas: Arc::clone(&self.triangle_areas),
            tiles: tiles.into(),
            tile_size: self.tile_size,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RecastBakeSettings {
    cell_size: Real,
    cell_height: Real,
    walkable_slope_degrees: Real,
    walkable_height: Real,
    walkable_climb: Real,
    walkable_radius: Real,
    min_region_area: Real,
    merge_region_area: Real,
    max_edge_length: Real,
    max_simplification_error: Real,
    max_vertices_per_polygon: u32,
}

impl Default for RecastBakeSettings {
    fn default() -> Self {
        Self {
            cell_size: 0.2,
            cell_height: 0.1,
            walkable_slope_degrees: 45.0,
            walkable_height: 2.0,
            walkable_climb: 0.4,
            walkable_radius: 0.0,
            min_region_area: 0.0,
            merge_region_area: 0.0,
            max_edge_length: 12.0,
            max_simplification_error: 1.3,
            max_vertices_per_polygon: 6,
        }
    }
}

impl RecastBakeSettings {
    fn to_ffi(&self) -> ZrNavRecastBakeSettings {
        ZrNavRecastBakeSettings {
            cell_size: self.cell_size,
            cell_height: self.cell_height,
            walkable_slope_degrees: self.walkable_slope_degrees,
            walkable_height: self.walkable_height,
            walkable_climb: self.walkable_climb,
            walkable_radius: self.walkable_radius,
            min_region_area: self.min_region_area,
            merge_region_area: self.merge_region_area,
            max_edge_length: self.max_edge_length,
            max_simplification_error: self.max_simplification_error,
            max_vertices_per_polygon: self.max_vertices_per_polygon,
        }
    }
}

impl RecastBackend {
    pub fn bake_simple_surface(
        &self,
        input: RecastBakeInput,
    ) -> Result<NavMeshAsset, NavigationError> {
        if input.half_extent <= 0.0 || !input.half_extent.is_finite() {
            return Err(NavigationError::new(
                NavigationErrorKind::InvalidConfiguration,
                "navigation bake half_extent must be positive and finite",
            ));
        }
        Ok(NavMeshAsset::simple_quad(
            input.agent_type,
            input.half_extent,
        ))
    }

    pub fn bake_triangle_mesh(
        &self,
        input: RecastBakeMeshInput,
    ) -> Result<NavMeshAsset, NavigationError> {
        if input.vertices.is_empty() || input.indices.len() < 3 {
            return Err(NavigationError::new(
                NavigationErrorKind::BakeFailed,
                "navigation bake source mesh has no triangles",
            ));
        }
        if input.indices.len() % 3 != 0 {
            return Err(NavigationError::new(
                NavigationErrorKind::BakeFailed,
                "navigation bake source mesh index count is not divisible by three",
            ));
        }
        if input
            .indices
            .iter()
            .any(|index| (*index as usize) >= input.vertices.len())
        {
            return Err(NavigationError::new(
                NavigationErrorKind::BakeFailed,
                "navigation bake source mesh references a missing vertex",
            ));
        }
        if input
            .vertices
            .iter()
            .flat_map(|vertex| vertex.iter())
            .any(|coordinate| !coordinate.is_finite())
        {
            return Err(NavigationError::new(
                NavigationErrorKind::BakeFailed,
                "navigation bake source mesh contains non-finite vertex coordinates",
            ));
        }

        let mut flat_vertices = Vec::with_capacity(input.vertices.len() * 3);
        for vertex in &input.vertices {
            flat_vertices.extend_from_slice(vertex);
        }
        let triangle_count = input.indices.len() / 3;
        let mut triangle_areas = vec![input.default_area; triangle_count];
        for (index, area) in input.triangle_areas.into_iter().enumerate() {
            if let Some(target) = triangle_areas.get_mut(index) {
                *target = area;
            }
        }

        let ffi_settings = RecastBakeSettings::default().to_ffi();
        let mut ffi_result = ZrNavRecastBakeResult::default();
        unsafe {
            ffi::zr_nav_recast_bake_triangle_mesh(
                flat_vertices.as_ptr(),
                input.vertices.len() as u32,
                input.indices.as_ptr(),
                input.indices.len() as u32,
                triangle_areas.as_ptr(),
                triangle_areas.len() as u32,
                &ffi_settings,
                &mut ffi_result,
            );
        }

        let result = native_bake_result_to_asset(input.agent_type, &mut ffi_result);
        unsafe {
            ffi::zr_nav_recast_free_bake_result(&mut ffi_result);
        }
        result
    }

    pub fn prepare_tiled_bake(
        &self,
        input: RecastTiledBakeInput,
    ) -> Result<RecastTiledBakePlan, NavigationError> {
        validate_mesh_input(&input.mesh)?;
        if !input.tile_size.is_finite() || input.tile_size <= 0.0 {
            return Err(NavigationError::new(
                NavigationErrorKind::InvalidConfiguration,
                "navigation tile size must be positive and finite",
            ));
        }
        let (source_min, source_max) = mesh_bounds(&input.mesh.vertices);
        let origin_x = (source_min[0] / input.tile_size).floor() * input.tile_size;
        let origin_z = (source_min[2] / input.tile_size).floor() * input.tile_size;
        let width = ((source_max[0] - origin_x) / input.tile_size)
            .ceil()
            .max(1.0) as i32;
        let depth = ((source_max[2] - origin_z) / input.tile_size)
            .ceil()
            .max(1.0) as i32;
        let mut tiles = Vec::new();
        for z in 0..depth {
            for x in 0..width {
                let bounds_min = [
                    origin_x + x as Real * input.tile_size,
                    source_min[1],
                    origin_z + z as Real * input.tile_size,
                ];
                let bounds_max = [
                    bounds_min[0] + input.tile_size,
                    source_max[1],
                    bounds_min[2] + input.tile_size,
                ];
                if tile_intersects_mesh(&input.mesh, bounds_min, bounds_max) {
                    tiles.push(RecastTileSpec {
                        id: (z * width + x) as u32,
                        x,
                        z,
                        bounds_min,
                        bounds_max,
                    });
                }
            }
        }
        let (flat_vertices, triangle_areas) = native_mesh_buffers(&input.mesh);
        Ok(RecastTiledBakePlan {
            mesh: Arc::new(input.mesh),
            flat_vertices: flat_vertices.into(),
            triangle_areas: triangle_areas.into(),
            tiles: tiles.into(),
            tile_size: input.tile_size,
        })
    }

    pub fn bake_planned_tile(
        &self,
        plan: &RecastTiledBakePlan,
        tile: RecastTileSpec,
    ) -> Result<NavMeshAsset, NavigationError> {
        let input = plan.mesh.as_ref();
        let ffi_settings = RecastBakeSettings::default().to_ffi();
        let ffi_tile = ZrNavRecastBakeTileRequest {
            id: tile.id,
            bounds_min: tile.bounds_min,
            bounds_max: tile.bounds_max,
        };
        let mut ffi_result = ZrNavRecastBakeResult::default();
        unsafe {
            ffi::zr_nav_recast_bake_tile(
                plan.flat_vertices.as_ptr(),
                input.vertices.len() as u32,
                input.indices.as_ptr(),
                input.indices.len() as u32,
                plan.triangle_areas.as_ptr(),
                plan.triangle_areas.len() as u32,
                &ffi_settings,
                &ffi_tile,
                &mut ffi_result,
            );
        }
        let result = native_bake_result_to_asset(input.agent_type.clone(), &mut ffi_result);
        unsafe {
            ffi::zr_nav_recast_free_bake_result(&mut ffi_result);
        }
        result
    }

    pub fn bake_tiled_mesh(
        &self,
        input: RecastTiledBakeInput,
    ) -> Result<NavMeshAsset, NavigationError> {
        let plan = self.prepare_tiled_bake(input)?;
        let mut tiles = Vec::with_capacity(plan.tiles.len());
        for tile in plan.tiles.iter().copied() {
            tiles.push(self.bake_planned_tile(&plan, tile)?);
        }
        merge_tiled_assets(plan.mesh.agent_type.clone(), tiles)
    }
}

pub fn merge_tiled_assets(
    agent_type: String,
    mut tile_assets: Vec<NavMeshAsset>,
) -> Result<NavMeshAsset, NavigationError> {
    tile_assets.sort_by_key(|asset| asset.tiles.first().map_or(u32::MAX, |tile| tile.id));
    let mut merged = NavMeshAsset::empty(agent_type);
    if let Some(first) = tile_assets.first() {
        merged.area_costs = first.area_costs.clone();
    }
    let mut vertex_map = HashMap::<[i64; 3], u32>::new();
    for asset in tile_assets {
        merged.tiles.extend(asset.tiles.iter().cloned());
        for polygon in asset.polygons {
            let start = polygon.first_index as usize;
            let end = start.saturating_add(polygon.index_count as usize);
            if end > asset.indices.len() {
                return Err(NavigationError::new(
                    NavigationErrorKind::BackendFailure,
                    "tiled Recast bake returned an invalid polygon index range",
                ));
            }
            let first_index = merged.indices.len() as u32;
            for source_index in &asset.indices[start..end] {
                let vertex = *asset.vertices.get(*source_index as usize).ok_or_else(|| {
                    NavigationError::new(
                        NavigationErrorKind::BackendFailure,
                        "tiled Recast bake returned a missing polygon vertex",
                    )
                })?;
                let key = vertex.map(|coordinate| (coordinate as f64 * 10_000.0).round() as i64);
                let target = *vertex_map.entry(key).or_insert_with(|| {
                    let index = merged.vertices.len() as u32;
                    merged.vertices.push(vertex);
                    index
                });
                merged.indices.push(target);
            }
            merged.polygons.push(NavMeshPolygonAsset {
                first_index,
                index_count: polygon.index_count,
                area: polygon.area,
                tile: polygon.tile,
            });
        }
    }
    Ok(merged)
}

fn validate_mesh_input(input: &RecastBakeMeshInput) -> Result<(), NavigationError> {
    if input.vertices.is_empty() || input.indices.len() < 3 {
        return Err(NavigationError::new(
            NavigationErrorKind::BakeFailed,
            "navigation bake source mesh has no triangles",
        ));
    }
    if input.indices.len() % 3 != 0 {
        return Err(NavigationError::new(
            NavigationErrorKind::BakeFailed,
            "navigation bake source mesh index count is not divisible by three",
        ));
    }
    if input
        .indices
        .iter()
        .any(|index| (*index as usize) >= input.vertices.len())
    {
        return Err(NavigationError::new(
            NavigationErrorKind::BakeFailed,
            "navigation bake source mesh references a missing vertex",
        ));
    }
    if input
        .vertices
        .iter()
        .flat_map(|vertex| vertex.iter())
        .any(|coordinate| !coordinate.is_finite())
    {
        return Err(NavigationError::new(
            NavigationErrorKind::BakeFailed,
            "navigation bake source mesh contains non-finite vertex coordinates",
        ));
    }
    Ok(())
}

fn native_mesh_buffers(input: &RecastBakeMeshInput) -> (Vec<Real>, Vec<NavAreaId>) {
    let mut flat_vertices = Vec::with_capacity(input.vertices.len() * 3);
    for vertex in &input.vertices {
        flat_vertices.extend_from_slice(vertex);
    }
    let triangle_count = input.indices.len() / 3;
    let mut triangle_areas = vec![input.default_area; triangle_count];
    for (index, area) in input.triangle_areas.iter().copied().enumerate() {
        if let Some(target) = triangle_areas.get_mut(index) {
            *target = area;
        }
    }
    (flat_vertices, triangle_areas)
}

fn mesh_bounds(vertices: &[[Real; 3]]) -> ([Real; 3], [Real; 3]) {
    let mut min = vertices[0];
    let mut max = vertices[0];
    for vertex in vertices.iter().skip(1) {
        for axis in 0..3 {
            min[axis] = min[axis].min(vertex[axis]);
            max[axis] = max[axis].max(vertex[axis]);
        }
    }
    (min, max)
}

fn tile_intersects_mesh(
    input: &RecastBakeMeshInput,
    bounds_min: [Real; 3],
    bounds_max: [Real; 3],
) -> bool {
    input.indices.chunks_exact(3).any(|triangle| {
        let vertices = [
            input.vertices[triangle[0] as usize],
            input.vertices[triangle[1] as usize],
            input.vertices[triangle[2] as usize],
        ];
        let triangle_min_x = vertices
            .iter()
            .map(|vertex| vertex[0])
            .fold(Real::INFINITY, Real::min);
        let triangle_max_x = vertices
            .iter()
            .map(|vertex| vertex[0])
            .fold(Real::NEG_INFINITY, Real::max);
        let triangle_min_z = vertices
            .iter()
            .map(|vertex| vertex[2])
            .fold(Real::INFINITY, Real::min);
        let triangle_max_z = vertices
            .iter()
            .map(|vertex| vertex[2])
            .fold(Real::NEG_INFINITY, Real::max);
        triangle_max_x >= bounds_min[0]
            && triangle_min_x <= bounds_max[0]
            && triangle_max_z >= bounds_min[2]
            && triangle_min_z <= bounds_max[2]
    })
}

fn native_bake_result_to_asset(
    agent_type: String,
    result: &mut ZrNavRecastBakeResult,
) -> Result<NavMeshAsset, NavigationError> {
    if result.status != 1 {
        return Err(NavigationError::new(
            NavigationErrorKind::BakeFailed,
            native_bake_message(result),
        ));
    }
    if (result.vertex_count > 0 && result.vertices.is_null())
        || (result.index_count > 0 && result.indices.is_null())
        || (result.polygon_count > 0 && result.polygons.is_null())
        || (result.tile_count > 0 && result.tiles.is_null())
    {
        return Err(NavigationError::new(
            NavigationErrorKind::BackendFailure,
            "native Recast bake returned incomplete output buffers",
        ));
    }

    let vertex_values = if result.vertex_count == 0 {
        &[][..]
    } else {
        unsafe { slice::from_raw_parts(result.vertices, result.vertex_count as usize * 3) }
    };
    let vertices = vertex_values
        .chunks_exact(3)
        .map(|vertex| [vertex[0], vertex[1], vertex[2]])
        .collect::<Vec<_>>();
    let indices = if result.index_count == 0 {
        Vec::new()
    } else {
        unsafe { slice::from_raw_parts(result.indices, result.index_count as usize) }.to_vec()
    };
    let polygon_values = if result.polygon_count == 0 {
        &[][..]
    } else {
        unsafe { slice::from_raw_parts(result.polygons, result.polygon_count as usize) }
    };
    let polygons = polygon_values
        .iter()
        .map(|polygon| NavMeshPolygonAsset {
            first_index: polygon.first_index,
            index_count: polygon.index_count,
            area: polygon.area,
            tile: polygon.tile,
        })
        .collect::<Vec<_>>();
    let tile_values = if result.tile_count == 0 {
        &[][..]
    } else {
        unsafe { slice::from_raw_parts(result.tiles, result.tile_count as usize) }
    };
    let tiles = tile_values
        .iter()
        .map(|tile| NavMeshTileAsset {
            id: tile.id,
            bounds_min: tile.bounds_min,
            bounds_max: tile.bounds_max,
            polygon_count: tile.polygon_count,
        })
        .collect::<Vec<_>>();

    if (vertices.is_empty() || indices.is_empty() || polygons.is_empty()) && tiles.is_empty() {
        return Err(NavigationError::new(
            NavigationErrorKind::BakeFailed,
            "native Recast bake produced no walkable polygons",
        ));
    }

    let area_costs = NavMeshAsset::empty(agent_type.clone()).area_costs;

    Ok(NavMeshAsset {
        version: NavMeshAsset::VERSION,
        agent_type,
        settings_hash: 0,
        area_costs,
        vertices,
        indices,
        polygons,
        tiles,
        off_mesh_links: Vec::new(),
    })
}

fn native_bake_message(result: &ZrNavRecastBakeResult) -> String {
    unsafe { CStr::from_ptr(result.message.as_ptr()) }
        .to_string_lossy()
        .trim()
        .to_string()
}

#[cfg(test)]
mod plan_tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn tiled_plan_clones_share_prepared_mesh_buffers() {
        let plan = RecastBackend::default()
            .prepare_tiled_bake(RecastTiledBakeInput {
                mesh: RecastBakeMeshInput {
                    agent_type: "humanoid".to_string(),
                    vertices: vec![[-2.0, 0.0, -1.0], [2.0, 0.0, -1.0], [0.0, 0.0, 1.0]],
                    indices: vec![0, 1, 2],
                    triangle_areas: Vec::new(),
                    default_area: 1,
                },
                tile_size: 1.0,
            })
            .unwrap();
        let cloned = plan.clone();

        assert!(Arc::ptr_eq(&plan.mesh, &cloned.mesh));
        assert!(Arc::ptr_eq(&plan.flat_vertices, &cloned.flat_vertices));
        assert!(Arc::ptr_eq(&plan.triangle_areas, &cloned.triangle_areas));
        assert!(Arc::ptr_eq(&plan.tiles, &cloned.tiles));
    }
}
