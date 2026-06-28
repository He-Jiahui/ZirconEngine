use std::path::PathBuf;
use std::str::Utf8Error;

use crate::asset::AssetImportError;
use crate::core::framework::navigation::NavigationError;
use crate::core::framework::render::RenderFrameworkError;
use crate::core::CoreError;
use crate::plugin::RuntimeExtensionRegistryError;
use crate::script::VmError;
use thiserror::Error;

pub(super) type RuntimeDynamicSessionResult<T> = Result<T, RuntimeDynamicSessionError>;
pub(super) type RuntimeProjectResult<T> = Result<T, RuntimeProjectError>;

#[derive(Debug, Error)]
pub(super) enum RuntimeDynamicSessionError {
    #[error("runtime module discovery failed: {message}")]
    ModuleDiscovery { message: String },
    #[error("{step}: {source}")]
    CoreStep {
        step: &'static str,
        #[source]
        source: CoreError,
    },
    #[error("{step}: {source}")]
    ProjectStep {
        step: &'static str,
        #[source]
        source: RuntimeProjectError,
    },
    #[error("{step}: {source}")]
    RenderBridgeStep {
        step: &'static str,
        #[source]
        source: RenderFrameworkError,
    },
    #[error("{step}: {source}")]
    RuntimeExtensionRegistryStep {
        step: &'static str,
        #[source]
        source: RuntimeExtensionRegistryError,
    },
    #[error("register scene runtime hook `{hook}`: {source}")]
    RegisterSceneRuntimeHook {
        hook: String,
        #[source]
        source: RuntimeExtensionRegistryError,
    },
    #[error("encode accessibility tree: {source}")]
    EncodeAccessibilityTree {
        #[source]
        source: serde_json::Error,
    },
}

#[derive(Debug, Error)]
pub(super) enum RuntimeProjectError {
    #[error("runtime project root must be UTF-8: {source}")]
    ProjectRootUtf8 {
        #[source]
        source: Utf8Error,
    },
    #[error("runtime project root cannot be empty")]
    EmptyProjectRoot,
    #[error("failed to open runtime project {}: {source}", root.display())]
    OpenProject {
        root: PathBuf,
        #[source]
        source: AssetImportError,
    },
    #[error(
        "runtime project {} requires AssetManager but it is unavailable: {source}",
        root.display()
    )]
    ResolveAssetManager {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error("failed to open runtime project assets {}: {source}", root.display())]
    OpenProjectAssets {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error(
        "failed to load default scene {scene} from project {}: {source}",
        root.display()
    )]
    LoadDefaultScene {
        root: PathBuf,
        scene: String,
        #[source]
        source: CoreError,
    },
    #[error(
        "runtime project {} requires ProjectAssetManager for scene asset reloads but it is unavailable: {source}",
        root.display()
    )]
    ResolveProjectAssetManager {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error(
        "runtime project {} has no active ProjectManager for scene asset reloads",
        root.display()
    )]
    MissingActiveProjectManager { root: PathBuf },
    #[error("failed to read runtime navmesh {}: {source}", path.display())]
    ReadNavmesh {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse runtime navmesh {}: {source}", path.display())]
    ParseNavmesh {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error(
        "runtime project {} declares a navmesh but NavigationManager is unavailable: {source}",
        root.display()
    )]
    ResolveNavigationManager {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error("failed to load runtime navmesh {}: {source}", path.display())]
    LoadNavmesh {
        path: PathBuf,
        #[source]
        source: NavigationError,
    },
    #[error(
        "runtime project {} declares scripts but ScriptModule is unavailable: {source}",
        root.display()
    )]
    ResolveScriptManager {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error(
        "failed to discover runtime script packages under {}: {source}",
        root.display()
    )]
    DiscoverScriptPackages {
        root: PathBuf,
        #[source]
        source: VmError,
    },
    #[error("failed to load runtime script package {package}: {source}")]
    LoadScriptPackage {
        package: String,
        #[source]
        source: VmError,
    },
    #[error("runtime startup script package {package} was not found")]
    MissingStartupScriptPackage { package: String },
}
