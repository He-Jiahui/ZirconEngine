mod io;
mod limits;
mod payload_cache;

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use serde::Deserialize;

use crate::script::{
    CapabilitySet, VmError, VmPluginManagementPolicy, VmPluginManifest, VmPluginPackage,
    VmPluginPackageSource, ZrVmExecutionMode, ZrVmPluginProjectSource,
};

pub use io::VmPluginDiscoveryRequest;
pub(crate) use io::VmPluginDiscoveryWorker;
pub use limits::VmPluginDiscoveryLimits;
pub(crate) use payload_cache::VmPluginPayloadCache;

use payload_cache::read_bounded_file;

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
    #[serde(default)]
    management: VmPluginManagementPolicy,
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
    discover_vm_plugin_packages_with_limits(root, VmPluginDiscoveryLimits::default())
}

pub fn discover_vm_plugin_packages_with_limits(
    root: impl AsRef<Path>,
    limits: VmPluginDiscoveryLimits,
) -> Result<Vec<DiscoveredVmPluginPackage>, VmError> {
    discover_vm_plugin_packages_internal(root.as_ref(), limits, None)
}

pub(super) fn discover_vm_plugin_packages_cancellable(
    root: PathBuf,
    limits: VmPluginDiscoveryLimits,
    cancellation: Arc<AtomicBool>,
) -> Result<Vec<DiscoveredVmPluginPackage>, VmError> {
    discover_vm_plugin_packages_internal(&root, limits, Some(cancellation))
}

fn discover_vm_plugin_packages_internal(
    root: &Path,
    limits: VmPluginDiscoveryLimits,
    cancellation: Option<Arc<AtomicBool>>,
) -> Result<Vec<DiscoveredVmPluginPackage>, VmError> {
    let root = canonical_discovery_root(root.as_ref())?;
    let mut budget = DiscoveryBudget::new(limits, cancellation);

    let mut manifest_paths = Vec::new();
    collect_plugin_manifests(&root, 0, &mut manifest_paths, &mut budget)?;
    manifest_paths.sort();
    let mut packages = manifest_paths
        .into_iter()
        .map(|manifest_path| discover_manifest(&manifest_path, &mut budget))
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
    discover_vm_plugin_package_with_limits(manifest_path, VmPluginDiscoveryLimits::default())
}

pub fn discover_vm_plugin_package_with_limits(
    manifest_path: impl AsRef<Path>,
    limits: VmPluginDiscoveryLimits,
) -> Result<DiscoveredVmPluginPackage, VmError> {
    let manifest_path = canonical_manifest_path(manifest_path.as_ref())?;
    let mut budget = DiscoveryBudget::new(limits, None);
    budget.record_entry(&manifest_path)?;
    budget.record_manifest(&manifest_path)?;
    discover_manifest(&manifest_path, &mut budget)
}

fn discover_manifest(
    manifest_path: &Path,
    budget: &mut DiscoveryBudget,
) -> Result<DiscoveredVmPluginPackage, VmError> {
    budget.check_elapsed()?;
    let manifest_len = fs::metadata(manifest_path)
        .map_err(|error| {
            VmError::Operation(format!(
                "failed to inspect plugin manifest {}: {error}",
                manifest_path.display()
            ))
        })?
        .len();
    budget.record_manifest_bytes(manifest_path, manifest_len)?;
    let manifest_bytes = read_bounded_file(
        manifest_path,
        budget.limits.max_manifest_bytes,
        "plugin manifest",
    )?;
    let manifest_source = String::from_utf8(manifest_bytes).map_err(|error| {
        VmError::Parse(format!(
            "plugin manifest {} is not UTF-8: {error}",
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
    let (bytecode_path, zr_vm_project) = resolve_package_payload(&package_root, &disk_manifest)?;
    disk_manifest.management.validate().map_err(|error| {
        VmError::Parse(format!(
            "invalid plugin management policy in {}: {error}",
            manifest_path.display()
        ))
    })?;

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
                management: disk_manifest.management,
            },
            zr_vm_project,
            bytecode: Vec::new(),
        },
    })
}

fn resolve_package_payload(
    package_root: &Path,
    disk_manifest: &DiskVmPluginManifest,
) -> Result<(Option<PathBuf>, Option<ZrVmPluginProjectSource>), VmError> {
    if is_zr_vm_project_backend(&disk_manifest.backend) {
        let zr_vm = disk_manifest.zr_vm.as_ref().ok_or_else(|| {
            VmError::Parse("zr_vm project backend requires a [zr_vm] project section".to_string())
        })?;
        let project_path = resolve_relative_package_path(package_root, &zr_vm.project, "project")?;
        validate_existing_project_path(package_root, &project_path)?;
        return Ok((
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
    let bytecode_path = resolve_relative_package_path(package_root, &bytecode_file, "bytecode")?;
    Ok((Some(bytecode_path), None))
}

fn collect_plugin_manifests(
    root: &Path,
    depth: usize,
    manifest_paths: &mut Vec<PathBuf>,
    budget: &mut DiscoveryBudget,
) -> Result<(), VmError> {
    budget.check_elapsed()?;
    let directory = fs::read_dir(root).map_err(|error| {
        VmError::Operation(format!(
            "failed to enumerate plugin package root {}: {error}",
            root.display()
        ))
    })?;
    let mut entries = Vec::new();
    for entry in directory {
        let entry = entry.map_err(|error| {
            VmError::Operation(format!(
                "failed to inspect plugin package entry under {}: {error}",
                root.display()
            ))
        })?;
        budget.record_entry(&entry.path())?;
        entries.push(entry);
    }
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            VmError::Operation(format!(
                "failed to inspect plugin package entry {}: {error}",
                path.display()
            ))
        })?;
        if file_type.is_symlink() {
            return Err(VmError::Operation(format!(
                "plugin discovery does not follow symbolic links: {}",
                path.display()
            )));
        }
        if file_type.is_dir() {
            if depth >= budget.limits.max_depth {
                return Err(VmError::Operation(format!(
                    "plugin discovery depth budget {} exceeded at {}",
                    budget.limits.max_depth,
                    path.display()
                )));
            }
            collect_plugin_manifests(&path, depth + 1, manifest_paths, budget)?;
        } else if path.file_name().and_then(|value| value.to_str()) == Some(PLUGIN_MANIFEST_FILE) {
            budget.record_manifest(&path)?;
            manifest_paths.push(path);
        }
    }
    Ok(())
}

struct DiscoveryBudget {
    limits: VmPluginDiscoveryLimits,
    started: Instant,
    entries: usize,
    manifests: usize,
    manifest_bytes: usize,
    path_bytes: usize,
    cancellation: Option<Arc<AtomicBool>>,
}

impl DiscoveryBudget {
    fn new(limits: VmPluginDiscoveryLimits, cancellation: Option<Arc<AtomicBool>>) -> Self {
        Self {
            limits,
            started: Instant::now(),
            entries: 0,
            manifests: 0,
            manifest_bytes: 0,
            path_bytes: 0,
            cancellation,
        }
    }

    fn record_entry(&mut self, path: &Path) -> Result<(), VmError> {
        self.check_elapsed()?;
        let path_bytes = path.as_os_str().as_encoded_bytes().len();
        if path_bytes > self.limits.max_path_bytes {
            return Err(VmError::Operation(format!(
                "plugin discovery path byte budget {} exceeded at {}",
                self.limits.max_path_bytes,
                path.display()
            )));
        }
        self.path_bytes = self.path_bytes.checked_add(path_bytes).ok_or_else(|| {
            VmError::Operation("plugin discovery path byte counter overflowed".to_string())
        })?;
        if self.path_bytes > self.limits.max_total_path_bytes {
            return Err(VmError::Operation(format!(
                "plugin discovery total path byte budget {} exceeded at {}",
                self.limits.max_total_path_bytes,
                path.display()
            )));
        }
        self.entries = self.entries.checked_add(1).ok_or_else(|| {
            VmError::Operation("plugin discovery entry counter overflowed".to_string())
        })?;
        if self.entries > self.limits.max_entries {
            return Err(VmError::Operation(format!(
                "plugin discovery entry budget {} exceeded at {}",
                self.limits.max_entries,
                path.display()
            )));
        }
        Ok(())
    }

    fn record_manifest(&mut self, path: &Path) -> Result<(), VmError> {
        self.manifests = self.manifests.checked_add(1).ok_or_else(|| {
            VmError::Operation("plugin discovery manifest counter overflowed".to_string())
        })?;
        if self.manifests > self.limits.max_manifests {
            return Err(VmError::Operation(format!(
                "plugin discovery manifest budget {} exceeded at {}",
                self.limits.max_manifests,
                path.display()
            )));
        }
        Ok(())
    }

    fn record_manifest_bytes(&mut self, path: &Path, bytes: u64) -> Result<(), VmError> {
        if bytes > self.limits.max_manifest_bytes as u64 {
            return Err(VmError::Operation(format!(
                "plugin manifest byte budget {} exceeded at {}",
                self.limits.max_manifest_bytes,
                path.display()
            )));
        }
        let bytes = usize::try_from(bytes).map_err(|_| {
            VmError::Operation(format!(
                "plugin manifest size cannot fit host usize: {}",
                path.display()
            ))
        })?;
        self.manifest_bytes = self.manifest_bytes.checked_add(bytes).ok_or_else(|| {
            VmError::Operation("plugin discovery manifest byte counter overflowed".to_string())
        })?;
        if self.manifest_bytes > self.limits.max_total_manifest_bytes {
            return Err(VmError::Operation(format!(
                "plugin discovery total manifest byte budget {} exceeded at {}",
                self.limits.max_total_manifest_bytes,
                path.display()
            )));
        }
        Ok(())
    }

    fn check_elapsed(&self) -> Result<(), VmError> {
        if self
            .cancellation
            .as_ref()
            .is_some_and(|cancellation| cancellation.load(Ordering::Acquire))
        {
            return Err(VmError::Operation(
                "plugin discovery was cancelled".to_string(),
            ));
        }
        if self.started.elapsed() > self.limits.max_wall_time {
            return Err(VmError::Operation(format!(
                "plugin discovery wall-time budget {:?} exceeded",
                self.limits.max_wall_time
            )));
        }
        Ok(())
    }
}

fn canonical_discovery_root(root: &Path) -> Result<PathBuf, VmError> {
    let metadata = fs::symlink_metadata(root).map_err(|error| {
        VmError::Operation(format!(
            "failed to inspect plugin package root {}: {error}",
            root.display()
        ))
    })?;
    if metadata.file_type().is_symlink() {
        return Err(VmError::Operation(format!(
            "plugin package root cannot be a symbolic link: {}",
            root.display()
        )));
    }
    if !metadata.is_dir() {
        return Err(VmError::Operation(format!(
            "plugin package root is not a directory: {}",
            root.display()
        )));
    }
    root.canonicalize().map_err(|error| {
        VmError::Operation(format!(
            "failed to resolve plugin package root {}: {error}",
            root.display()
        ))
    })
}

fn canonical_manifest_path(path: &Path) -> Result<PathBuf, VmError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        VmError::Operation(format!(
            "failed to inspect plugin manifest {}: {error}",
            path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(VmError::Operation(format!(
            "plugin manifest must be a regular non-symlink file: {}",
            path.display()
        )));
    }
    path.canonicalize().map_err(|error| {
        VmError::Operation(format!(
            "failed to resolve plugin manifest {}: {error}",
            path.display()
        ))
    })
}

fn resolve_relative_package_path(
    package_root: &Path,
    relative: &str,
    description: &str,
) -> Result<PathBuf, VmError> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(VmError::Parse(format!(
            "plugin {description} path must stay within its package root: {relative}"
        )));
    }
    Ok(package_root.join(relative_path))
}

fn validate_existing_project_path(package_root: &Path, project_path: &Path) -> Result<(), VmError> {
    let metadata = fs::symlink_metadata(project_path).map_err(|error| {
        VmError::Operation(format!(
            "failed to inspect zr_vm project {}: {error}",
            project_path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() {
        return Err(VmError::Operation(format!(
            "zr_vm project cannot be a symbolic link: {}",
            project_path.display()
        )));
    }
    let canonical_project = project_path.canonicalize().map_err(|error| {
        VmError::Operation(format!(
            "failed to resolve zr_vm project {}: {error}",
            project_path.display()
        ))
    })?;
    if !canonical_project.starts_with(package_root) {
        return Err(VmError::Operation(format!(
            "zr_vm project escapes package root {}: {}",
            package_root.display(),
            canonical_project.display()
        )));
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::script::{VmError, VmPluginGarbageCollectionMode, VmPluginHotReloadPolicy};

    use super::{
        discover_vm_plugin_package, discover_vm_plugin_package_with_limits,
        discover_vm_plugin_packages_with_limits, VmPluginDiscoveryLimits, VmPluginPayloadCache,
    };

    #[test]
    fn discovery_defers_bytecode_until_selected_package_materialization() {
        let fixture = PackageFixture::new();
        fixture.write_bytecode_package(concat!(
            "name = \"lazy_payload\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"main\"\n",
            "backend = \"mock\"\n",
            "bytecode = \"plugin.bin\"\n",
        ));

        let discovered = discover_vm_plugin_package(&fixture.manifest_path).unwrap();

        assert!(discovered.package.bytecode.is_empty());
        let cache = VmPluginPayloadCache::default();
        let materialized = cache.materialize(&discovered).unwrap();
        assert_eq!(materialized.bytecode, [1, 2, 3]);
    }

    #[test]
    fn discovery_rejects_tree_depth_beyond_the_configured_limit() {
        let fixture = PackageFixture::new();
        let nested_root = fixture.root.join("nested");
        fs::create_dir_all(&nested_root).unwrap();
        fs::write(
            nested_root.join("plugin.toml"),
            concat!(
                "name = \"too_deep\"\n",
                "version = \"0.1.0\"\n",
                "entry = \"main\"\n",
                "backend = \"mock\"\n",
            ),
        )
        .unwrap();
        let limits = VmPluginDiscoveryLimits {
            max_depth: 0,
            ..VmPluginDiscoveryLimits::default()
        };

        let error = discover_vm_plugin_packages_with_limits(&fixture.root, limits).unwrap_err();

        assert!(error.to_string().contains("depth"));
    }

    #[test]
    fn discovery_rejects_manifest_and_bytecode_before_oversized_allocation() {
        let fixture = PackageFixture::new();
        fixture.write_bytecode_package(concat!(
            "name = \"bounded_payload\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"main\"\n",
            "backend = \"mock\"\n",
            "bytecode = \"plugin.bin\"\n",
        ));
        let manifest_limits = VmPluginDiscoveryLimits {
            max_manifest_bytes: 16,
            ..VmPluginDiscoveryLimits::default()
        };
        let manifest_error =
            discover_vm_plugin_package_with_limits(&fixture.manifest_path, manifest_limits)
                .unwrap_err();
        assert!(manifest_error.to_string().contains("manifest byte budget"));

        let discovered = discover_vm_plugin_package(&fixture.manifest_path).unwrap();
        let payload_cache = VmPluginPayloadCache::new(VmPluginDiscoveryLimits {
            max_bytecode_bytes: 2,
            ..VmPluginDiscoveryLimits::default()
        });
        let payload_error = payload_cache.materialize(&discovered).unwrap_err();
        assert!(payload_error.to_string().contains("bytecode byte budget"));
    }

    #[test]
    fn unchanged_bytecode_fingerprint_reuses_the_single_flight_payload() {
        let fixture = PackageFixture::new();
        fixture.write_bytecode_package(concat!(
            "name = \"single_flight\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"main\"\n",
            "backend = \"mock\"\n",
            "bytecode = \"plugin.bin\"\n",
        ));
        let cache = VmPluginPayloadCache::default();

        let first = cache.load_path(&fixture.bytecode_path).unwrap();
        let second = cache.load_path(&fixture.bytecode_path).unwrap();

        assert!(std::sync::Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn discovery_defaults_vm_management_policy_when_manifest_omits_it() {
        let fixture = PackageFixture::new();
        fixture.write_bytecode_package(concat!(
            "name = \"default_policy\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"main\"\n",
            "backend = \"mock\"\n",
            "bytecode = \"plugin.bin\"\n",
            "\n",
            "[capabilities]\n",
            "capabilities = [\"render\"]\n",
        ));

        let discovered = discover_vm_plugin_package(&fixture.manifest_path).unwrap();

        assert_eq!(
            discovered.package.manifest.management.hot_reload,
            VmPluginHotReloadPolicy::PreserveState
        );
        assert_eq!(
            discovered
                .package
                .manifest
                .management
                .garbage_collection
                .mode,
            VmPluginGarbageCollectionMode::BackendManaged
        );
    }

    #[test]
    fn discovery_parses_vm_management_policy_from_manifest() {
        let fixture = PackageFixture::new();
        fixture.write_bytecode_package(concat!(
            "name = \"managed_policy\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"main\"\n",
            "backend = \"mock\"\n",
            "bytecode = \"plugin.bin\"\n",
            "\n",
            "[capabilities]\n",
            "capabilities = [\"render\"]\n",
            "\n",
            "[management]\n",
            "hot_reload = \"stateless\"\n",
            "\n",
            "[management.garbage_collection]\n",
            "mode = \"cooperative\"\n",
            "interval_frames = 120\n",
            "\n",
            "[management.memory]\n",
            "soft_limit_bytes = 1024\n",
            "hard_limit_bytes = 2048\n",
        ));

        let discovered = discover_vm_plugin_package(&fixture.manifest_path).unwrap();
        let management = discovered.package.manifest.management;

        assert_eq!(management.hot_reload, VmPluginHotReloadPolicy::Stateless);
        assert_eq!(
            management.garbage_collection.mode,
            VmPluginGarbageCollectionMode::Cooperative
        );
        assert_eq!(management.garbage_collection.interval_frames, Some(120));
        assert_eq!(management.memory.soft_limit_bytes, Some(1024));
        assert_eq!(management.memory.hard_limit_bytes, Some(2048));
    }

    #[test]
    fn discovery_rejects_invalid_vm_management_policy() {
        let fixture = PackageFixture::new();
        fixture.write_bytecode_package(concat!(
            "name = \"bad_policy\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"main\"\n",
            "backend = \"mock\"\n",
            "bytecode = \"plugin.bin\"\n",
            "\n",
            "[capabilities]\n",
            "capabilities = [\"render\"]\n",
            "\n",
            "[management.memory]\n",
            "soft_limit_bytes = 2048\n",
            "hard_limit_bytes = 1024\n",
        ));

        let error = discover_vm_plugin_package(&fixture.manifest_path).unwrap_err();

        assert!(matches!(error, VmError::Parse(_)));
        assert!(error
            .to_string()
            .contains("invalid plugin management policy"));
        assert!(error.to_string().contains("soft_limit_bytes 2048 exceeds"));
    }

    #[test]
    fn discovery_rejects_zr_vm_project_fallback_backend() {
        let fixture = PackageFixture::new();
        fs::write(
            &fixture.manifest_path,
            concat!(
                "name = \"fallback_project\"\n",
                "version = \"0.1.0\"\n",
                "entry = \"main\"\n",
                "backend = \"zr_vm_fallback:project\"\n",
                "\n",
                "[zr_vm]\n",
                "project = \"script/plugin.zrp\"\n",
            ),
        )
        .unwrap();

        let error = discover_vm_plugin_package(&fixture.manifest_path).unwrap_err();

        assert!(matches!(error, VmError::Parse(_)));
        assert!(error
            .to_string()
            .contains("[zr_vm] project section requires backend = \"zr_vm:project\""));
    }

    struct PackageFixture {
        root: PathBuf,
        manifest_path: PathBuf,
        bytecode_path: PathBuf,
    }

    impl PackageFixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("zircon-vm-management-{nonce}"));
            fs::create_dir_all(&root).unwrap();
            Self {
                manifest_path: root.join("plugin.toml"),
                bytecode_path: root.join("plugin.bin"),
                root,
            }
        }

        fn write_bytecode_package(&self, manifest: &str) {
            fs::write(&self.manifest_path, manifest).unwrap();
            fs::write(&self.bytecode_path, [1, 2, 3]).unwrap();
        }
    }

    impl Drop for PackageFixture {
        fn drop(&mut self) {
            remove_dir_all_if_exists(&self.root);
        }
    }

    fn remove_dir_all_if_exists(path: &Path) {
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }
}
