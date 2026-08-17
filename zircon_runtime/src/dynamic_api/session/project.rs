use std::collections::HashSet;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::{ZrByteSlice, ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1};

use crate::asset::project::{
    ProjectManager, ProjectManifest, ProjectPaths, ProjectScriptManifest, ResolvedProjectPath,
};
use crate::asset::{asset_manager_handle, project_asset_manager_handle, ProjectInfo};
use crate::core::framework::navigation::NavMeshAsset;
use crate::core::framework::project::ProjectPluginManifest;
use crate::core::manager::{navigation_manager_handle, resolve_manager_service};
use crate::core::CoreHandle;
use crate::diagnostic_log::{write_log, write_log_lazy};
use crate::scene::{DynamicScene, DynamicSceneAssetReloadQueue, LevelMetadata, LevelSystem, World};
use crate::script::{VmPluginManager, VM_PLUGIN_MANAGER_NAME};

use super::error::{RuntimeProjectError, RuntimeProjectResult};
use super::runtime_ui::RuntimeUiSurfaceSet;

const DEFAULT_PROJECT_NAVMESH_PATH: &[&str] = &["assets", "navigation", "main.navmesh.toml"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeProjectConfig {
    root: ResolvedProjectPath,
    play_scene: Option<RelPath>,
}

impl RuntimeProjectConfig {
    /// Resolves the caller-owned project path once at the dynamic-runtime boundary.
    ///
    /// The configuration retains both resolver views so project preparation can open the same
    /// physical root without recreating a platform-specific path branch below the ABI.
    pub(super) fn from_root(root: impl AsRef<Path>) -> RuntimeProjectResult<Self> {
        let requested_root = root.as_ref();
        let root = ProjectPaths::resolve_path(requested_root).map_err(|source| {
            RuntimeProjectError::ResolveProjectRoot {
                root: requested_root.to_path_buf(),
                source,
            }
        })?;
        let root = if ProjectPaths::is_project_manifest_file(root.operation_path()) {
            root.parent()
                .ok_or_else(|| RuntimeProjectError::ResolveProjectRoot {
                    root: requested_root.to_path_buf(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "project manifest has no parent directory",
                    ),
                })?
        } else {
            root
        };
        Ok(Self {
            root,
            play_scene: None,
        })
    }

    pub(super) fn from_abi_slice(slice: ZrByteSlice) -> RuntimeProjectResult<Option<Self>> {
        if slice.is_empty() {
            return Ok(None);
        }
        let bytes = unsafe { slice.checked_slice(ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1) }
            .map_err(|error| RuntimeProjectError::ResolveProjectRoot {
                root: PathBuf::from("<invalid ABI project root>"),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("invalid runtime project root byte slice: {error:?}"),
                ),
            })?;
        let value = std::str::from_utf8(bytes)
            .map_err(|source| RuntimeProjectError::ProjectRootUtf8 { source })?
            .trim();
        if value.is_empty() {
            return Err(RuntimeProjectError::EmptyProjectRoot);
        }
        Self::from_root(Path::new(value)).map(Some)
    }

    pub(super) fn from_abi_startup_config(
        project_root: ZrByteSlice,
        play_scene: ZrByteSlice,
        play_report_pipe: ZrByteSlice,
    ) -> RuntimeProjectResult<Option<Self>> {
        for slice in [project_root, play_scene, play_report_pipe] {
            unsafe { slice.checked_slice(ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1) }.map_err(
                |error| RuntimeProjectError::ResolveProjectRoot {
                    root: PathBuf::from("<invalid ABI startup path>"),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("invalid runtime startup path byte slice: {error:?}"),
                    ),
                },
            )?;
        }
        let Some(mut project) = Self::from_abi_slice(project_root)? else {
            if !play_scene.is_empty() {
                return Err(RuntimeProjectError::PlaySceneRequiresProject);
            }
            if !play_report_pipe.is_empty() {
                return Err(RuntimeProjectError::PlayReportPipeRequiresProject);
            }
            return Ok(None);
        };

        project.play_scene = parse_optional_play_scene(play_scene)?;
        // The app process owns the child-output transport; the dynamic session only validates
        // its typed startup input before bootstrap.
        let _ = parse_optional_play_report_pipe(play_report_pipe)?;
        Ok(Some(project))
    }

    pub(super) fn root_display(&self) -> String {
        self.root.display_path().display().to_string()
    }

    pub(super) fn prepare(self) -> RuntimeProjectResult<RuntimePreparedProject> {
        let root = self.root.operation_path().to_path_buf();
        let project = ProjectManager::open_resolved(&self.root).map_err(|source| {
            RuntimeProjectError::OpenProject {
                root: root.clone(),
                source,
            }
        })?;
        let play_scene = self
            .play_scene
            .as_ref()
            .map(|relative| prepare_play_scene(&project, &root, relative))
            .transpose()?;
        let manifest = RuntimeLoadedProjectManifest::from(project.manifest());
        Ok(RuntimePreparedProject {
            root,
            manifest,
            project: Some(project),
            play_scene,
        })
    }
}

pub(super) struct RuntimePreparedProject {
    root: PathBuf,
    manifest: RuntimeLoadedProjectManifest,
    project: Option<ProjectManager>,
    play_scene: Option<PreparedPlayScene>,
}

struct PreparedPlayScene {
    relative: RelPath,
    operation_path: ResolvedProjectPath,
    kind: PreparedPlaySceneKind,
}

enum PreparedPlaySceneKind {
    VersionedSnapshot,
    SceneAsset { uri: String },
}

pub(super) fn project_opened_log(info: &ProjectInfo) -> String {
    format!(
        "runtime_project_opened root={} name={} default_scene={} library_version={} assets={} ready_assets={} failed_assets={} registry_diagnostics={}",
        ProjectPaths::display_path(Path::new(&info.root_path)).display(),
        info.name,
        info.default_scene_uri,
        info.library_version,
        info.asset_count,
        info.ready_asset_count,
        info.failed_asset_count,
        info.registry_diagnostic_count,
    )
}

impl RuntimePreparedProject {
    pub(super) fn root_display(&self) -> String {
        ProjectPaths::display_path(&self.root).display().to_string()
    }

    pub(super) fn plugin_manifest(&self) -> &ProjectPluginManifest {
        &self.manifest.plugins
    }

    pub(super) fn has_play_scene_override(&self) -> bool {
        self.play_scene.is_some()
    }

    pub(super) fn play_scene_identifier(&self) -> Option<String> {
        self.play_scene
            .as_ref()
            .map(|play_scene| play_scene.relative.to_string())
    }

    pub(super) fn open_project_assets(
        &mut self,
        core: &CoreHandle,
    ) -> RuntimeProjectResult<ProjectInfo> {
        let asset_manager = asset_manager_handle(core)
            .and_then(|handle| resolve_manager_service(core, handle))
            .map_err(|source| RuntimeProjectError::ResolveAssetManager {
                root: self.root.clone(),
                source,
            })?;
        let project = self.project.take().ok_or_else(|| {
            RuntimeProjectError::PreparedProjectManagerTransferred {
                root: self.root.clone(),
            }
        })?;
        asset_manager
            .open_prepared_project(project)
            .map_err(|source| RuntimeProjectError::OpenProjectAssets {
                root: self.root.clone(),
                source,
            })
    }

    pub(super) fn load_default_level(
        &self,
        core: &CoreHandle,
    ) -> RuntimeProjectResult<LevelSystem> {
        let asset_manager = asset_manager_handle(core)
            .and_then(|handle| resolve_manager_service(core, handle))
            .map_err(|source| RuntimeProjectError::ResolveAssetManager {
                root: self.root.clone(),
                source,
            })?;
        crate::scene::load_level_asset(
            core,
            asset_manager.as_ref(),
            self.manifest.default_scene.as_str(),
        )
        .map_err(|source| RuntimeProjectError::LoadDefaultScene {
            root: self.root.clone(),
            scene: self.manifest.default_scene.clone(),
            source,
        })
    }

    pub(super) fn load_play_scene_level(
        &self,
        core: &CoreHandle,
    ) -> RuntimeProjectResult<LevelSystem> {
        let play_scene = self
            .play_scene
            .as_ref()
            .ok_or(RuntimeProjectError::MissingPlaySceneOverride)?;
        match &play_scene.kind {
            PreparedPlaySceneKind::VersionedSnapshot => {
                let path = play_scene.operation_path.operation_path();
                let document = std::fs::read_to_string(path).map_err(|source| {
                    RuntimeProjectError::ReadPlayScene {
                        path: path.to_path_buf(),
                        source,
                    }
                })?;
                let scene = DynamicScene::from_versioned_json(&document).map_err(|source| {
                    RuntimeProjectError::ParsePlayScene {
                        path: path.to_path_buf(),
                        source,
                    }
                })?;
                let level = crate::scene::create_level(
                    core,
                    World::new(),
                    LevelMetadata {
                        project_root: Some(self.root_display()),
                        asset_uri: None,
                        display_name: Some(format!("Play snapshot {}", play_scene.relative)),
                    },
                )
                .map_err(|source| RuntimeProjectError::CreatePlaySceneLevel {
                    scene: play_scene.relative.clone(),
                    source,
                })?;
                level
                    .with_world_mut(|world| scene.spawn_into(world))
                    .map_err(|source| RuntimeProjectError::ApplyPlayScene {
                        scene: play_scene.relative.clone(),
                        source,
                    })?;
                Ok(level)
            }
            PreparedPlaySceneKind::SceneAsset { uri } => {
                let asset_manager = asset_manager_handle(core)
                    .and_then(|handle| resolve_manager_service(core, handle))
                    .map_err(|source| RuntimeProjectError::ResolveAssetManager {
                        root: self.root.clone(),
                        source,
                    })?;
                crate::scene::load_level_asset(core, asset_manager.as_ref(), uri).map_err(
                    |source| RuntimeProjectError::LoadPlaySceneAsset {
                        root: self.root.clone(),
                        scene: play_scene.relative.clone(),
                        source,
                    },
                )
            }
        }
    }

    pub(super) fn scene_asset_reload_queue(
        &self,
        core: &CoreHandle,
    ) -> RuntimeProjectResult<DynamicSceneAssetReloadQueue> {
        let asset_manager = project_asset_manager_handle(core)
            .and_then(|handle| resolve_manager_service(core, handle))
            .map_err(|source| RuntimeProjectError::ResolveProjectAssetManager {
                root: self.root.clone(),
                source,
            })?;
        let project = asset_manager.current_project_manager().ok_or_else(|| {
            RuntimeProjectError::MissingActiveProjectManager {
                root: self.root.clone(),
            }
        })?;
        Ok(DynamicSceneAssetReloadQueue::from_project_asset_manager(
            project,
            asset_manager.as_ref(),
        ))
    }

    pub(super) fn load_runtime_ui_surfaces(
        &self,
        core: &CoreHandle,
    ) -> RuntimeProjectResult<RuntimeUiSurfaceSet> {
        let asset_manager = project_asset_manager_handle(core)
            .and_then(|handle| resolve_manager_service(core, handle))
            .map_err(|source| RuntimeProjectError::ResolveProjectAssetManager {
                root: self.root.clone(),
                source,
            })?;
        let project = asset_manager.current_project_manager().ok_or_else(|| {
            RuntimeProjectError::MissingActiveProjectManager {
                root: self.root.clone(),
            }
        })?;
        RuntimeUiSurfaceSet::load(&project, &self.manifest.ui_roots)
    }

    pub(super) fn load_default_navigation(&self, core: &CoreHandle) -> RuntimeProjectResult<()> {
        let navmesh_path = DEFAULT_PROJECT_NAVMESH_PATH
            .iter()
            .fold(self.root.clone(), |path, segment| path.join(segment));
        if !navmesh_path.exists() {
            return Ok(());
        }
        let document = std::fs::read_to_string(&navmesh_path).map_err(|source| {
            RuntimeProjectError::ReadNavmesh {
                path: navmesh_path.clone(),
                source,
            }
        })?;
        let asset = toml::from_str::<NavMeshAsset>(&document).map_err(|source| {
            RuntimeProjectError::ParseNavmesh {
                path: navmesh_path.clone(),
                source,
            }
        })?;
        let navigation = navigation_manager_handle(core)
            .and_then(|handle| resolve_manager_service(core, handle))
            .map_err(|source| RuntimeProjectError::ResolveNavigationManager {
                root: self.root.clone(),
                source,
            })?;
        navigation
            .load_nav_mesh(asset)
            .map(|_| ())
            .map_err(|source| RuntimeProjectError::LoadNavmesh {
                path: navmesh_path,
                source,
            })
    }

    pub(super) fn load_startup_scripts(&self, core: &CoreHandle) -> RuntimeProjectResult<()> {
        if self.manifest.scripts.is_empty() {
            return Ok(());
        }
        let manager = core
            .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
            .map_err(|source| RuntimeProjectError::ResolveScriptManager {
                root: self.root.clone(),
                source,
            })?;
        let mut packages = Vec::new();
        for root in self.manifest.script_package_roots(&self.root) {
            write_log_lazy("runtime_session", || {
                format!(
                    "runtime_project_script_discover_start root={}",
                    ProjectPaths::display_path(&root).display()
                )
            });
            packages.extend(manager.discover_packages(&root).map_err(|source| {
                RuntimeProjectError::DiscoverScriptPackages {
                    root: root.clone(),
                    source,
                }
            })?);
            write_log_lazy("runtime_session", || {
                format!(
                    "runtime_project_script_discover_done root={} packages={}",
                    ProjectPaths::display_path(&root).display(),
                    packages.len()
                )
            });
        }
        for package in self.manifest.filter_startup_packages(packages)? {
            write_log_lazy("runtime_session", || {
                format!(
                    "runtime_project_script_load_start package={} backend={}",
                    package.package.manifest.name, package.backend_name
                )
            });
            manager
                .load_discovered_package(&package)
                .map_err(|source| RuntimeProjectError::LoadScriptPackage {
                    package: package.package.manifest.name.clone(),
                    source,
                })?;
            write_log_lazy("runtime_session", || {
                format!(
                    "runtime_project_script_load_done package={}",
                    package.package.manifest.name
                )
            });
        }
        Ok(())
    }
}

fn prepare_play_scene(
    project: &ProjectManager,
    root: &Path,
    relative: &RelPath,
) -> RuntimeProjectResult<PreparedPlayScene> {
    let operation_path =
        ProjectPaths::resolve_existing(relative.join_to(root)).map_err(|source| {
            RuntimeProjectError::ResolvePlayScene {
                root: root.to_path_buf(),
                scene: relative.clone(),
                source,
            }
        })?;
    let kind = if relative.as_str().ends_with(".zrscene.json") {
        PreparedPlaySceneKind::VersionedSnapshot
    } else if relative.as_str().ends_with(".scene.toml") {
        let uri = project
            .project_uri_for_source_path(operation_path.operation_path())
            .map_err(|source| RuntimeProjectError::ResolvePlaySceneAssetUri {
                scene: relative.clone(),
                source,
            })?;
        PreparedPlaySceneKind::SceneAsset {
            uri: uri.to_string(),
        }
    } else {
        return Err(RuntimeProjectError::UnsupportedPlaySceneFormat {
            scene: relative.clone(),
        });
    };
    Ok(PreparedPlayScene {
        relative: relative.clone(),
        operation_path,
        kind,
    })
}

fn parse_optional_play_scene(slice: ZrByteSlice) -> RuntimeProjectResult<Option<RelPath>> {
    if slice.is_empty() {
        return Ok(None);
    }
    let bytes = unsafe { slice.checked_slice(ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1) }
        .expect("runtime startup byte slices are validated before Play scene parsing");
    let value = std::str::from_utf8(bytes)
        .map_err(|source| RuntimeProjectError::PlaySceneUtf8 { source })?;
    if value.trim().is_empty() {
        return Err(RuntimeProjectError::EmptyPlayScene);
    }
    RelPath::parse(value)
        .map(Some)
        .map_err(|source| RuntimeProjectError::InvalidPlayScene {
            scene: value.to_owned(),
            source,
        })
}

fn parse_optional_play_report_pipe(slice: ZrByteSlice) -> RuntimeProjectResult<Option<String>> {
    if slice.is_empty() {
        return Ok(None);
    }
    let bytes = unsafe { slice.checked_slice(ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1) }
        .expect("runtime startup byte slices are validated before Play report parsing");
    let value = std::str::from_utf8(bytes)
        .map_err(|source| RuntimeProjectError::PlayReportPipeUtf8 { source })?;
    if value.trim().is_empty() {
        return Err(RuntimeProjectError::EmptyPlayReportPipe);
    }
    Ok(Some(value.to_owned()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeLoadedProjectManifest {
    default_scene: String,
    ui_roots: Vec<crate::asset::AssetUri>,
    plugins: ProjectPluginManifest,
    scripts: ProjectScriptManifest,
}

impl RuntimeLoadedProjectManifest {
    fn script_package_roots(&self, project_root: &Path) -> Vec<PathBuf> {
        self.scripts
            .package_roots
            .iter()
            .map(|root| project_root.join(root))
            .collect()
    }

    fn filter_startup_packages(
        &self,
        packages: Vec<crate::script::DiscoveredVmPluginPackage>,
    ) -> RuntimeProjectResult<Vec<crate::script::DiscoveredVmPluginPackage>> {
        if self.scripts.startup_packages.is_empty() {
            return Ok(packages);
        }
        let discovered = packages
            .iter()
            .map(|package| package.package.manifest.name.clone())
            .collect::<HashSet<_>>();
        for startup_package in &self.scripts.startup_packages {
            if !discovered.contains(startup_package) {
                return Err(RuntimeProjectError::MissingStartupScriptPackage {
                    package: startup_package.clone(),
                });
            }
        }
        Ok(packages
            .into_iter()
            .filter(|package| {
                self.scripts
                    .startup_packages
                    .iter()
                    .any(|name| name == &package.package.manifest.name)
            })
            .collect())
    }
}

impl From<&ProjectManifest> for RuntimeLoadedProjectManifest {
    fn from(manifest: &ProjectManifest) -> Self {
        Self {
            default_scene: manifest.default_scene.to_string(),
            ui_roots: manifest.ui_roots.clone(),
            plugins: manifest.plugins.clone(),
            scripts: manifest.scripts.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::asset::project::{ProjectManifest, ProjectPaths, ProjectScriptManifest};
    use crate::asset::{AssetUri, ProjectInfo};
    use crate::core::framework::project::ProjectPluginManifest;
    use crate::script::{
        CapabilitySet, DiscoveredVmPluginPackage, VmPluginManagementPolicy, VmPluginManifest,
        VmPluginPackage, VmPluginPackageSource,
    };
    use zircon_runtime_interface::ZrByteSlice;

    use super::{
        project_opened_log, RuntimeLoadedProjectManifest, RuntimeProjectConfig, RuntimeProjectError,
    };

    #[test]
    fn project_opened_log_reports_the_activated_project_snapshot() {
        let log = project_opened_log(&ProjectInfo {
            root_path: "C:\\projects\\renderable-empty".to_string(),
            name: "Renderable Empty".to_string(),
            default_scene_uri: "res://scenes/main.scene.toml".to_string(),
            library_version: 1,
            asset_count: 8,
            ready_asset_count: 7,
            failed_asset_count: 1,
            registry_diagnostic_count: 2,
        });

        assert_eq!(
            log,
            "runtime_project_opened root=C:\\projects\\renderable-empty name=Renderable Empty default_scene=res://scenes/main.scene.toml library_version=1 assets=8 ready_assets=7 failed_assets=1 registry_diagnostics=2"
        );
    }

    #[test]
    fn project_config_omits_empty_abi_slice() {
        let parsed = RuntimeProjectConfig::from_abi_slice(ZrByteSlice::empty()).unwrap();

        assert_eq!(parsed, None);
    }

    #[test]
    fn project_config_rejects_whitespace_only_path() {
        let raw = b"   ";
        let error = RuntimeProjectConfig::from_abi_slice(ZrByteSlice {
            data: raw.as_ptr(),
            len: raw.len(),
        })
        .unwrap_err();

        assert_eq!(error.to_string(), "runtime project root cannot be empty");
    }

    #[test]
    fn project_startup_rejects_play_inputs_without_a_project_root() {
        let scene = b".zircon/play/instance/play-scene.zrscene.json";
        let error = RuntimeProjectConfig::from_abi_startup_config(
            ZrByteSlice::empty(),
            ZrByteSlice {
                data: scene.as_ptr(),
                len: scene.len(),
            },
            ZrByteSlice::empty(),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            RuntimeProjectError::PlaySceneRequiresProject
        ));
    }

    #[test]
    fn project_startup_keeps_the_existing_rel_path_contract_for_play_scene() {
        let root = b"examples/vampire";
        let scene = b".zircon/play/instance/play-scene.zrscene.json";
        let parsed = RuntimeProjectConfig::from_abi_startup_config(
            ZrByteSlice {
                data: root.as_ptr(),
                len: root.len(),
            },
            ZrByteSlice {
                data: scene.as_ptr(),
                len: scene.len(),
            },
            ZrByteSlice::empty(),
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            parsed.play_scene.as_ref().map(|scene| scene.as_str()),
            Some(".zircon/play/instance/play-scene.zrscene.json")
        );
    }

    #[test]
    fn project_startup_rejects_absolute_play_scene_and_blank_report_outlet() {
        let root = b"examples/vampire";
        let absolute_scene = b"C:/outside.scene.toml";
        let scene_error = RuntimeProjectConfig::from_abi_startup_config(
            ZrByteSlice {
                data: root.as_ptr(),
                len: root.len(),
            },
            ZrByteSlice {
                data: absolute_scene.as_ptr(),
                len: absolute_scene.len(),
            },
            ZrByteSlice::empty(),
        )
        .unwrap_err();
        assert!(matches!(
            scene_error,
            RuntimeProjectError::InvalidPlayScene { .. }
        ));

        let blank_outlet = b"  ";
        let outlet_error = RuntimeProjectConfig::from_abi_startup_config(
            ZrByteSlice {
                data: root.as_ptr(),
                len: root.len(),
            },
            ZrByteSlice::empty(),
            ZrByteSlice {
                data: blank_outlet.as_ptr(),
                len: blank_outlet.len(),
            },
        )
        .unwrap_err();
        assert!(matches!(
            outlet_error,
            RuntimeProjectError::EmptyPlayReportPipe
        ));
    }

    #[test]
    fn project_config_parses_project_root_path() {
        let raw = b"examples/vampire";
        let parsed = RuntimeProjectConfig::from_abi_slice(ZrByteSlice {
            data: raw.as_ptr(),
            len: raw.len(),
        })
        .unwrap()
        .unwrap();

        assert_eq!(
            parsed.root_display(),
            ProjectPaths::resolve_path("examples/vampire")
                .unwrap()
                .display_path()
                .display()
                .to_string()
        );
    }

    #[test]
    fn project_config_normalizes_a_manifest_input_to_its_project_root() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("templates")
            .join("projects")
            .join("renderable-empty");
        let config = RuntimeProjectConfig::from_root(root.join("zircon-project.toml")).unwrap();

        assert_eq!(
            config.root_display(),
            ProjectPaths::resolve_existing(&root)
                .unwrap()
                .display_path()
                .display()
                .to_string()
        );
    }

    #[cfg(windows)]
    #[test]
    fn project_config_displays_operation_root_without_verbatim_prefix() {
        let project =
            RuntimeProjectConfig::from_root(r"\\?\C:\ZirconBuilds\stage\project").unwrap();

        assert_eq!(project.root_display(), r"C:\ZirconBuilds\stage\project");
    }

    #[cfg(windows)]
    #[test]
    fn project_config_rejects_drive_relative_abi_paths_at_the_resolver_boundary() {
        let raw = br"C:runtime-project";
        let error = RuntimeProjectConfig::from_abi_slice(ZrByteSlice {
            data: raw.as_ptr(),
            len: raw.len(),
        })
        .unwrap_err();

        assert!(matches!(
            error,
            RuntimeProjectError::ResolveProjectRoot { .. }
        ));
        assert!(error
            .to_string()
            .contains("Windows project paths must be drive-rooted, not drive-relative"));
    }

    #[test]
    fn project_startup_snapshot_survives_disk_manifest_rewrite_before_activation() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon_runtime_prepared_project_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("zircon-project.toml");
        ProjectManifest::new(
            "Prepared Snapshot One",
            AssetUri::parse("res://scenes/one.scene.toml").unwrap(),
            1,
        )
        .save(&manifest_path)
        .unwrap();

        let prepared = RuntimeProjectConfig::from_root(&root)
            .unwrap()
            .prepare()
            .unwrap();

        ProjectManifest::new(
            "Prepared Snapshot Two",
            AssetUri::parse("res://scenes/two.scene.toml").unwrap(),
            2,
        )
        .save(&manifest_path)
        .unwrap();

        assert_eq!(
            prepared.manifest.default_scene,
            "res://scenes/one.scene.toml"
        );
        assert!(prepared.project.is_some());
        assert_eq!(prepared.root_display(), root.to_string_lossy());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prepared_project_normalizes_a_relative_root_before_runtime_consumers_use_it() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let relative_root = PathBuf::from(format!(
            "zircon_runtime_relative_project_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&relative_root).unwrap();
        ProjectManifest::new(
            "Relative Runtime Root",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(relative_root.join("zircon-project.toml"))
        .unwrap();

        let config = RuntimeProjectConfig::from_root(&relative_root).unwrap();
        assert_eq!(
            config.root_display(),
            std::env::current_dir()
                .unwrap()
                .join(&relative_root)
                .to_string_lossy()
                .into_owned()
        );
        let prepared = config.prepare().unwrap();

        assert_eq!(
            prepared.root_display(),
            std::env::current_dir()
                .unwrap()
                .join(&relative_root)
                .to_string_lossy()
                .into_owned()
        );

        fs::remove_dir_all(relative_root).unwrap();
    }

    #[test]
    fn project_manifest_filters_startup_script_packages() {
        let manifest = RuntimeLoadedProjectManifest {
            default_scene: "res://scenes/main.scene.toml".to_string(),
            ui_roots: Vec::new(),
            plugins: ProjectPluginManifest::default(),
            scripts: ProjectScriptManifest {
                package_roots: vec!["scripts".to_string()],
                startup_packages: vec!["vampire_game".to_string()],
            },
        };

        let packages = manifest
            .filter_startup_packages(vec![
                script_package("debug_tools"),
                script_package("vampire_game"),
            ])
            .unwrap();

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].package.manifest.name, "vampire_game");
    }

    #[test]
    fn project_manifest_rejects_missing_startup_script_package() {
        let manifest = RuntimeLoadedProjectManifest {
            default_scene: "res://scenes/main.scene.toml".to_string(),
            ui_roots: Vec::new(),
            plugins: ProjectPluginManifest::default(),
            scripts: ProjectScriptManifest {
                package_roots: vec!["scripts".to_string()],
                startup_packages: vec!["vampire_game".to_string()],
            },
        };

        let error = manifest
            .filter_startup_packages(vec![script_package("debug_tools")])
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup script package vampire_game was not found"
        );
    }

    fn script_package(name: &str) -> DiscoveredVmPluginPackage {
        DiscoveredVmPluginPackage {
            backend_name: "mock".to_string(),
            source: VmPluginPackageSource::default(),
            package: VmPluginPackage {
                manifest: VmPluginManifest {
                    name: name.to_string(),
                    version: "0.1.0".to_string(),
                    entry: "main".to_string(),
                    capabilities: CapabilitySet::default(),
                    management: VmPluginManagementPolicy::default(),
                },
                zr_vm_project: None,
                bytecode: Vec::new(),
            },
        }
    }
}
