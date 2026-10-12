#[path = "loader.rs"]
mod loader;
#[path = "prepared.rs"]
mod prepared;
#[path = "task.rs"]
mod task;

pub use prepared::PreparedDynamicSceneSpawn;
pub(crate) use prepared::{DynamicSceneSpawnTargetSnapshot, StagedDynamicSceneSpawn};
pub use task::DynamicSceneSpawnTask;

type SpawnTaskResult = Result<PreparedDynamicSceneSpawn, super::DynamicSceneError>;
