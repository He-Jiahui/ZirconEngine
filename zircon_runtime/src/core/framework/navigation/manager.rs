use super::error::NavigationError;
use super::handle::NavMeshHandle;
use super::query::{
    NavPathQuery, NavPathResult, NavQueryFilter, NavRaycastQuery, NavRaycastResult, NavSampleHit,
    NavSampleQuery,
};
use super::stats::NavigationRuntimeStats;
use super::{NavMeshAsset, NavigationSettingsAsset};

/// 导航查询的运行时边界；内建实现负责基础加载与查询，插件实现可提供烘焙和每次查询的过滤能力。
/// 调用者须先装载非空网格；未指定句柄时实现会选择默认已加载网格。
pub trait NavigationManager: Send + Sync {
    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError>;

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError>;

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError>;

    /// 为单次路径请求覆盖区域成本和 Detour 标志；内建管理器可能返回 BackendFailure，调用方应处理能力差异。
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
