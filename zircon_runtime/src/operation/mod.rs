//! Runtime-owned asynchronous operation registry and task lifecycle.

mod context;
mod error;
mod handler;
mod service;
mod task;

pub use context::RuntimeOperationContext;
pub use error::{RuntimeOperationHandlerError, RuntimeOperationServiceError};
pub use handler::RuntimeOperationHandler;
pub use service::RuntimeOperationService;

#[cfg(test)]
mod tests;
