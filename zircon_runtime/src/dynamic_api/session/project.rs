use std::collections::HashSet;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::ZrByteSlice;

use crate::asset::project::{ProjectManager, ProjectScriptManifest};
use crate::asset::NavMeshAsset;
use crate::asset::{ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};
use crate::core::manager::resolve_navigation_manager;
use crate::core::CoreHandle;
use crate::diagnostic_log::write_log;
use crate::scene::{DynamicSceneAssetReloadQueue, LevelSystem};
use crate::script::{VmPluginManager, VM_PLUGIN_MANAGER_NAME};

const DEFAULT_PROJECT_NAVMESH_PATH: &[&str] = &["assets", "navigation", "main.navmesh.toml"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeProjectConfig {
    root: PathBuf,
}

impl RuntimeProjectConfig {
    pub(super) fn from_abi_slice(slice: ZrByteSlice) -> Result<Option<Self>, String> {
        if slice.is_empty() {
            return Ok(None);
        }
        let bytes = unsafe { slice.as_slice() };
        let value = std::str::from_utf8(bytes)
            .map_err(|error| format!("runtime project root must be UTF-8: {error}"))?
            .trim();
        if value.is_empty() {
            return Err("runtime project root cannot be empty".to_string());
        }
        Ok(Some(Self {
            root: PathBuf::from(value),
        }))
    }

    pub(super) fn root_display(&self) -> String {
        self.root.to_string_lossy().into_owned()
    }

    pub(super) fn load_manifest(&self) -> Result<RuntimeLoadedProjectManifest, String> {
        let project = ProjectManager::open(&self.root).map_err(|error| {
            format!(
                "failed to open runtime project {}: {error}",
                self.root.display()
            )
        })?;
        Ok(RuntimeLoadedProjectManifest {
            default_scene: project.manifest().default_scene.to_string(),
            scripts: project.manifest().scripts.clone(),
        })
    }

    pub(super) fn open_project_assets(&self, core: &CoreHandle) -> Result<(), String> {
        let asset_manager =
            crate::asset::pipeline::manager::resolve_asset_manager(core).map_err(|error| {
                format!(
                    "runtime project {} requires AssetManager but it is unavailable: {error}",
                    self.root.display()
                )
            })?;
        asset_manager
            .open_project(&self.root_display())
            .map(|_| ())
            .map_err(|error| {
                format!(
                    "failed to open runtime project assets {}: {error}",
                    self.root.display()
                )
            })
    }

    pub(super) fn load_default_level(&self, core: &CoreHandle) -> Result<LevelSystem, String> {
        let manifest = self.load_manifest()?;
        crate::scene::load_level_asset(core, &self.root_display(), manifest.default_scene.as_str())
            .map_err(|error| {
                format!(
                    "failed to load default scene {} from project {}: {error}",
                    manifest.default_scene,
                    self.root.display()
                )
            })
    }

    pub(super) fn scene_asset_reload_queue(
        &self,
        core: &CoreHandle,
    ) -> Result<DynamicSceneAssetReloadQueue, String> {
        let asset_manager = core
            .resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
            .map_err(|error| {
                format!(
                    "runtime project {} requires ProjectAssetManager for scene asset reloads but it is unavailable: {error}",
                    self.root.display()
                )
            })?;
        let project = asset_manager.current_project_manager().ok_or_else(|| {
            format!(
                "runtime project {} has no active ProjectManager for scene asset reloads",
                self.root.display()
            )
        })?;
        Ok(DynamicSceneAssetReloadQueue::from_project_asset_manager(
            project,
            asset_manager.as_ref(),
        ))
    }

    pub(super) fn load_default_navigation(&self, core: &CoreHandle) -> Result<(), String> {
        let navmesh_path = DEFAULT_PROJECT_NAVMESH_PATH
            .iter()
            .fold(self.root.clone(), |path, segment| path.join(segment));
        if !navmesh_path.exists() {
            return Ok(());
        }
        let document = std::fs::read_to_string(&navmesh_path).map_err(|error| {
            format!(
                "failed to read runtime navmesh {}: {error}",
                navmesh_path.display()
            )
        })?;
        let asset = toml::from_str::<NavMeshAsset>(&document).map_err(|error| {
            format!(
                "failed to parse runtime navmesh {}: {error}",
                navmesh_path.display()
            )
        })?;
        let navigation = resolve_navigation_manager(core).map_err(|error| {
            format!(
                "runtime project {} declares a navmesh but NavigationManager is unavailable: {error}",
                self.root.display()
            )
        })?;
        navigation
            .load_nav_mesh(asset)
            .map(|_| ())
            .map_err(|error| {
                format!(
                    "failed to load runtime navmesh {}: {error}",
                    navmesh_path.display()
                )
            })
    }

    pub(super) fn load_startup_scripts(&self, core: &CoreHandle) -> Result<(), String> {
        let manifest = self.load_manifest()?;
        if manifest.scripts.is_empty() {
            return Ok(());
        }
        let manager = core
            .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
            .map_err(|error| {
                format!(
                    "runtime project {} declares scripts but ScriptModule is unavailable: {error}",
                    self.root.display()
                )
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
            packages.extend(manager.discover_packages(&root).map_err(|error| {
                format!(
                    "failed to discover runtime script packages under {}: {error}",
                    root.display()
                )
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
            manager.load_discovered_package(&package).map_err(|error| {
                format!(
                    "failed to load runtime script package {}: {error}",
                    package.package.manifest.name
                )
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
    ) -> Result<Vec<crate::script::DiscoveredVmPluginPackage>, String> {
        if self.scripts.startup_packages.is_empty() {
            return Ok(packages);
        }
        let discovered = packages
            .iter()
            .map(|package| package.package.manifest.name.clone())
            .collect::<HashSet<_>>();
        for startup_package in &self.scripts.startup_packages {
            if !discovered.contains(startup_package) {
                return Err(format!(
                    "runtime startup script package {startup_package} was not found"
                ));
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

        assert_eq!(error, "runtime project root cannot be empty");
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
            error,
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
