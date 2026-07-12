use zircon_runtime::core::framework::navigation::{
    NavPathQuery, NavPathResult, NavQueryFilter, NavRaycastQuery, NavRaycastResult, NavSampleHit,
    NavSampleQuery, NavigationError,
};

use super::DefaultNavigationManager;

pub(super) fn find_path(
    manager: &DefaultNavigationManager,
    query: NavPathQuery,
) -> Result<NavPathResult, NavigationError> {
    let asset = manager.selected_asset(query.nav_mesh)?;
    manager.backend.find_path(&asset, &query)
}

pub(super) fn find_path_with_filter(
    manager: &DefaultNavigationManager,
    query: NavPathQuery,
    filter: &NavQueryFilter,
) -> Result<NavPathResult, NavigationError> {
    let asset = manager.selected_asset(query.nav_mesh)?;
    manager
        .backend
        .find_path_with_filter(&asset, &query, filter)
}

pub(super) fn sample_position(
    manager: &DefaultNavigationManager,
    query: NavSampleQuery,
) -> Result<Option<NavSampleHit>, NavigationError> {
    let asset = manager.selected_asset(query.nav_mesh)?;
    manager.backend.sample_position(&asset, &query)
}

pub(super) fn raycast(
    manager: &DefaultNavigationManager,
    query: NavRaycastQuery,
) -> Result<NavRaycastResult, NavigationError> {
    let asset = manager.selected_asset(query.nav_mesh)?;
    manager.backend.raycast(&asset, &query)
}
