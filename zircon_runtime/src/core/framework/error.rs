//! Core error types.

use crate::core::ServiceKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZirconError {
    #[error("channel send failed: {0}")]
    ChannelSend(String),
    #[error("thread spawn failed: {0}")]
    ThreadSpawn(String),
}

pub type CoreResult<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid registry name: {0}")]
    InvalidRegistryName(String),
    #[error("invalid module name: {0}")]
    InvalidModuleName(String),
    #[error("module already registered: {0}")]
    DuplicateModule(String),
    #[error("module not found: {0}")]
    MissingModule(String),
    #[error("service already registered: {0}")]
    DuplicateService(String),
    #[error("service not found: {0}")]
    MissingService(String),
    #[error("service owner mismatch for {name}: expected module {expected}, found {actual}")]
    ServiceOwnerMismatch {
        name: String,
        expected: String,
        actual: String,
    },
    #[error("service kind mismatch for {name}: expected {expected:?}, found {actual:?}")]
    ServiceKindMismatch {
        name: String,
        expected: ServiceKind,
        actual: ServiceKind,
    },
    #[error(
        "invalid service dependency for {service}: {service_kind:?} cannot depend on {dependency} ({dependency_kind:?})"
    )]
    InvalidServiceDependencyKind {
        service: String,
        service_kind: ServiceKind,
        dependency: String,
        dependency_kind: ServiceKind,
    },
    #[error("cyclic dependency detected while resolving {0}")]
    DependencyCycle(String),
    #[error("service initialization failed for {0}: {1}")]
    Initialization(String, String),
    #[error("service unload blocked for {0}; still referenced by {1:?}")]
    UnloadBlocked(String, Vec<String>),
    #[error("plugin bridge lifecycle blocked: {0}")]
    PluginBridgeLifecycleBlocked(String),
    #[error("service downcast failed for {0}")]
    ServiceDowncast(String),
    #[error("missing resource record for locator {locator}")]
    MissingResourceRecordForLocator { locator: String },
    #[error("missing resource record for id {id}")]
    MissingResourceRecordForId { id: String },
    #[error("config missing: {0}")]
    MissingConfig(String),
    #[error("config parse failed for {0}: {1}")]
    ConfigParse(String, String),
}
