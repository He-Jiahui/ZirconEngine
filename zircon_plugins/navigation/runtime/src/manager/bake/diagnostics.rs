use zircon_runtime::core::framework::navigation::{
    NavMeshBakeDiagnostic, NavMeshBakeDiagnosticSeverity, NavMeshSurfaceDescriptor,
};

use super::geometry::BakeGeometry;

pub(super) fn bake_geometry_diagnostics(
    geometry: &BakeGeometry,
    surface_entity: Option<u64>,
) -> Vec<NavMeshBakeDiagnostic> {
    let mut diagnostics = vec![NavMeshBakeDiagnostic {
        severity: NavMeshBakeDiagnosticSeverity::Info,
        message: format!(
            "collected {} navigation bake source entity/entities ({} triangle(s))",
            geometry.source_entities,
            geometry.source_triangles()
        ),
        entity: surface_entity,
    }];
    if geometry.skipped_navigation_components > 0 {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Info,
            message: format!(
                "excluded {} navigation authoring/runtime component node(s) from bake geometry",
                geometry.skipped_navigation_components
            ),
            entity: surface_entity,
        });
    }
    if geometry.removed_by_modifier > 0 {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Info,
            message: format!(
                "removed {} bake source node(s) by NavMeshModifier",
                geometry.removed_by_modifier
            ),
            entity: surface_entity,
        });
    }
    if geometry.modified_by_area_override > 0 {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Info,
            message: format!(
                "applied area override to {} bake source node(s)",
                geometry.modified_by_area_override
            ),
            entity: surface_entity,
        });
    }
    if geometry.carved_by_obstacle > 0 {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Info,
            message: format!(
                "carved {} bake source node(s) by stationary NavMeshObstacle",
                geometry.carved_by_obstacle
            ),
            entity: surface_entity,
        });
    }
    diagnostics
}

pub(super) fn unsupported_bake_setting_diagnostics(
    surface: &NavMeshSurfaceDescriptor,
    surface_entity: Option<u64>,
) -> Vec<NavMeshBakeDiagnostic> {
    let mut diagnostics = Vec::new();
    if surface.override_voxel_size.is_some()
        || surface.override_tile_size.is_some()
        || surface.min_region_area != NavMeshSurfaceDescriptor::default().min_region_area
        || surface.build_height_mesh
    {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Warning,
            message: "advanced Recast bake knobs are recorded in the settings hash but the v1 fallback backend does not yet rasterize voxels, tiles, regions, or height meshes".to_string(),
            entity: surface_entity,
        });
    }
    diagnostics
}
