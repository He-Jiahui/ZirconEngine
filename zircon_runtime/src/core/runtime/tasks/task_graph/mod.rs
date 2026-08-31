//! Explicitly owned task execution for runtime and module lifecycle work.

mod admission;
mod engine_task_graph;
mod lease;
mod options;
mod scope;
mod scope_model;
mod scope_registration;
mod shutdown;
mod task_handle;
mod worker_inventory;

pub use admission::TaskGraphAdmissionError;
pub use engine_task_graph::EngineTaskGraph;
pub use options::{EngineTaskGraphInitError, EngineTaskGraphOptions};
pub use scope::{TaskCancellationToken, TaskGraphScope};
pub use scope_model::{
    TaskGraphScopeCensus, TaskGraphScopeDescriptor, DEFAULT_TASK_GRAPH_SCOPE_TASK_CAPACITY,
};
pub use shutdown::{TaskGraphShutdownError, TaskGraphShutdownReport};
pub use task_handle::TaskHandle;
pub use worker_inventory::{TaskGraphWorkerInventory, TaskGraphWorkerShutdownCensus};
