use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RuntimeExtensionRegistryError {
    #[error("module {0} already registered")]
    DuplicateModule(String),
    #[error("render feature {0} already registered")]
    DuplicateRenderFeature(String),
    #[error("component type {0} already registered")]
    DuplicateComponentType(String),
    #[error("ui component {0} already registered")]
    DuplicateUiComponent(String),
}
