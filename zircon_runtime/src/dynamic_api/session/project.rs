use std::collections::HashSet;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::ZrByteSlice;

use crate::asset::project::{ProjectManager, ProjectScriptManifest};
use crate::asset::{ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};
use crate::core::framework::navigation::NavMeshAsset;
use crate::core::manager::resolve_navigation_manager;
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

    pub(super) fn load_manifest(&self) -> RuntimeProjectResult<RuntimeLoadedProjectManifest> {
        let project = ProjectManager::open(&self.root).map_err(|source| {
            RuntimeProjectError::OpenProject {
                root: self.root.clone(),
                source,
            }
        })?;
        Ok(RuntimeLoadedProjectManifest {
            default_scene: project.manifest().default_scene.to_string(),
            scripts: project.manifest().scripts.clone(),
        })
    }

    pub(super) fn open_project_assets(&self, core: &CoreHandle) -> RuntimeProjectResult<()> {
        let asset_manager =
            crate::asset::pipeline::manager::resolve_asset_manager(core).map_err(|source| {
                RuntimeProjectError::ResolveAssetManager {
                    root: self.root.clone(),
                    source,
                }
            })?;
        let asset_manager = asset_manager.shared();
        asset_manager
            .open_project(&self.root_display())
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
        let manifest = self.load_manifest()?;
        crate::scene::load_level_asset(core, &self.root_display(), manifest.default_scene.as_str())
            .map_err(|source| RuntimeProjectError::LoadDefaultScene {
                root: self.root.clone(),
                scene: manifest.default_scene,
                source,
            })
    }

    pub(super) fn scene_asset_reload_queue(
        &self,
        core: &CoreHandle,
    ) -> RuntimeProjectResult<DynamicSceneAssetReloadQueue> {
        let asset_manager = core
            .resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
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
        let navigation = resolve_navigation_manager(core).map_err(|source| {
            RuntimeProjectError::ResolveNavigationManager {
                root: self.root.clone(),
                source,
            }
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
        let manifest = self.load_manifest()?;
        if manifest.scripts.is_empty() {
            return Ok(());
        }
        let manager = core
            .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
            .map_err(|source| RuntimeProjectError::ResolveScriptManager {
                root: self.root.clone(),
                source,
            })?;
        let mut packages = Vec::new();
        for root in manifest.script_package_roots(&self.root) {
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
        for package in manifest.filter_startup_packages(packages)? {
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

#[cfg(test)]
mod tests {
    use crate::asset::project::ProjectScriptManifest;
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
    fn project_manifest_filters_startup_script_packages() {
        let manifest = RuntimeLoadedProjectManifest {
            default_scene: "res://scenes/main.scene.toml".to_string(),
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
