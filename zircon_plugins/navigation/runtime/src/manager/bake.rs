mod area_volume;
mod asset;
mod diagnostics;
mod dirty;
mod filter;
mod geometry;
mod geometry_provider;
mod modifier;
mod source_selection;
mod surface;
pub(super) mod task_pool;
mod tiled;

#[cfg(test)]
#[path = "bake/tests/geometry_provider_generation_tests.rs"]
mod geometry_provider_generation_tests;

use std::any::Any;

use zircon_runtime::asset::{ProjectAssetGenerationToken, ProjectGenerationMatch};
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeReport, NavMeshBakeRequest, NavMeshSurfaceDescriptor, NavigationError,
    NavigationErrorKind, NavigationGeneratedBakeSnapshot,
};
use zircon_runtime::scene::{LevelSystem, World, WorldPublicationError, WorldPublicationSource};

use self::asset::{bake_nav_mesh_asset, embed_off_mesh_links, stamp_asset_settings};
use self::diagnostics::{bake_geometry_diagnostics, unsupported_bake_setting_diagnostics};
use self::geometry::collect_bake_geometry;
use self::surface::{collect_surfaces, select_bake_surface};
use super::stats::count_obstacles;
use super::DefaultNavigationManager;
use crate::off_mesh_connections::collect_off_mesh_connections;

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
    source_world_generation: u64,
    project_asset_generation: Option<ProjectAssetGenerationToken>,
    off_mesh_links: Vec<zircon_runtime::core::framework::navigation::NavMeshLinkAsset>,
    counts: (usize, usize, usize),
}

/// All expensive preparation is completed before a World publication guard is acquired.
#[derive(Debug)]
struct PreparedBakePublication {
    report: NavMeshBakeReport,
    surface: Option<u64>,
    generation: u64,
    tiled_bake: Option<(
        super::state::TiledBakeIdentity,
        zircon_plugin_navigation_recast::RecastTiledBakePlan,
        zircon_runtime::core::framework::navigation::NavMeshAsset,
    )>,
    generated_snapshot: NavigationGeneratedBakeSnapshot,
    diagnostics: Vec<zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic>,
    counts: (usize, usize, usize),
    source_world_generation: u64,
    project_asset_generation: Option<ProjectAssetGenerationToken>,
}

pub(super) struct BakeOperationSnapshot {
    preparation: BakePreparation,
    generation: super::state::BakeGenerationToken,
}

pub(super) struct BakeOperationPrepared {
    publication: PreparedBakePublication,
    generation: super::state::BakeGenerationToken,
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

// 同步烘焙先取得表面代次，只有未被较新请求取代的结果才能发布快照和统计。
pub(super) fn bake_surface(
    manager: &DefaultNavigationManager,
    level: &LevelSystem,
    request: NavMeshBakeRequest,
) -> Result<NavMeshBakeReport, NavigationError> {
    let source = level.capture();
    let generated_mutation_epoch = manager.capture_generated_mutation_epoch();
    let preparation = prepare_bake(manager, source.snapshot(), request)?;
    if manager.capture_generated_mutation_epoch() != generated_mutation_epoch {
        return Err(NavigationError::new(
            NavigationErrorKind::InvalidConfiguration,
            "navigation manager generated state changed during owner bake preparation",
        ));
    }
    let generation = manager.begin_bake_generation(preparation.surface_entity)?;
    let prepared = prepare_bake_publication_from_preparation(manager, preparation, generation)?;
    if prepared.source_world_generation != source.generation() {
        return Err(world_generation_error(
            source.generation(),
            prepared.source_world_generation,
        ));
    }
    let report = prepared.report.clone();
    publish_bake_with_source_fence(
        manager,
        &source,
        prepared.source_world_generation,
        prepared.project_asset_generation.as_ref(),
        prepared.surface,
        prepared.generation,
        generated_mutation_epoch,
        prepared.tiled_bake,
        prepared.generated_snapshot,
        prepared.diagnostics,
        prepared.counts,
    )?;
    Ok(report)
}

pub(super) fn capture_bake_operation(
    manager: &DefaultNavigationManager,
    source: &WorldPublicationSource,
    request: NavMeshBakeRequest,
) -> Result<(Box<dyn Any + Send>, usize, NavigationGeneratedBakeSnapshot), NavigationError> {
    let surface_hint = canonical_surface_key(source.snapshot(), request.surface_entity);
    let generation_before = manager.capture_bake_generation(surface_hint);
    let generated_mutation_epoch_before = manager.capture_generated_mutation_epoch();
    let preparation = prepare_bake(manager, source.snapshot(), request)?;
    if preparation.source_world_generation != source.generation() {
        return Err(world_generation_error(
            source.generation(),
            preparation.source_world_generation,
        ));
    }
    let generation = manager.capture_bake_generation(preparation.surface_entity);
    if generation != generation_before
        || preparation.surface_entity != generation_before.surface
        || manager.capture_generated_mutation_epoch() != generated_mutation_epoch_before
    {
        return Err(NavigationError::new(
            NavigationErrorKind::InvalidConfiguration,
            "navigation manager bake state changed during owner snapshot",
        ));
    }
    let before = manager
        .lock_state()
        .generated_snapshot(preparation.surface_entity);
    let retained_bytes = preparation_retained_bytes(&preparation);
    Ok((
        Box::new(BakeOperationSnapshot {
            preparation,
            generation,
        }),
        retained_bytes,
        before,
    ))
}

pub(super) fn prepare_bake_operation(
    manager: &DefaultNavigationManager,
    snapshot: Box<dyn Any + Send>,
) -> Result<
    (
        Box<dyn Any + Send>,
        NavMeshBakeReport,
        NavigationGeneratedBakeSnapshot,
        usize,
    ),
    NavigationError,
> {
    let snapshot = snapshot
        .downcast::<BakeOperationSnapshot>()
        .map_err(|_| invalid_operation_snapshot())?;
    let BakeOperationSnapshot {
        preparation,
        generation,
    } = *snapshot;
    let publication = prepare_bake_publication_from_preparation(
        manager,
        preparation,
        generation.next_generation,
    )?;
    let report = publication.report.clone();
    let generated_snapshot = publication.generated_snapshot.clone();
    let owner_bytes = publication_retained_bytes(&publication);
    Ok((
        Box::new(BakeOperationPrepared {
            publication,
            generation,
        }),
        report,
        generated_snapshot,
        owner_bytes,
    ))
}

pub(super) fn apply_bake_operation(
    manager: &DefaultNavigationManager,
    source: &WorldPublicationSource,
    expected_before: &NavigationGeneratedBakeSnapshot,
    prepared: Box<dyn Any + Send>,
) -> Result<(), NavigationError> {
    let prepared = prepared
        .downcast::<BakeOperationPrepared>()
        .map_err(|_| invalid_operation_snapshot())?;
    let BakeOperationPrepared {
        publication,
        generation,
    } = *prepared;
    publish_operation_bake_with_source_fence(
        manager,
        source,
        generation,
        expected_before,
        publication.source_world_generation,
        publication.project_asset_generation.as_ref(),
        publication.tiled_bake,
        publication.generated_snapshot,
        publication.diagnostics,
        publication.counts,
    )
}

fn preparation_retained_bytes(preparation: &BakePreparation) -> usize {
    preparation
        .geometry
        .vertices
        .len()
        .saturating_mul(std::mem::size_of::<[f32; 3]>())
        .saturating_add(
            preparation
                .geometry
                .indices
                .len()
                .saturating_mul(std::mem::size_of::<u32>()),
        )
        .saturating_add(
            preparation
                .geometry
                .triangle_areas
                .len()
                .saturating_mul(std::mem::size_of::<u8>()),
        )
        .saturating_add(
            preparation
                .off_mesh_links
                .len()
                .saturating_mul(std::mem::size_of::<
                    zircon_runtime::core::framework::navigation::NavMeshLinkAsset,
                >()),
        )
        .saturating_add(
            preparation
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.len())
                .sum::<usize>(),
        )
}

fn publication_retained_bytes(publication: &PreparedBakePublication) -> usize {
    let tile_bytes = publication.tiled_bake.as_ref().map_or(0, |(_, plan, _)| {
        plan.tiles().len().saturating_mul(std::mem::size_of::<
            zircon_plugin_navigation_recast::RecastTileSpec,
        >())
    });
    tile_bytes.saturating_add(
        publication
            .report
            .asset
            .as_ref()
            .map_or(0, |asset| {
                asset
                    .vertices
                    .len()
                    .saturating_mul(std::mem::size_of::<[f32; 3]>())
                    .saturating_add(
                        asset
                            .indices
                            .len()
                            .saturating_mul(std::mem::size_of::<u32>()),
                    )
                    .saturating_add(
                        asset
                            .polygons
                            .len()
                            .saturating_mul(std::mem::size_of::<u32>()),
                    )
                    .saturating_add(
                        asset
                            .off_mesh_links
                            .len()
                            .saturating_mul(std::mem::size_of::<
                                zircon_runtime::core::framework::navigation::NavMeshLinkAsset,
                            >()),
                    )
            })
            .saturating_add(
                publication
                    .diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.message.len())
                    .sum::<usize>(),
            ),
    )
}

fn invalid_operation_snapshot() -> NavigationError {
    NavigationError::new(
        NavigationErrorKind::InvalidConfiguration,
        "navigation operation snapshot did not match its captured backend type",
    )
}

fn prepare_bake_publication_from_preparation(
    manager: &DefaultNavigationManager,
    mut preparation: BakePreparation,
    generation: u64,
) -> Result<PreparedBakePublication, NavigationError> {
    let context_surface = preparation.surface_entity;
    let counts = preparation.counts;
    let mut asset = bake_nav_mesh_asset(
        &manager.backend,
        &preparation.agent_type,
        &preparation.surface,
        &preparation.geometry,
        fallback_surface_half_extent(&preparation.surface),
        preparation.surface_entity,
        &mut preparation.diagnostics,
    )?;
    let tiled_plan = tiled::plan_for_preparation(&manager.backend, &preparation)?;
    let tiled_identity = preparation.tiled_identity();
    let source_world_generation = preparation.source_world_generation;
    let project_asset_generation = preparation.project_asset_generation.clone();
    let report = finish_bake(preparation, &mut asset);
    let generated_snapshot = NavigationGeneratedBakeSnapshot {
        surface_entity: context_surface,
        asset: report.asset.clone(),
        output_asset: report.output_asset.clone(),
    };
    let tiled_bake = tiled_plan.map(|plan| (tiled_identity, plan, asset));
    Ok(PreparedBakePublication {
        report: report.clone(),
        surface: context_surface,
        generation,
        tiled_bake,
        generated_snapshot,
        diagnostics: report.diagnostics.clone(),
        counts,
        source_world_generation,
        project_asset_generation,
    })
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
    let source_world_generation = world.world_generation();
    let project_asset_manager = manager
        .project_asset_access()
        .resolve()
        .map_err(|error| asset_manager_error(error.to_string()))?;
    let project_asset_generation = project_asset_manager
        .current_project_generation_snapshot()
        .map(|snapshot| snapshot.into_parts().1);
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

    let geometry = collect_bake_geometry(
        &project_asset_manager,
        world,
        surface_entity,
        &surface,
        &agent_type,
    )?;
    ensure_world_generation(world, source_world_generation)?;
    ensure_project_generation(&project_asset_manager, project_asset_generation.as_ref())?;
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
    let off_mesh_links = if surface.generate_links {
        collect_off_mesh_connections(world, &agent_type)
    } else {
        Vec::new()
    };
    Ok(BakePreparation {
        surfaces: surfaces.len(),
        surface,
        surface_entity,
        agent_type,
        settings,
        geometry,
        diagnostics,
        output_asset,
        source_world_generation,
        project_asset_generation,
        off_mesh_links,
        counts: bake_runtime_counts(world),
    })
}

pub(super) fn ensure_world_generation(world: &World, expected: u64) -> Result<(), NavigationError> {
    let actual = world.world_generation();
    if actual == expected {
        return Ok(());
    }
    Err(NavigationError::new(
        NavigationErrorKind::InvalidConfiguration,
        format!(
            "navigation geometry snapshot was superseded by World generation {actual} (captured {expected})"
        ),
    ))
}

fn ensure_project_generation(
    project_assets: &zircon_runtime::asset::ProjectAssetManager,
    expected: Option<&ProjectAssetGenerationToken>,
) -> Result<(), NavigationError> {
    let Some(expected) = expected else {
        return Ok(());
    };
    if project_assets.check_project_generation(expected) == ProjectGenerationMatch::Current {
        return Ok(());
    }
    Err(NavigationError::new(
        NavigationErrorKind::InvalidConfiguration,
        "navigation geometry snapshot was superseded by a newer project asset generation",
    ))
}

fn asset_manager_error(message: String) -> NavigationError {
    NavigationError::new(
        NavigationErrorKind::InvalidConfiguration,
        format!("navigation could not resolve the active ProjectAssetManager: {message}"),
    )
}

fn world_generation_error(expected: u64, actual: u64) -> NavigationError {
    NavigationError::new(
        NavigationErrorKind::InvalidConfiguration,
        format!(
            "navigation World publication generation changed (expected {expected}, actual {actual})"
        ),
    )
}

fn publish_bake_with_source_fence(
    manager: &DefaultNavigationManager,
    source: &WorldPublicationSource,
    source_world_generation: u64,
    project_asset_generation: Option<&ProjectAssetGenerationToken>,
    surface: Option<u64>,
    generation: u64,
    generated_mutation_epoch: u64,
    tiled_bake: Option<(
        super::state::TiledBakeIdentity,
        zircon_plugin_navigation_recast::RecastTiledBakePlan,
        zircon_runtime::core::framework::navigation::NavMeshAsset,
    )>,
    generated_snapshot: NavigationGeneratedBakeSnapshot,
    diagnostics: Vec<zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic>,
    counts: (usize, usize, usize),
) -> Result<(), NavigationError> {
    if source.generation() != source_world_generation {
        return Err(world_generation_error(
            source_world_generation,
            source.generation(),
        ));
    }
    let project_assets = project_asset_generation
        .map(|_| {
            manager
                .project_asset_access()
                .resolve()
                .map_err(|error| asset_manager_error(error.to_string()))
        })
        .transpose()?;
    source
        .publish(|_| {
            let publish = || {
                manager.publish_bake(
                    surface,
                    generation,
                    generated_mutation_epoch,
                    tiled_bake,
                    generated_snapshot,
                    diagnostics,
                    counts,
                )
            };
            let Some(project_asset_generation) = project_asset_generation else {
                return publish();
            };
            let project_assets = project_assets
                .as_ref()
                .expect("project asset manager resolved for its generation token");
            match project_assets.commit_if_project_generation(project_asset_generation, publish) {
                zircon_runtime::asset::ProjectGenerationCommitOutcome::Committed(result) => result,
                zircon_runtime::asset::ProjectGenerationCommitOutcome::Superseded { .. } => {
                    Err(NavigationError::new(
                        NavigationErrorKind::InvalidConfiguration,
                        "navigation bake was not published because its project asset generation changed",
                    ))
                }
            }
        })
        .map_err(world_publication_error)?
}

fn publish_operation_bake_with_source_fence(
    manager: &DefaultNavigationManager,
    source: &WorldPublicationSource,
    token: super::state::BakeGenerationToken,
    expected_before: &NavigationGeneratedBakeSnapshot,
    source_world_generation: u64,
    project_asset_generation: Option<&ProjectAssetGenerationToken>,
    tiled_bake: Option<(
        super::state::TiledBakeIdentity,
        zircon_plugin_navigation_recast::RecastTiledBakePlan,
        zircon_runtime::core::framework::navigation::NavMeshAsset,
    )>,
    generated_snapshot: NavigationGeneratedBakeSnapshot,
    diagnostics: Vec<zircon_runtime::core::framework::navigation::NavMeshBakeDiagnostic>,
    counts: (usize, usize, usize),
) -> Result<(), NavigationError> {
    if source.generation() != source_world_generation {
        return Err(world_generation_error(
            source_world_generation,
            source.generation(),
        ));
    }
    let project_assets = project_asset_generation
        .map(|_| {
            manager
                .project_asset_access()
                .resolve()
                .map_err(|error| asset_manager_error(error.to_string()))
        })
        .transpose()?;
    source
        .publish(|_| {
            let publish = || {
                manager.publish_operation_bake(
                    token,
                    expected_before,
                    tiled_bake,
                    generated_snapshot,
                    diagnostics,
                    counts,
                )
            };
            let Some(project_asset_generation) = project_asset_generation else {
                return publish();
            };
            let project_assets = project_assets
                .as_ref()
                .expect("project asset manager resolved for its generation token");
            match project_assets.commit_if_project_generation(project_asset_generation, publish) {
                zircon_runtime::asset::ProjectGenerationCommitOutcome::Committed(result) => result,
                zircon_runtime::asset::ProjectGenerationCommitOutcome::Superseded { .. } => {
                    Err(NavigationError::new(
                        NavigationErrorKind::InvalidConfiguration,
                        "navigation bake was not published because its project asset generation changed",
                    ))
                }
            }
        })
        .map_err(world_publication_error)?
}

fn world_publication_error(error: WorldPublicationError) -> NavigationError {
    NavigationError::new(
        NavigationErrorKind::InvalidConfiguration,
        format!("navigation World publication was rejected: {error}"),
    )
}

fn finish_bake(
    mut preparation: BakePreparation,
    asset: &mut zircon_runtime::core::framework::navigation::NavMeshAsset,
) -> NavMeshBakeReport {
    stamp_asset_settings(asset, &preparation.surface, &preparation.settings);
    embed_off_mesh_links(
        std::mem::take(&mut preparation.off_mesh_links),
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
