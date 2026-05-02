use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RuntimeExtensionRegistryError {
    #[error("manager {0} already registered")]
    DuplicateManager(String),
    #[error("module {0} already registered")]
    DuplicateModule(String),
    #[error("render feature {0} already registered")]
    DuplicateRenderFeature(String),
    #[error("render pass executor {0} already registered")]
    DuplicateRenderPassExecutor(String),
    #[error("virtual geometry runtime provider {0} already registered")]
    DuplicateVirtualGeometryRuntimeProvider(String),
    #[error("component type {0} already registered")]
    DuplicateComponentType(String),
    #[error("invalid component type: {0}")]
    InvalidComponentType(String),
    #[error("ui component {0} already registered")]
    DuplicateUiComponent(String),
    #[error("invalid ui component: {0}")]
    InvalidUiComponent(String),
}
