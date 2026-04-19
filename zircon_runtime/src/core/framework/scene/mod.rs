use crate::core::CoreError;

mod entity_path;
mod level_summary;
mod mobility;
mod world_handle;

pub type EntityId = u64;
pub type NodeId = EntityId;

pub use entity_path::{ComponentPropertyPath, EntityPath, PathParseError};
pub use level_summary::LevelSummary;
pub use mobility::Mobility;
pub use world_handle::WorldHandle;

pub trait LevelManager: Send + Sync {
    fn create_default_level_handle(&self) -> WorldHandle;
    fn level_exists(&self, handle: WorldHandle) -> bool;
    fn level_summary(&self, handle: WorldHandle) -> Option<LevelSummary>;
    fn load_level_asset(&self, project_root: &str, uri: &str) -> Result<WorldHandle, CoreError>;
    fn save_level_asset(
        &self,
        handle: WorldHandle,
        project_root: &str,
        uri: &str,
    ) -> Result<(), CoreError>;
}
