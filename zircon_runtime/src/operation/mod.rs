//! Runtime-owned asynchronous operation registry and task lifecycle.

mod context;
mod error;
mod handler;
mod maintenance;
mod service;
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
