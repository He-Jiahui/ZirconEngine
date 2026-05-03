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
    #[error("hybrid GI runtime provider {0} already registered")]
    DuplicateHybridGiRuntimeProvider(String),
    #[error("component type {0} already registered")]
    DuplicateComponentType(String),
    #[error("invalid component type: {0}")]
    InvalidComponentType(String),
    #[error("ui component {0} already registered")]
    DuplicateUiComponent(String),
    #[error("invalid ui component: {0}")]
    InvalidUiComponent(String),
    #[error("plugin option {0} already registered")]
    DuplicatePluginOption(String),
    #[error("invalid plugin option: {0}")]
    InvalidPluginOption(String),
    #[error("plugin event catalog {0} already registered")]
    DuplicatePluginEventCatalog(String),
    #[error("invalid plugin event catalog: {0}")]
    InvalidPluginEventCatalog(String),
    #[error("asset importer registration failed: {0}")]
    AssetImporter(String),
    #[error("scene hook {0} already registered")]
    DuplicateSceneHook(String),
    #[error("invalid scene hook: {0}")]
    InvalidSceneHook(String),
}
