use super::error::NavigationError;
use super::handle::NavMeshHandle;
use super::query::{
    NavPathQuery, NavPathResult, NavQueryFilter, NavRaycastQuery, NavRaycastResult, NavSampleHit,
    NavSampleQuery,
};
use super::stats::NavigationRuntimeStats;
use super::{NavMeshAsset, NavigationSettingsAsset};

pub trait NavigationManager: Send + Sync {
    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError>;

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError>;

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError>;

    fn find_path_with_filter(
        &self,
        query: NavPathQuery,
        filter: &NavQueryFilter,
    ) -> Result<NavPathResult, NavigationError>;

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError>;

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError>;

    fn stats(&self) -> NavigationRuntimeStats;
}
