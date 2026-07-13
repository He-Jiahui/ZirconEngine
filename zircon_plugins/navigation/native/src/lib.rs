use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavPathQuery, NavPathResult, NavQueryFilter, NavRaycastQuery, NavRaycastResult, NavSampleHit,
    NavSampleQuery, NavigationError,
};

mod asset_ffi;
mod bake;
mod crowd;
mod detour;
mod detour_result;
mod fallback_query;
mod ffi;
mod tile_cache;

pub use bake::{
    merge_tiled_assets, RecastBakeInput, RecastBakeMeshInput, RecastTileSpec, RecastTiledBakeInput,
    RecastTiledBakePlan,
};
pub use crowd::{RecastCrowd, RecastCrowdAgentHandle, RecastCrowdAgentState, RecastCrowdConfig};
pub use tile_cache::{
    RecastNavigationObstacle, RecastNavigationObstacleShape, RecastTileCache,
    RecastTileCacheObstacleHandle,
};

pub fn native_backend_version() -> u32 {
    unsafe { ffi::zr_nav_recast_bridge_version() }
}

pub fn native_runtime_modules_available() -> bool {
    unsafe { ffi::zr_nav_recast_runtime_modules_smoke() == 1 }
}

#[derive(Clone, Debug, Default)]
pub struct RecastBackend;

impl RecastBackend {
    pub fn find_path(
        &self,
        asset: &NavMeshAsset,
        query: &NavPathQuery,
    ) -> Result<NavPathResult, NavigationError> {
        let filter = asset_ffi::asset_query_filter(asset);
        self.find_path_with_filter(asset, query, &filter)
    }

    pub fn find_path_with_filter(
        &self,
        asset: &NavMeshAsset,
        query: &NavPathQuery,
        filter: &NavQueryFilter,
    ) -> Result<NavPathResult, NavigationError> {
        fallback_query::validate_query_agent(asset, &query.agent_type)?;
        if asset.is_empty() {
            return Ok(NavPathResult::no_path());
        }
        if let Some(result) = detour::find_path(asset, query, filter) {
            return Ok(result);
        }
        Ok(fallback_query::find_path(asset, query, filter))
    }

    pub fn find_path_with_obstacles(
        &self,
        asset: &NavMeshAsset,
        query: &NavPathQuery,
        obstacles: &[RecastNavigationObstacle],
    ) -> Result<NavPathResult, NavigationError> {
        if obstacles.is_empty() {
            return self.find_path(asset, query);
        }
        fallback_query::validate_query_agent(asset, &query.agent_type)?;
        if asset.is_empty() {
            return Ok(NavPathResult::no_path());
        }
        if let Some(result) = tile_cache::find_path(asset, query, obstacles) {
            return Ok(result);
        }
        self.find_path(asset, query)
    }

    pub fn sample_position(
        &self,
        asset: &NavMeshAsset,
        query: &NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        fallback_query::validate_query_agent(asset, &query.agent_type)?;
        if asset.is_empty() {
            return Ok(None);
        }
        if let Some(result) = detour::sample_position(asset, query) {
            return Ok(result);
        }
        Ok(fallback_query::sample_position(asset, query))
    }

    pub fn raycast(
        &self,
        asset: &NavMeshAsset,
        query: &NavRaycastQuery,
    ) -> Result<NavRaycastResult, NavigationError> {
        fallback_query::validate_query_agent(asset, &query.agent_type)?;
        if asset.is_empty() {
            return Ok(fallback_query::blocked_raycast_result(query));
        }
        let Some(start_polygon) =
            fallback_query::containing_allowed_polygon(asset, query.start, query.area_mask)
        else {
            return Ok(fallback_query::blocked_raycast_result(query));
        };
        if let Some(result) = detour::raycast(asset, query) {
            return Ok(result);
        }
        Ok(fallback_query::raycast_from_polygon(
            asset,
            query,
            start_polygon,
        ))
    }
}

#[cfg(test)]
mod tests;
