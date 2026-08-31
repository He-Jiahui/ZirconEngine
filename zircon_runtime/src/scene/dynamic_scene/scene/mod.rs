//! Dynamic scene snapshot data and world operations.

mod capture;
mod snapshot;
mod spawn;
mod validation;
mod world_operations;

pub use snapshot::DynamicScene;
pub(crate) use spawn::{CompiledSceneSpawn, PreflightedSceneMutation};
