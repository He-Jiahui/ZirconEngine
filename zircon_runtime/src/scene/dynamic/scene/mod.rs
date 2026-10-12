//! Dynamic scene snapshot data and world operations.

#[path = "capture.rs"]
mod capture;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "spawn/mod.rs"]
mod spawn;
#[path = "validation.rs"]
mod validation;
#[path = "world_operations.rs"]
mod world_operations;

pub use snapshot::DynamicScene;
pub(crate) use spawn::{CompiledSceneSpawn, PreflightedSceneMutation};
