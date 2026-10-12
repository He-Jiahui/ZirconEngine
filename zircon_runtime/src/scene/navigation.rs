use crate::core::runtime::{ServiceAliasSource, ServiceCallGuard};
use crate::core::{CoreError, CoreHandle, ServiceKind};
use std::any::Any;
use std::fmt;

use crate::core::framework::navigation::{
    NavAgentTickReport, NavMeshBakeReport, NavMeshBakeRequest, NavigationError,
    NavigationErrorKind, NavigationGeneratedBakeSnapshot,
};
use crate::core::math::Real;
use crate::scene::{LevelSystem, World};

pub const SCENE_NAVIGATION_RUNTIME_DRIVER_NAME: &str =
    "navigation.runtime.Driver.SceneNavigationRuntime";

/// 场景对导航服务的运行时依赖；World/Level 仅提交烘焙与代理步进请求，网格缓存与寻路算法由实现方拥有。
pub trait SceneNavigationRuntime: Send + Sync {
    fn bake_surface(
        &self,
        level: &LevelSystem,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError>;

    fn generated_bake_snapshot(
        &self,
        surface_entity: Option<u64>,
    ) -> NavigationGeneratedBakeSnapshot;

    fn replace_generated_bake_snapshot(
        &self,
        snapshot: NavigationGeneratedBakeSnapshot,
    ) -> Result<(), NavigationError>;

    /// Returns the common mutation epoch for generated bake state. Every clear,
    /// restore, and replacement increments this epoch so equal snapshots cannot
    /// admit an older owner operation after an A→empty→A transition.
    fn generated_bake_mutation_epoch(&self, surface_entity: Option<u64>) -> u64 {
        let _ = surface_entity;
        0
    }

    /// Captures bounded, immutable bake input while the owner still has the live level.
    /// The returned opaque value is moved to the worker; it must not retain the World mutex.
    fn capture_bake_operation(
        &self,
        source: &crate::scene::WorldPublicationSource,
        request: NavMeshBakeRequest,
    ) -> Result<(Box<dyn Any + Send>, usize, NavigationGeneratedBakeSnapshot), NavigationError>
    {
        let _ = (source, request);
        Err(NavigationError::new(
            NavigationErrorKind::BackendFailure,
            "navigation runtime does not provide an operation bake backend",
        ))
    }

    /// Runs backend preparation from the immutable operation snapshot.
    fn prepare_bake_operation(
        &self,
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
        let _ = snapshot;
        Err(NavigationError::new(
            NavigationErrorKind::BackendFailure,
            "navigation runtime does not provide an operation bake backend",
        ))
    }

    /// Publishes an already prepared bake while the owner publication guard is held.
    fn apply_bake_operation(
        &self,
        source: &crate::scene::WorldPublicationSource,
        expected_before: &NavigationGeneratedBakeSnapshot,
        prepared: Box<dyn Any + Send>,
    ) -> Result<(), NavigationError> {
        let _ = (source, expected_before, prepared);
        Err(NavigationError::new(
            NavigationErrorKind::BackendFailure,
            "navigation runtime does not provide an operation bake backend",
        ))
    }

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

/// An idle, generation-bound navigation dependency. Each operation must enter
/// the original physical driver and the declared scene view together.
#[derive(Clone)]
pub struct SceneNavigationRuntimeHandle {
    runtime: ServiceAliasSource<dyn SceneNavigationRuntime>,
}

impl SceneNavigationRuntimeHandle {
    pub fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        core.resolve_alias_source(SCENE_NAVIGATION_RUNTIME_DRIVER_NAME, ServiceKind::Driver)
            .map(|runtime| Self { runtime })
    }

    /// Rejects closed or replaced registrations without a default backend result.
    pub fn enter(&self) -> Result<ServiceCallGuard<dyn SceneNavigationRuntime>, CoreError> {
        self.runtime.enter()
    }
}

impl fmt::Debug for SceneNavigationRuntimeHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneNavigationRuntimeHandle")
            .finish_non_exhaustive()
    }
}
