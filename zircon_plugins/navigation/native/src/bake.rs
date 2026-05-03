use std::ffi::CStr;
use std::slice;

use zircon_runtime::asset::{NavMeshAsset, NavMeshPolygonAsset, NavMeshTileAsset};
use zircon_runtime::core::framework::navigation::{
    NavAreaId, NavigationError, NavigationErrorKind,
};
use zircon_runtime::core::math::Real;

use crate::ffi::{self, ZrNavRecastBakeResult, ZrNavRecastBakeSettings};
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
    if result.vertices.is_null()
        || result.indices.is_null()
        || result.polygons.is_null()
        || result.tiles.is_null()
    {
        return Err(NavigationError::new(
            NavigationErrorKind::BackendFailure,
            "native Recast bake returned incomplete output buffers",
        ));
    }

    let vertex_values =
        unsafe { slice::from_raw_parts(result.vertices, result.vertex_count as usize * 3) };
    let vertices = vertex_values
        .chunks_exact(3)
        .map(|vertex| [vertex[0], vertex[1], vertex[2]])
        .collect::<Vec<_>>();
    let indices =
        unsafe { slice::from_raw_parts(result.indices, result.index_count as usize) }.to_vec();
    let polygons = unsafe { slice::from_raw_parts(result.polygons, result.polygon_count as usize) }
        .iter()
        .map(|polygon| NavMeshPolygonAsset {
            first_index: polygon.first_index,
            index_count: polygon.index_count,
            area: polygon.area,
            tile: polygon.tile,
        })
        .collect::<Vec<_>>();
    let tiles = unsafe { slice::from_raw_parts(result.tiles, result.tile_count as usize) }
        .iter()
        .map(|tile| NavMeshTileAsset {
            id: tile.id,
            bounds_min: tile.bounds_min,
            bounds_max: tile.bounds_max,
            polygon_count: tile.polygon_count,
        })
        .collect::<Vec<_>>();

    if vertices.is_empty() || indices.is_empty() || polygons.is_empty() {
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
