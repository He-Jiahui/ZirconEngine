use crate::asset::{NavMeshAsset, NavigationSettingsAsset};
use crate::core::math::Real;
use crate::scene::World;

use super::agent::NavAgentTickReport;
use super::bake::{NavMeshBakeReport, NavMeshBakeRequest};
use super::error::NavigationError;
use super::handle::NavMeshHandle;
use super::query::{
    NavPathQuery, NavPathResult, NavRaycastQuery, NavRaycastResult, NavSampleHit, NavSampleQuery,
};
use super::stats::NavigationRuntimeStats;

pub trait NavigationManager: Send + Sync {
    fn bake_surface(
        &self,
        world: &World,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError>;

    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError>;

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError>;

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError>;

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError>;

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError>;

    fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError>;

    fn tick_world_agent(
        &self,
        world: &mut World,
        entity: u64,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        let _ = entity;
        self.tick_world_agents(world, dt_seconds)
    }

    fn stats(&self) -> NavigationRuntimeStats;
}
