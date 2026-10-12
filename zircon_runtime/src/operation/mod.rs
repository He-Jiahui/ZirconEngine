//! Runtime-owned asynchronous operation registry and task lifecycle.

#[path = "context.rs"]
mod context;
#[path = "error.rs"]
mod error;
#[path = "handler.rs"]
mod handler;
#[path = "maintenance.rs"]
mod maintenance;
#[path = "service.rs"]
mod service;
#[path = "task.rs"]
mod task;

pub use context::RuntimeOperationContext;
pub use error::{RuntimeOperationHandlerError, RuntimeOperationServiceError};
pub use handler::{
    RuntimeOperationApply, RuntimeOperationHandler, RuntimeOperationPrepared,
    RuntimeOperationSnapshot,
};
pub use service::RuntimeOperationService;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
