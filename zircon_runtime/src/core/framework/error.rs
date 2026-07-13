//! Core error types.

use crate::core::ServiceKind;
use thiserror::Error;

use std::time::Duration;

#[derive(Debug, Error)]
pub enum ZirconError {
    #[error("channel send failed: {0}")]
    ChannelSend(String),
    #[error("thread spawn failed: {0}")]
    ThreadSpawn(String),
}

pub type CoreResult<T> = std::result::Result<T, CoreError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
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
    #[error("module dependency missing for {module}: {dependency}")]
    MissingModuleDependency { module: String, dependency: String },
    #[error(
        "module init-level violation for {module} ({module_level}): dependency {dependency} is later at {dependency_level}"
    )]
    ModuleInitLevelViolation {
        module: String,
        module_level: String,
        dependency: String,
        dependency_level: String,
    },
    #[error("module dependency cycle detected: {path:?}")]
    ModuleDependencyCycle { path: Vec<String> },
    #[error("module ready timeout for {module} after {budget:?}")]
    ModuleReadyTimeout { module: String, budget: Duration },
    #[error("module activation failed: {activation}; cleanup also failed: {cleanup}")]
    ModuleActivationRollback {
        activation: Box<CoreError>,
        cleanup: Box<CoreError>,
    },
    #[error(
        "module batch activation failed: {activation}; cleanup failures: {cleanup_failures:?}"
    )]
    ModuleBatchActivationRollback {
        activation: Box<CoreError>,
        cleanup_failures: Vec<(String, CoreError)>,
    },
    #[error("service initialization failed for {0}: {1}")]
    Initialization(String, String),
    #[error("runtime is no longer available")]
    RuntimeUnavailable,
    #[error("service unload blocked for {0}; still referenced by {1:?}")]
    UnloadBlocked(String, Vec<String>),
    #[error("runtime module lifecycle observer blocked deactivation: {0}")]
    RuntimeModuleLifecycleBlocked(String),
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

impl CoreError {
    pub(crate) fn module_activation_failed(activation: Self, cleanup: Option<Self>) -> Self {
        match cleanup {
            Some(cleanup) => Self::ModuleActivationRollback {
                activation: Box::new(activation),
                cleanup: Box::new(cleanup),
            },
            None => activation,
        }
    }

    pub(crate) fn module_batch_activation_failed(
        activation: Self,
        cleanup_failures: Vec<(String, Self)>,
    ) -> Self {
        if cleanup_failures.is_empty() {
            activation
        } else {
            Self::ModuleBatchActivationRollback {
                activation: Box::new(activation),
                cleanup_failures,
            }
        }
    }
}
