use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::script::{
    CapabilitySet, VmError, VmPluginManifest, VmPluginPackage, VmPluginPackageSource,
    ZrVmExecutionMode, ZrVmPluginProjectSource,
};

const DEFAULT_BACKEND_NAME: &str = "unavailable";
const DEFAULT_BYTECODE_FILE: &str = "plugin.bin";
const PLUGIN_MANIFEST_FILE: &str = "plugin.toml";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredVmPluginPackage {
    pub backend_name: String,
    pub source: VmPluginPackageSource,
    pub package: VmPluginPackage,
}

#[derive(Debug, Deserialize)]
struct DiskVmPluginManifest {
    name: String,
    version: String,
    entry: String,
    #[serde(default)]
    capabilities: CapabilitySet,
    #[serde(default = "default_backend_name")]
    backend: String,
    #[serde(default)]
    bytecode: Option<String>,
    #[serde(default)]
    zr_vm: Option<DiskZrVmProject>,
}

#[derive(Debug, Deserialize)]
struct DiskZrVmProject {
    project: String,
    #[serde(default = "default_zr_vm_entry_module")]
    entry_module: String,
    #[serde(default)]
    execution_mode: ZrVmExecutionMode,
}

pub fn discover_vm_plugin_packages(
    root: impl AsRef<Path>,
) -> Result<Vec<DiscoveredVmPluginPackage>, VmError> {
    let root = root.as_ref();
    if !root.exists() {
        return Err(VmError::Operation(format!(
            "plugin package root does not exist: {}",
            root.display()
        )));
    }

    let mut manifest_paths = Vec::new();
    collect_plugin_manifests(root, &mut manifest_paths)?;
    let mut packages = manifest_paths
        .into_iter()
        .map(discover_vm_plugin_package)
        .collect::<Result<Vec<_>, _>>()?;
    packages.sort_by(|left, right| {
        left.package
            .manifest
            .name
            .cmp(&right.package.manifest.name)
            .then_with(|| {
                left.package
                    .manifest
                    .version
                    .cmp(&right.package.manifest.version)
            })
    });
    Ok(packages)
}

pub fn discover_vm_plugin_package(
    manifest_path: impl AsRef<Path>,
) -> Result<DiscoveredVmPluginPackage, VmError> {
    let manifest_path = manifest_path.as_ref();
    let manifest_source = fs::read_to_string(manifest_path).map_err(|error| {
        VmError::Operation(format!(
            "failed to read plugin manifest {}: {error}",
            manifest_path.display()
        ))
    })?;
    let disk_manifest: DiskVmPluginManifest =
        toml::from_str(&manifest_source).map_err(|error| {
            VmError::Parse(format!(
                "failed to parse plugin manifest {}: {error}",
                manifest_path.display()
            ))
        })?;

    let package_root = manifest_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            VmError::Operation(format!(
                "plugin manifest has no parent directory: {}",
                manifest_path.display()
            ))
        })?;
    let (bytecode, bytecode_path, zr_vm_project) =
        load_package_payload(&package_root, &disk_manifest)?;

    Ok(DiscoveredVmPluginPackage {
        backend_name: disk_manifest.backend,
        source: VmPluginPackageSource {
            package_root: Some(package_root),
            manifest_path: Some(manifest_path.to_path_buf()),
            bytecode_path,
            zr_vm_project_path: zr_vm_project
                .as_ref()
                .map(|project| project.project_path.clone()),
        },
        package: VmPluginPackage {
            manifest: VmPluginManifest {
                name: disk_manifest.name,
                version: disk_manifest.version,
                entry: disk_manifest.entry,
                capabilities: disk_manifest.capabilities,
            },
            zr_vm_project,
            bytecode,
        },
    })
}

fn load_package_payload(
    package_root: &Path,
    disk_manifest: &DiskVmPluginManifest,
) -> Result<(Vec<u8>, Option<PathBuf>, Option<ZrVmPluginProjectSource>), VmError> {
    if is_zr_vm_project_backend(&disk_manifest.backend) {
        let zr_vm = disk_manifest.zr_vm.as_ref().ok_or_else(|| {
            VmError::Parse("zr_vm:project backend requires a [zr_vm] project section".to_string())
        })?;
        let project_path = package_root.join(&zr_vm.project);
        if !project_path.exists() {
            return Err(VmError::Operation(format!(
                "zr_vm project does not exist: {}",
                project_path.display()
            )));
        }
        return Ok((
            Vec::new(),
            None,
            Some(ZrVmPluginProjectSource {
                project_path,
                entry_module: zr_vm.entry_module.clone(),
                execution_mode: zr_vm.execution_mode,
            }),
        ));
    }

    if disk_manifest.zr_vm.is_some() {
        return Err(VmError::Parse(
            "[zr_vm] project section requires backend = \"zr_vm:project\"".to_string(),
        ));
    }

    let bytecode_file = disk_manifest
        .bytecode
        .clone()
        .unwrap_or_else(default_bytecode_file);
    let bytecode_path = package_root.join(bytecode_file);
    let bytecode = fs::read(&bytecode_path).map_err(|error| {
        VmError::Operation(format!(
            "failed to read plugin bytecode {}: {error}",
            bytecode_path.display()
        ))
    })?;
    Ok((bytecode, Some(bytecode_path), None))
}

fn collect_plugin_manifests(root: &Path, manifest_paths: &mut Vec<PathBuf>) -> Result<(), VmError> {
    for entry in fs::read_dir(root).map_err(|error| {
        VmError::Operation(format!(
            "failed to enumerate plugin package root {}: {error}",
            root.display()
        ))
    })? {
        let entry = entry.map_err(|error| {
            VmError::Operation(format!(
                "failed to inspect plugin package entry under {}: {error}",
                root.display()
            ))
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_plugin_manifests(&path, manifest_paths)?;
        } else if path.file_name().and_then(|value| value.to_str()) == Some(PLUGIN_MANIFEST_FILE) {
            manifest_paths.push(path);
        }
    }
    Ok(())
}

fn default_backend_name() -> String {
    DEFAULT_BACKEND_NAME.to_string()
}

fn default_bytecode_file() -> String {
    DEFAULT_BYTECODE_FILE.to_string()
}

fn default_zr_vm_entry_module() -> String {
    "main".to_string()
}

fn is_zr_vm_project_backend(backend: &str) -> bool {
    backend == "zr_vm:project"
}
