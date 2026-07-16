use std::collections::HashSet;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::ZrByteSlice;

use crate::asset::project::ProjectManifest;
use crate::asset::project::{ProjectManager, ProjectScriptManifest};
use crate::asset::{asset_manager_handle, project_asset_manager_handle};
use crate::core::framework::navigation::NavMeshAsset;
use crate::core::framework::project::ProjectPluginManifest;
use crate::core::manager::{navigation_manager_handle, resolve_manager_service};
use crate::core::CoreHandle;
use crate::diagnostic_log::write_log;
use crate::scene::{DynamicSceneAssetReloadQueue, LevelSystem};
use crate::script::{VmPluginManager, VM_PLUGIN_MANAGER_NAME};

use super::error::{RuntimeProjectError, RuntimeProjectResult};

const DEFAULT_PROJECT_NAVMESH_PATH: &[&str] = &["assets", "navigation", "main.navmesh.toml"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeProjectConfig {
    root: PathBuf,
}

impl RuntimeProjectConfig {
    pub(super) fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub(super) fn from_abi_slice(slice: ZrByteSlice) -> RuntimeProjectResult<Option<Self>> {
        if slice.is_empty() {
            return Ok(None);
        }
        let bytes = unsafe { slice.as_slice() };
        let value = std::str::from_utf8(bytes)
            .map_err(|source| RuntimeProjectError::ProjectRootUtf8 { source })?
            .trim();
        if value.is_empty() {
            return Err(RuntimeProjectError::EmptyProjectRoot);
        }
        Ok(Some(Self {
            root: PathBuf::from(value),
        }))
    }

    pub(super) fn root_display(&self) -> String {
        self.root.to_string_lossy().into_owned()
    }

    pub(super) fn prepare(self) -> RuntimeProjectResult<RuntimePreparedProject> {
        let project = ProjectManager::open(&self.root).map_err(|source| {
            RuntimeProjectError::OpenProject {
                root: self.root.clone(),
                source,
            }
        })?;
        let manifest = RuntimeLoadedProjectManifest::from(project.manifest());
        Ok(RuntimePreparedProject {
            root: self.root,
            manifest,
            project: Some(project),
        })
    }
}

pub(super) struct RuntimePreparedProject {
    root: PathBuf,
    manifest: RuntimeLoadedProjectManifest,
    project: Option<ProjectManager>,
}

impl RuntimePreparedProject {
    pub(super) fn root_display(&self) -> String {
        self.root.to_string_lossy().into_owned()
    }

    pub(super) fn plugin_manifest(&self) -> &ProjectPluginManifest {
        &self.manifest.plugins
    }

    pub(super) fn open_project_assets(&mut self, core: &CoreHandle) -> RuntimeProjectResult<()> {
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
            .map(|_| ())
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
            write_log(
                "runtime_session",
                format!(
                    "runtime_project_script_discover_start root={}",
                    root.display()
                ),
            );
            packages.extend(manager.discover_packages(&root).map_err(|source| {
                RuntimeProjectError::DiscoverScriptPackages {
                    root: root.clone(),
                    source,
                }
            })?);
            write_log(
                "runtime_session",
                format!(
                    "runtime_project_script_discover_done root={} packages={}",
                    root.display(),
                    packages.len()
                ),
            );
        }
        for package in self.manifest.filter_startup_packages(packages)? {
            write_log(
                "runtime_session",
                format!(
                    "runtime_project_script_load_start package={} backend={}",
                    package.package.manifest.name, package.backend_name
                ),
            );
            manager
                .load_discovered_package(&package)
                .map_err(|source| RuntimeProjectError::LoadScriptPackage {
                    package: package.package.manifest.name.clone(),
                    source,
                })?;
            write_log(
                "runtime_session",
                format!(
                    "runtime_project_script_load_done package={}",
                    package.package.manifest.name
                ),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeLoadedProjectManifest {
    default_scene: String,
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
            plugins: manifest.plugins.clone(),
            scripts: manifest.scripts.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::asset::project::ProjectManifest;
    use crate::asset::project::ProjectScriptManifest;
    use crate::asset::AssetUri;
    use crate::core::framework::project::ProjectPluginManifest;
    use crate::script::{
        CapabilitySet, DiscoveredVmPluginPackage, VmPluginManagementPolicy, VmPluginManifest,
        VmPluginPackage, VmPluginPackageSource,
    };
    use zircon_runtime_interface::ZrByteSlice;

    use super::{RuntimeLoadedProjectManifest, RuntimeProjectConfig};

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
    fn project_config_parses_project_root_path() {
        let raw = b"examples/vampire";
        let parsed = RuntimeProjectConfig::from_abi_slice(ZrByteSlice {
            data: raw.as_ptr(),
            len: raw.len(),
        })
        .unwrap()
        .unwrap();

        assert_eq!(parsed.root_display(), "examples/vampire");
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

        let prepared = RuntimeProjectConfig::from_root(&root).prepare().unwrap();

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
    fn project_manifest_filters_startup_script_packages() {
        let manifest = RuntimeLoadedProjectManifest {
            default_scene: "res://scenes/main.scene.toml".to_string(),
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
