mod loader;
mod prepared;
mod task;

pub use prepared::PreparedDynamicSceneSpawn;
pub(crate) use prepared::{DynamicSceneSpawnTargetSnapshot, StagedDynamicSceneSpawn};
pub use task::DynamicSceneSpawnTask;

type SpawnTaskResult = Result<PreparedDynamicSceneSpawn, super::DynamicSceneError>;
