use std::path::{Path, PathBuf};
use std::str::Utf8Error;

use zircon_runtime_interface::project::{RelPath, RelPathError};

use crate::asset::AssetImportError;
use crate::asset::project::ProjectPaths;
use crate::core::framework::navigation::NavigationError;
use crate::core::framework::render::RenderFrameworkError;
use crate::core::framework::time::ProductTimePolicyError;
use crate::core::{CoreError, EngineTaskGraphInitError, TaskGraphAdmissionError};
use crate::operation::RuntimeOperationServiceError;
use crate::plugin::RuntimeExtensionRegistryError;
use crate::scene::{DynamicSceneError, LevelTickError};
use crate::script::VmError;
use thiserror::Error;
use zircon_runtime_interface::ui::tree::UiTreeError;

pub(super) type RuntimeDynamicSessionResult<T> = Result<T, RuntimeDynamicSessionError>;
pub(super) type RuntimeProjectResult<T> = Result<T, RuntimeProjectError>;

#[derive(Debug, Error)]
pub enum RuntimeDynamicSessionError {
    #[error("unknown runtime session profile `{profile}`")]
    UnknownProfile { profile: String },
    #[error("runtime session handle space exhausted")]
    SessionHandleSpaceExhausted,
    #[error("initialize runtime execution: {source}")]
    EngineTaskGraphInitialization {
        #[source]
        source: EngineTaskGraphInitError,
    },
    #[error("create runtime session execution scope: {source}")]
    TaskGraphScopeAdmission {
        #[source]
        source: TaskGraphAdmissionError,
    },
    #[error("runtime module discovery failed: {message}")]
    ModuleDiscovery { message: String },
    #[error("{step}: {source}")]
    CoreStep {
        step: &'static str,
        #[source]
        source: CoreError,
    },
    #[error("tick loaded level: {source}")]
    LevelTick {
        #[source]
        source: LevelTickError,
    },
    #[error("{step}: {source}")]
    ProductTimePolicy {
        step: &'static str,
        #[source]
        source: ProductTimePolicyError,
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
    #[error("encode accessibility tree: {source}")]
    EncodeAccessibilityTree {
        #[source]
        source: serde_json::Error,
    },
    #[error("register runtime operation handlers: {source}")]
    RuntimeOperationRegistry {
        #[source]
        source: RuntimeOperationServiceError,
    },
    #[error("rebuild declared runtime UI surface: {source}")]
    RuntimeUiLayout {
        #[source]
        source: UiTreeError,
    },
}

#[derive(Debug, Error)]
pub enum RuntimeProjectError {
    #[error("runtime project root must be UTF-8: {source}")]
    ProjectRootUtf8 {
        #[source]
        source: Utf8Error,
    },
    #[error("runtime project root cannot be empty")]
    EmptyProjectRoot,
    #[error("runtime Play scene requires a project root")]
    PlaySceneRequiresProject,
    #[error("runtime Play report outlet requires a project root")]
    PlayReportPipeRequiresProject,
    #[error("runtime Play scene must be UTF-8: {source}")]
    PlaySceneUtf8 {
        #[source]
        source: Utf8Error,
    },
    #[error("runtime Play scene cannot be empty")]
    EmptyPlayScene,
    #[error("runtime Play scene {scene:?} is not project-relative: {source}")]
    InvalidPlayScene {
        scene: String,
        #[source]
        source: RelPathError,
    },
    #[error(
        "runtime Play scene {scene} must use .zrscene.json for a versioned snapshot or .scene.toml for a project scene asset"
    )]
    UnsupportedPlaySceneFormat { scene: RelPath },
    #[error("runtime Play report outlet must be UTF-8: {source}")]
    PlayReportPipeUtf8 {
        #[source]
        source: Utf8Error,
    },
    #[error("runtime Play report outlet cannot be empty")]
    EmptyPlayReportPipe,
    #[error(
        "failed to resolve runtime project root {root_path}: {source}",
        root_path = display_project_path(root)
    )]
    ResolveProjectRoot {
        root: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "failed to resolve runtime Play scene {scene} under project {root_path}: {source}",
        root_path = display_project_path(root)
    )]
    ResolvePlayScene {
        root: PathBuf,
        scene: RelPath,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to map runtime Play scene {scene} to a project asset URI: {source}")]
    ResolvePlaySceneAssetUri {
        scene: RelPath,
        #[source]
        source: AssetImportError,
    },
    #[error(
        "failed to open runtime project {root_path}: {source}",
        root_path = display_project_path(root)
    )]
    OpenProject {
        root: PathBuf,
        #[source]
        source: AssetImportError,
    },
    #[error(
        "runtime project {root_path} requires AssetManager but it is unavailable: {source}",
        root_path = display_project_path(root)
    )]
    ResolveAssetManager {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error(
        "failed to open runtime project assets {root_path}: {source}",
        root_path = display_project_path(root)
    )]
    OpenProjectAssets {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error(
        "runtime project {root_path} already transferred its prepared ProjectManager to AssetModule",
        root_path = display_project_path(root)
    )]
    PreparedProjectManagerTransferred { root: PathBuf },
    #[error(
        "failed to load default scene {scene} from project {root_path}: {source}",
        root_path = display_project_path(root)
    )]
    LoadDefaultScene {
        root: PathBuf,
        scene: String,
        #[source]
        source: CoreError,
    },
    #[error("failed to load runtime Play scene asset {scene} from project {root_path}: {source}", root_path = display_project_path(root))]
    LoadPlaySceneAsset {
        root: PathBuf,
        scene: RelPath,
        #[source]
        source: CoreError,
    },
    #[error("runtime Play scene override was not prepared")]
    MissingPlaySceneOverride,
    #[error("failed to read runtime Play scene {path_display}: {source}", path_display = display_project_path(path))]
    ReadPlayScene {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse versioned runtime Play scene {path_display}: {source}", path_display = display_project_path(path))]
    ParsePlayScene {
        path: PathBuf,
        #[source]
        source: DynamicSceneError,
    },
    #[error("failed to create runtime Play level from {scene}: {source}")]
    CreatePlaySceneLevel {
        scene: RelPath,
        #[source]
        source: CoreError,
    },
    #[error("failed to apply runtime Play scene {scene}: {source}")]
    ApplyPlayScene {
        scene: RelPath,
        #[source]
        source: DynamicSceneError,
    },
    #[error(
        "runtime project {root_path} requires ProjectAssetManager for scene asset reloads but it is unavailable: {source}",
        root_path = display_project_path(root)
    )]
    ResolveProjectAssetManager {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error(
        "runtime project {root_path} has no active ProjectManager for scene asset reloads",
        root_path = display_project_path(root)
    )]
    MissingActiveProjectManager { root: PathBuf },
    #[error("failed to load declared runtime UI root {root}: {source}")]
    LoadRuntimeUiRoot {
        root: String,
        #[source]
        source: AssetImportError,
    },
    #[error("declared runtime UI root {root} is invalid: {detail}")]
    BuildRuntimeUiRoot { root: String, detail: String },
    #[error(
        "failed to read runtime navmesh {path_display}: {source}",
        path_display = display_project_path(path)
    )]
    ReadNavmesh {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "failed to parse runtime navmesh {path_display}: {source}",
        path_display = display_project_path(path)
    )]
    ParseNavmesh {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error(
        "runtime project {root_path} declares a navmesh but NavigationManager is unavailable: {source}",
        root_path = display_project_path(root)
    )]
    ResolveNavigationManager {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error(
        "failed to load runtime navmesh {path_display}: {source}",
        path_display = display_project_path(path)
    )]
    LoadNavmesh {
        path: PathBuf,
        #[source]
        source: NavigationError,
    },
    #[error(
        "runtime project {root_path} declares scripts but ScriptModule is unavailable: {source}",
        root_path = display_project_path(root)
    )]
    ResolveScriptManager {
        root: PathBuf,
        #[source]
        source: CoreError,
    },
    #[error(
        "failed to discover runtime script packages under {root_path}: {source}",
        root_path = display_project_path(root)
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

fn display_project_path(path: &Path) -> String {
    ProjectPaths::display_path(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::RuntimeProjectError;

    #[cfg(windows)]
    #[test]
    fn runtime_project_error_displays_windows_operation_roots_without_verbatim_prefixes() {
        let error = RuntimeProjectError::PreparedProjectManagerTransferred {
            root: PathBuf::from(r"\\?\C:\ZirconBuilds\stage\project"),
        };

        assert_eq!(
            error.to_string(),
            "runtime project C:\\ZirconBuilds\\stage\\project already transferred its prepared ProjectManager to AssetModule"
        );
    }
}
