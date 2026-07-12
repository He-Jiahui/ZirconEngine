use zircon_plugin_navigation_recast::{
    RecastBakeMeshInput, RecastTiledBakeInput, RecastTiledBakePlan,
};
use zircon_runtime::core::framework::navigation::NavigationError;

use super::BakePreparation;
use crate::manager::DefaultNavigationManager;

pub(super) fn plan_for_preparation(
    manager: &DefaultNavigationManager,
    preparation: &BakePreparation,
) -> Result<Option<RecastTiledBakePlan>, NavigationError> {
    let Some(tile_size) = preparation.surface.override_tile_size else {
        return Ok(None);
    };
    if preparation.geometry.source_triangles() == 0 {
        return Ok(None);
    }
    manager
        .backend
        .prepare_tiled_bake(RecastTiledBakeInput {
            mesh: mesh_input(preparation),
            tile_size: tile_size as f32,
        })
        .map(Some)
}

pub(super) fn mesh_input(preparation: &BakePreparation) -> RecastBakeMeshInput {
    RecastBakeMeshInput {
        agent_type: preparation.agent_type.clone(),
        vertices: preparation.geometry.vertices.clone(),
        indices: preparation.geometry.indices.clone(),
        triangle_areas: preparation.geometry.triangle_areas.clone(),
        default_area: preparation.surface.default_area,
    }
}
