use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RuntimeExtensionRegistryError {
    #[error("manager {0} already registered")]
    DuplicateManager(String),
    #[error("invalid manager contribution: {0}")]
    InvalidManager(String),
    #[error("module {0} already registered")]
    DuplicateModule(String),
    #[error("invalid module contribution: {0}")]
    InvalidModule(String),
    #[error("render feature {0} already registered")]
    DuplicateRenderFeature(String),
    #[error("render pass executor {0} already registered")]
    DuplicateRenderPassExecutor(String),
    #[error("runtime prepare collector {0} already registered")]
    DuplicateRuntimePrepareCollector(String),
    #[error("virtual geometry runtime provider {0} already registered")]
    DuplicateVirtualGeometryRuntimeProvider(String),
    #[error("hybrid GI runtime provider {0} already registered")]
    DuplicateHybridGiRuntimeProvider(String),
    #[error("Solari runtime provider {0} already registered")]
    DuplicateSolariRuntimeProvider(String),
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
    #[error("invalid plugin module: {0}")]
    InvalidPluginModule(String),
    #[error("plugin system {0} already registered")]
    DuplicatePluginSystem(String),
    #[error("invalid plugin system: {0}")]
    InvalidPluginSystem(String),
    #[error("plugin resource {0} already registered")]
    DuplicatePluginResource(String),
    #[error("plugin event {0} already registered")]
    DuplicatePluginEvent(String),
    #[error("plugin interface {0} already registered")]
    DuplicatePluginInterface(String),
    #[error("plugin interface {0} is not enabled")]
    MissingPluginInterface(String),
    #[error("plugin world registration failed: {0}")]
    WorldRegistration(String),
}
