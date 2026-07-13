mod area_volume;
mod asset;
mod diagnostics;
mod dirty;
mod filter;
mod geometry;
mod modifier;
mod source_selection;
mod surface;
pub(super) mod task_pool;
mod tiled;

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

pub(in crate::manager) use dirty::PendingDirtyBake;
pub use dirty::{NavMeshDirtyBakeReport, NavMeshDirtyBounds};
pub use task_pool::{NavMeshBakeTaskHandle, NavMeshBakeTaskState};

#[derive(Clone, Debug)]
struct BakePreparation {
    surfaces: usize,
    surface: NavMeshSurfaceDescriptor,
    surface_entity: Option<u64>,
    agent_type: String,
    settings: zircon_runtime::core::framework::navigation::NavigationSettingsAsset,
    geometry: geometry::BakeGeometry,
    diagnostics: Vec<zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic>,
    output_asset: Option<String>,
}

impl BakePreparation {
    fn tiled_identity(&self) -> super::state::TiledBakeIdentity {
        super::state::TiledBakeIdentity {
            surface_entity: self.surface_entity,
            agent_type: self.agent_type.clone(),
            surface: self.surface.clone(),
            settings: self.settings.clone(),
        }
    }
}

pub(super) fn bake_surface(
    manager: &DefaultNavigationManager,
    world: &World,
    request: NavMeshBakeRequest,
) -> Result<NavMeshBakeReport, NavigationError> {
    let mut preparation = prepare_bake(manager, world, request)?;
    let context_surface = preparation.surface_entity;
    let generation = manager.begin_bake_generation(context_surface);
    let mut asset = bake_nav_mesh_asset(
        manager,
        &preparation.agent_type,
        &preparation.surface,
        &preparation.geometry,
        fallback_surface_half_extent(&preparation.surface),
        preparation.surface_entity,
        &mut preparation.diagnostics,
    )?;
    let tiled_plan = tiled::plan_for_preparation(manager, &preparation)?;
    let tiled_identity = preparation.tiled_identity();
    let report = finish_bake(world, preparation, &mut asset);
    let tiled_bake = tiled_plan.map(|plan| (tiled_identity, plan, asset));
    manager.publish_bake(
        context_surface,
        generation,
        tiled_bake,
        report.diagnostics.clone(),
        bake_runtime_counts(world),
    )?;
    Ok(report)
}

fn canonical_surface_key(world: &World, requested: Option<u64>) -> Option<u64> {
    let surfaces = collect_surfaces(world);
    select_bake_surface(&surfaces, requested).map(|(entity, _)| entity)
}

fn prepare_bake(
    manager: &DefaultNavigationManager,
    world: &World,
    request: NavMeshBakeRequest,
) -> Result<BakePreparation, NavigationError> {
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
    if let Some(tile_size) = surface.override_tile_size {
        diagnostics.push(
            zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic {
                severity: zircon_runtime::core::framework::navigation::NavMeshBakeDiagnosticSeverity::Info,
                message: format!(
                    "baking Recast tile grid with {tile_size} world-unit tile size"
                ),
                entity: surface_entity,
            },
        );
    }

    let output_asset = request.output_asset.or(surface.output_asset.clone());
    Ok(BakePreparation {
        surfaces: surfaces.len(),
        surface,
        surface_entity,
        agent_type,
        settings,
        geometry,
        diagnostics,
        output_asset,
    })
}

fn finish_bake(
    world: &World,
    mut preparation: BakePreparation,
    asset: &mut zircon_runtime::core::framework::navigation::NavMeshAsset,
) -> NavMeshBakeReport {
    stamp_asset_settings(asset, &preparation.surface, &preparation.settings);
    embed_off_mesh_links(
        world,
        &preparation.agent_type,
        &preparation.surface,
        preparation.surface_entity,
        asset,
        &mut preparation.diagnostics,
    );

    NavMeshBakeReport {
        asset: Some(asset.clone()),
        output_asset: preparation.output_asset,
        surfaces: preparation.surfaces,
        source_vertices: preparation.geometry.vertices.len(),
        source_triangles: preparation.geometry.source_triangles(),
        baked_vertices: asset.vertices.len(),
        baked_polygons: asset.polygons.len(),
        tiles: asset.tiles.len(),
        diagnostics: preparation.diagnostics,
    }
}

fn bake_runtime_counts(world: &World) -> (usize, usize, usize) {
    (
        count_obstacles(world),
        crate::off_mesh_connections::count_off_mesh_links(world),
        crate::off_mesh_connections::count_off_mesh_bridges(world),
    )
}

fn validate_agent_type(
    settings: &zircon_runtime::core::framework::navigation::NavigationSettingsAsset,
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
