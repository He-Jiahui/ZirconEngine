use std::fmt;
use std::sync::Arc;

use crate::core::framework::navigation::{
    NavAgentTickReport, NavMeshBakeReport, NavMeshBakeRequest, NavigationError,
};
use crate::core::math::Real;
use crate::scene::World;

pub const SCENE_NAVIGATION_RUNTIME_DRIVER_NAME: &str =
    "navigation.runtime.Driver.SceneNavigationRuntime";

pub trait SceneNavigationRuntime: Send + Sync {
    fn bake_surface(
        &self,
        world: &World,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError>;

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
}

#[derive(Clone)]
pub struct SceneNavigationRuntimeHandle {
    runtime: Arc<dyn SceneNavigationRuntime>,
}

impl SceneNavigationRuntimeHandle {
    pub fn new<T>(runtime: Arc<T>) -> Self
    where
        T: SceneNavigationRuntime + 'static,
    {
        Self { runtime }
    }
}

impl SceneNavigationRuntime for SceneNavigationRuntimeHandle {
    fn bake_surface(
        &self,
        world: &World,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError> {
        self.runtime.bake_surface(world, request)
    }

    fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        self.runtime.tick_world_agents(world, dt_seconds)
    }

    fn tick_world_agent(
        &self,
        world: &mut World,
        entity: u64,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        self.runtime.tick_world_agent(world, entity, dt_seconds)
    }
}

impl fmt::Debug for SceneNavigationRuntimeHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneNavigationRuntimeHandle")
            .finish_non_exhaustive()
    }
}
