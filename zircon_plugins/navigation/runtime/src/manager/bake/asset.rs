use crate::off_mesh_connections::collect_off_mesh_connections;
use zircon_plugin_navigation_recast::{RecastBakeInput, RecastBakeMeshInput, RecastTiledBakeInput};
use zircon_runtime::core::framework::navigation::{
    NavMeshAreaCostAsset, NavMeshAsset, NavigationSettingsAsset,
};
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeDiagnostic, NavMeshBakeDiagnosticSeverity, NavMeshSurfaceDescriptor, NavigationError,
};
use zircon_runtime::scene::World;

use super::geometry::BakeGeometry;
use crate::manager::DefaultNavigationManager;

pub(super) fn bake_nav_mesh_asset(
    manager: &DefaultNavigationManager,
    agent_type: &str,
    surface: &NavMeshSurfaceDescriptor,
    geometry: &BakeGeometry,
    half_extent: f32,
    surface_entity: Option<u64>,
    diagnostics: &mut Vec<NavMeshBakeDiagnostic>,
) -> Result<NavMeshAsset, NavigationError> {
    if geometry.source_triangles() > 0 {
        let mesh = RecastBakeMeshInput {
            agent_type: agent_type.to_string(),
            vertices: geometry.vertices.clone(),
            indices: geometry.indices.clone(),
            triangle_areas: geometry.triangle_areas.clone(),
            default_area: surface.default_area,
        };
        if let Some(tile_size) = surface.override_tile_size {
            return manager.backend.bake_tiled_mesh(RecastTiledBakeInput {
                mesh,
                tile_size: tile_size as f32,
            });
        }
        return manager.backend.bake_triangle_mesh(mesh);
    }
    if geometry.carved_by_obstacle > 0 || geometry.removed_by_modifier > 0 {
        return Ok(NavMeshAsset::empty(agent_type.to_string()));
    }

    diagnostics.push(NavMeshBakeDiagnostic {
        severity: NavMeshBakeDiagnosticSeverity::Warning,
        message:
            "no render mesh or collider bake source was collected; baked surface volume fallback"
                .to_string(),
        entity: surface_entity,
    });
    manager.backend.bake_simple_surface(RecastBakeInput {
        agent_type: agent_type.to_string(),
        source_vertices: 4,
        source_triangles: 2,
        half_extent,
    })
}

pub(super) fn stamp_asset_settings(
    asset: &mut NavMeshAsset,
    surface: &NavMeshSurfaceDescriptor,
    settings: &NavigationSettingsAsset,
) {
    asset.settings_hash = crate::settings_hash::navigation_settings_hash(surface, settings);
    asset.area_costs = settings
        .areas
        .iter()
        .map(|area| NavMeshAreaCostAsset {
            area: area.id,
            cost: area.cost,
            walkable: area.walkable,
        })
        .collect();
}

pub(super) fn embed_off_mesh_links(
    world: &World,
    agent_type: &str,
    surface: &NavMeshSurfaceDescriptor,
    surface_entity: Option<u64>,
    asset: &mut NavMeshAsset,
    diagnostics: &mut Vec<NavMeshBakeDiagnostic>,
) {
    let off_mesh_links = if surface.generate_links {
        collect_off_mesh_connections(world, agent_type)
    } else {
        Vec::new()
    };
    if off_mesh_links.is_empty() {
        return;
    }

    diagnostics.push(NavMeshBakeDiagnostic {
        severity: NavMeshBakeDiagnosticSeverity::Info,
        message: format!(
            "embedded {} active off-mesh link(s) in baked navigation asset",
            off_mesh_links.len()
        ),
        entity: surface_entity,
    });
    asset.off_mesh_links = off_mesh_links;
}
