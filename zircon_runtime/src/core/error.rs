//! Core error types.

use crate::core::lifecycle::ServiceKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZirconError {
    #[error("channel send failed: {0}")]
    ChannelSend(String),
    #[error("thread spawn failed: {0}")]
    ThreadSpawn(String),
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid registry name: {0}")]
    InvalidRegistryName(String),
    #[error("module already registered: {0}")]
    DuplicateModule(String),
    #[error("module not found: {0}")]
    MissingModule(String),
    #[error("service already registered: {0}")]
    DuplicateService(String),
    #[error("service not found: {0}")]
    MissingService(String),
    #[error("service kind mismatch for {name}: expected {expected:?}, found {actual:?}")]
    ServiceKindMismatch {
        name: String,
        expected: ServiceKind,
        actual: ServiceKind,
    },
    #[error("cyclic dependency detected while resolving {0}")]
    DependencyCycle(String),
    #[error("service initialization failed for {0}: {1}")]
    Initialization(String, String),
    #[error("service unload blocked for {0}; still referenced by {1:?}")]
    UnloadBlocked(String, Vec<String>),
    #[error("service downcast failed for {0}")]
    ServiceDowncast(String),
    #[error("config missing: {0}")]
    MissingConfig(String),
    #[error("config parse failed for {0}: {1}")]
    ConfigParse(String, String),
}
