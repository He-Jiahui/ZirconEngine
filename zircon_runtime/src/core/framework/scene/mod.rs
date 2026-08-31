use std::fmt;
use std::sync::Arc;
use std::time::Instant;

mod component_type_descriptor;
mod entity_path;
mod level_manager_error;
mod level_summary;
mod mobility;
mod module_identity;
pub mod physics;
mod property_value;
mod resource;
mod system_stage;
mod world_handle;

pub type EntityId = u64;
pub type NodeId = EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneArtifactTerminal {
    Succeeded,
    Failed { code: &'static str },
    DeadlineBeforeStart,
    CancelledBeforeStart,
    Superseded { successor: u64 },
    Shutdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneArtifactWaitResult {
    Terminal(SceneArtifactTerminal),
    ObserverTimedOut,
}

pub trait SceneArtifactTicket: Send + Sync + fmt::Debug + 'static {
    fn generation(&self) -> u64;
    fn terminal(&self) -> Option<SceneArtifactTerminal>;
    fn wait_until(&self, deadline: Instant) -> SceneArtifactWaitResult;
}

pub use component_type_descriptor::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
pub use entity_path::{ComponentPropertyPath, EntityPath, PathParseError};
pub use level_manager_error::LevelManagerError;
pub use level_summary::LevelSummary;
pub use mobility::Mobility;
pub use module_identity::SCENE_MODULE_NAME;
pub(crate) use property_value::ScenePropertyEntry;
pub use property_value::ScenePropertyValue;
pub use resource::SceneResource;
pub use system_stage::SystemStage;
pub use world_handle::WorldHandle;

pub trait LevelManager: Send + Sync {
    fn create_default_level_handle(&self) -> Result<WorldHandle, LevelManagerError>;
    fn level_exists(&self, handle: WorldHandle) -> bool;
    fn level_summary(&self, handle: WorldHandle) -> Option<LevelSummary>;
    fn load_level_asset(
        &self,
        project_root: &str,
        uri: &str,
    ) -> Result<WorldHandle, LevelManagerError>;
    fn save_level_asset(
        &self,
        handle: WorldHandle,
        project_root: &str,
        uri: &str,
    ) -> Result<Arc<dyn SceneArtifactTicket>, LevelManagerError>;
}
