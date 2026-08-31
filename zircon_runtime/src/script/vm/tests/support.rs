use super::*;
use crate::script::VmReflectionCatalog;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ObservedHostContext {
    pub(super) plugin_name: String,
    pub(super) backend_selector: String,
    pub(super) package_root: Option<PathBuf>,
    pub(super) source_root: Option<PathBuf>,
    pub(super) data_root: Option<PathBuf>,
    pub(super) package_source: VmPluginPackageSource,
    pub(super) capabilities: CapabilitySet,
}

impl ObservedHostContext {
    fn capture(host: &VmPluginHostContext) -> Self {
        Self {
            plugin_name: host.plugin.plugin_name.clone(),
            backend_selector: host.backend_selector.clone(),
            package_root: host.plugin.package_root.clone(),
            source_root: host.plugin.source_root.clone(),
            data_root: host.plugin.data_root.clone(),
            package_source: host.package_source.clone(),
            capabilities: host.capabilities.clone(),
        }
    }
}

#[derive(Debug, Default)]
struct NoopSlotLifecycle;

impl VmPluginSlotLifecycle for NoopSlotLifecycle {
    fn load_package(
        &self,
        backend_selector: &str,
        _package: VmPluginPackage,
    ) -> Result<super::super::PluginSlotId, VmError> {
        Err(VmError::Operation(format!(
            "noop slot lifecycle cannot load backend {backend_selector}"
        )))
    }

    fn hot_reload_slot(
        &self,
        slot: super::super::PluginSlotId,
        _package: VmPluginPackage,
    ) -> Result<(), VmError> {
        Err(VmError::Operation(format!(
            "noop slot lifecycle cannot hot reload slot {}",
            slot.get()
        )))
    }

    fn unload_slot(&self, slot: super::super::PluginSlotId) -> Result<(), VmError> {
        Err(VmError::Operation(format!(
            "noop slot lifecycle cannot unload slot {}",
            slot.get()
        )))
    }

    fn slot(&self, slot: super::super::PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        Err(VmError::MissingSlot(slot.get()))
    }

    fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        Vec::new()
    }
}

#[derive(Debug)]
struct RecordingVmPluginInstance {
    manifest: VmPluginManifest,
    observations: Arc<Mutex<Vec<ObservedHostContext>>>,
}

impl VmPluginInstance for RecordingVmPluginInstance {
    fn manifest(&self) -> &VmPluginManifest {
        &self.manifest
    }

    fn activate(&mut self, host: &VmPluginHostContext) -> Result<(), VmError> {
        self.observations
            .lock()
            .unwrap()
            .push(ObservedHostContext::capture(host));
        Ok(())
    }
}

#[derive(Debug)]
struct RecordingVmBackend {
    observations: Arc<Mutex<Vec<ObservedHostContext>>>,
}

impl VmBackend for RecordingVmBackend {
    fn backend_name(&self) -> &str {
        "recording"
    }

    fn load_package(
        &self,
        package: &VmPluginPackage,
        host: &VmPluginHostContext,
    ) -> Result<Box<dyn VmPluginInstance>, VmError> {
        self.observations
            .lock()
            .unwrap()
            .push(ObservedHostContext::capture(host));
        Ok(Box::new(RecordingVmPluginInstance {
            manifest: package.manifest.clone(),
            observations: Arc::clone(&self.observations),
        }))
    }
}

#[derive(Debug)]
pub(super) struct RecordingVmBackendFamily {
    observations: Arc<Mutex<Vec<ObservedHostContext>>>,
}

impl RecordingVmBackendFamily {
    pub(super) fn new(observations: Arc<Mutex<Vec<ObservedHostContext>>>) -> Self {
        Self { observations }
    }
}

impl VmBackendFamily for RecordingVmBackendFamily {
    fn family_name(&self) -> &str {
        "recording"
    }

    fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        match selector {
            "recording:capture" | "capture" => Ok(Arc::new(RecordingVmBackend {
                observations: Arc::clone(&self.observations),
            })),
            other => Err(VmError::UnknownBackend(other.to_string())),
        }
    }

    fn visit_selectors(&self, visitor: &mut dyn FnMut(&str)) {
        visitor("recording:capture");
        visitor("capture");
    }
}

pub(super) fn test_package(name: &str, version: &str) -> VmPluginPackage {
    VmPluginPackage {
        manifest: VmPluginManifest {
            name: name.to_string(),
            version: version.to_string(),
            entry: "main".to_string(),
            capabilities: CapabilitySet::default().with("render"),
            management: super::super::VmPluginManagementPolicy::default(),
        },
        zr_vm_project: None,
        bytecode: vec![1, 2, 3],
    }
}

pub(super) fn test_host_context(
    plugin_name: &str,
    backend_selector: &str,
    source: VmPluginPackageSource,
    capabilities: CapabilitySet,
) -> VmPluginHostContext {
    let runtime = CoreRuntime::new();
    let package_root = source.package_root.clone().or_else(|| {
        source
            .manifest_path
            .as_ref()
            .and_then(|path| path.parent().map(Path::to_path_buf))
    });
    let source_root = source.manifest_path.as_ref().and_then(|path| {
        path.parent()
            .map(Path::to_path_buf)
            .or_else(|| package_root.clone())
    });
    let data_root = package_root.as_ref().map(|root| root.join("data"));

    VmPluginHostContext::new(
        PluginContext {
            plugin_name: plugin_name.to_string(),
            core: runtime.handle().downgrade(),
            package_root,
            source_root,
            data_root,
        },
        capabilities,
        backend_selector.to_string(),
        source,
        HostRegistry::default(),
        HostExportRegistry::default(),
        VmHostInterfaceRegistry::default(),
        VmReflectionCatalog::default(),
        Default::default(),
        Arc::new(NoopSlotLifecycle),
    )
}

pub(super) struct PluginFixture {
    pub(super) root: PathBuf,
    pub(super) package_root: PathBuf,
    pub(super) manifest_path: PathBuf,
    pub(super) bytecode_path: PathBuf,
}

impl PluginFixture {
    pub(super) fn new(name: &str, version: &str, backend: &str, bytecode: &[u8]) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zircon-script-fixture-{nonce}"));
        let package_root = root.join(name);
        fs::create_dir_all(&package_root).unwrap();
        fs::create_dir_all(package_root.join("data")).unwrap();

        let manifest_path = package_root.join("plugin.toml");
        let bytecode_path = package_root.join("plugin.bin");
        fs::write(
            &manifest_path,
            format!(
                "name = \"{name}\"\nversion = \"{version}\"\nentry = \"main\"\nbackend = \"{backend}\"\nbytecode = \"plugin.bin\"\n\n[capabilities]\ncapabilities = [\"render\"]\n"
            ),
        )
        .unwrap();
        fs::write(&bytecode_path, bytecode).unwrap();

        Self {
            root,
            package_root,
            manifest_path,
            bytecode_path,
        }
    }
}

impl Drop for PluginFixture {
    fn drop(&mut self) {
        let _ = remove_dir_all_if_exists(&self.root);
    }
}

pub(super) struct ZrVmProjectFixture {
    pub(super) root: PathBuf,
    pub(super) project_path: PathBuf,
}

impl ZrVmProjectFixture {
    pub(super) fn new(name: &str, version: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zircon-zr-vm-project-fixture-{nonce}"));
        let package_root = root.join(name);
        let project_root = package_root.join("script");
        fs::create_dir_all(project_root.join("src")).unwrap();
        let manifest_path = package_root.join("plugin.toml");
        let project_path = project_root.join("plugin.zrp");
        fs::write(&project_path, "name = \"sample_zr\"\n").unwrap();
        fs::write(project_root.join("src").join("main.zr"), "return 1;\n").unwrap();
        fs::write(
            &manifest_path,
            format!(
                "name = \"{name}\"\nversion = \"{version}\"\nentry = \"main\"\nbackend = \"zr_vm:project\"\n\n[capabilities]\ncapabilities = [\"foundation.time\"]\n\n[zr_vm]\nproject = \"script/plugin.zrp\"\nentry_module = \"main\"\nexecution_mode = \"binary\"\n"
            ),
        )
        .unwrap();

        Self { root, project_path }
    }
}

impl Drop for ZrVmProjectFixture {
    fn drop(&mut self) {
        let _ = remove_dir_all_if_exists(&self.root);
    }
}

fn remove_dir_all_if_exists(path: &Path) -> Result<(), std::io::Error> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}
