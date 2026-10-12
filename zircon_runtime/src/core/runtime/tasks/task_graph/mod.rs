//! Explicitly owned task execution for runtime and module lifecycle work.

#[path = "admission.rs"]
mod admission;
#[path = "engine_task_graph.rs"]
pub(super) mod engine_task_graph;
#[path = "lease.rs"]
mod lease;
#[path = "options.rs"]
mod options;
#[path = "scope.rs"]
mod scope;
#[path = "scope_model.rs"]
mod scope_model;
#[path = "scope_registration.rs"]
mod scope_registration;
#[path = "shutdown.rs"]
mod shutdown;
#[path = "task_handle.rs"]
mod task_handle;
#[path = "worker_inventory.rs"]
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
pub use worker_inventory::{
    TaskGraphWorkerDomainInventory, TaskGraphWorkerInventory, TaskGraphWorkerShutdownCensus,
};
