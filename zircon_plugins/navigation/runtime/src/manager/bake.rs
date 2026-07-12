mod area_volume;
mod asset;
mod diagnostics;
mod filter;
mod geometry;
mod modifier;
mod source_selection;
mod surface;

use zircon_runtime::core::framework::navigation::{
    NavMeshBakeReport, NavMeshBakeRequest, NavMeshSurfaceDescriptor, NavigationError,
    NavigationErrorKind,
};
use zircon_runtime::scene::World;

use self::asset::{bake_nav_mesh_asset, embed_off_mesh_links, stamp_asset_settings};
use self::diagnostics::{bake_geometry_diagnostics, unsupported_bake_setting_diagnostics};
use self::geometry::collect_bake_geometry;
use self::surface::{collect_surfaces, select_bake_surface};
use super::stats::count_obstacles;
use super::DefaultNavigationManager;

pub(super) fn bake_surface(
    manager: &DefaultNavigationManager,
    world: &World,
    request: NavMeshBakeRequest,
) -> Result<NavMeshBakeReport, NavigationError> {
    let surfaces = collect_surfaces(world);
    let selected_surface = select_bake_surface(&surfaces, request.surface_entity);
    let surface = selected_surface
        .as_ref()
        .map(|(_, surface)| surface.clone())
        .unwrap_or_default();
    let surface_entity = selected_surface.as_ref().map(|(entity, _)| *entity);
    let agent_type = request
        .agent_type
        .clone()
        .unwrap_or_else(|| surface.agent_type.clone());
    let settings = manager.active_settings();
    validate_agent_type(&settings, &agent_type)?;

    let geometry = collect_bake_geometry(world, surface_entity, &surface, &agent_type);
    let mut diagnostics = bake_geometry_diagnostics(&geometry, surface_entity);
    diagnostics.extend(unsupported_bake_setting_diagnostics(
        &surface,
        surface_entity,
    ));

    let mut asset = bake_nav_mesh_asset(
        manager,
        &agent_type,
        &surface,
        &geometry,
        fallback_surface_half_extent(&surface),
        surface_entity,
        &mut diagnostics,
    )?;
    stamp_asset_settings(&mut asset, &surface, &settings);
    embed_off_mesh_links(
        world,
        &agent_type,
        &surface,
        surface_entity,
        &mut asset,
        &mut diagnostics,
    );

    let output_asset = request.output_asset.or(surface.output_asset.clone());
    manager.record_bake_counts(
        count_obstacles(world),
        crate::off_mesh_connections::count_off_mesh_links(world),
        crate::off_mesh_connections::count_off_mesh_bridges(world),
    );

    Ok(NavMeshBakeReport {
        asset: Some(asset.clone()),
        output_asset,
        surfaces: surfaces.len(),
        source_vertices: geometry.vertices.len(),
        source_triangles: geometry.source_triangles(),
        baked_vertices: asset.vertices.len(),
        baked_polygons: asset.polygons.len(),
        tiles: asset.tiles.len(),
        diagnostics,
    })
}

fn validate_agent_type(
    settings: &zircon_runtime::asset::NavigationSettingsAsset,
    agent_type: &str,
) -> Result<(), NavigationError> {
    if settings.agents.iter().any(|agent| agent.id == agent_type) {
        return Ok(());
    }
    Err(NavigationError::new(
        NavigationErrorKind::InvalidConfiguration,
        format!("navigation settings do not define agent type `{agent_type}`"),
    ))
}

fn fallback_surface_half_extent(surface: &NavMeshSurfaceDescriptor) -> f32 {
    surface
        .volume_size
        .into_iter()
        .fold(0.0_f32, |largest, value| largest.max(value))
        .max(1.0)
        * 0.5
}
