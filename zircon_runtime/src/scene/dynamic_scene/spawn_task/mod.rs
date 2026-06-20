mod loader;
mod prepared;
mod task;

pub use prepared::PreparedDynamicSceneSpawn;
pub use task::DynamicSceneSpawnTask;

type SpawnTaskResult = Result<PreparedDynamicSceneSpawn, super::DynamicSceneError>;
