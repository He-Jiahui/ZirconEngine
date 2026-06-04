use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ZR_VM_PROJECT_BACKEND_SELECTOR;

pub(super) fn build_real_backend_host(
    manager: &Arc<zircon_runtime::script::VmPluginManager>,
    package: &zircon_runtime::script::DiscoveredVmPluginPackage,
) -> zircon_runtime::script::VmPluginHostContext {
    let source = package.source.clone();
    let package_root = source.package_root.clone().or_else(|| {
        source
            .manifest_path
            .as_ref()
            .and_then(|path| path.parent().map(Path::to_path_buf))
    });
    let mut plugin = manager.base_plugin_context().clone();
    plugin.package_root = package_root.clone();
    plugin.source_root = source
        .manifest_path
        .as_ref()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| package_root.clone());
    plugin.data_root = package_root.as_ref().map(|root| root.join("data"));

    zircon_runtime::script::VmPluginHostContext {
        plugin,
        capabilities: package.package.manifest.capabilities.clone(),
        backend_selector: ZR_VM_PROJECT_BACKEND_SELECTOR.to_string(),
        package_source: source,
        host_registry: manager.host_registry(),
        host_exports: manager.host_exports(),
        slot_lifecycle: Arc::new(NoopSlotLifecycle),
    }
}

struct NoopSlotLifecycle;

impl zircon_runtime::script::VmPluginSlotLifecycle for NoopSlotLifecycle {
    fn load_package(
        &self,
        _backend_selector: &str,
        _package: zircon_runtime::script::VmPluginPackage,
    ) -> Result<zircon_runtime::script::PluginSlotId, zircon_runtime::script::VmError> {
        Err(zircon_runtime::script::VmError::Operation(
            "test lifecycle facade does not load slots".to_string(),
        ))
    }

    fn hot_reload_slot(
        &self,
        _slot: zircon_runtime::script::PluginSlotId,
        _package: zircon_runtime::script::VmPluginPackage,
    ) -> Result<(), zircon_runtime::script::VmError> {
        Err(zircon_runtime::script::VmError::Operation(
            "test lifecycle facade does not hot reload slots".to_string(),
        ))
    }

    fn unload_slot(
        &self,
        _slot: zircon_runtime::script::PluginSlotId,
    ) -> Result<(), zircon_runtime::script::VmError> {
        Err(zircon_runtime::script::VmError::Operation(
            "test lifecycle facade does not unload slots".to_string(),
        ))
    }

    fn slot(
        &self,
        slot: zircon_runtime::script::PluginSlotId,
    ) -> Result<zircon_runtime::script::VmPluginSlotRecord, zircon_runtime::script::VmError> {
        Err(zircon_runtime::script::VmError::MissingSlot(slot.get()))
    }

    fn list_slots(&self) -> Vec<zircon_runtime::script::VmPluginSlotRecord> {
        Vec::new()
    }
}

pub(super) struct ZrVmProjectFixture {
    pub(super) root: PathBuf,
    pub(super) project_path: PathBuf,
}

pub(super) struct DocumentedZrVmExampleFixture {
    pub(super) root: PathBuf,
}

impl ZrVmProjectFixture {
    pub(super) fn new(name: &str, version: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zircon-zr-vm-real-fixture-{nonce}"));
        let package_root = root.join(name);
        let project_root = package_root.join("script");
        let source_root = project_root.join("src");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(project_root.join("bin")).unwrap();
        fs::create_dir_all(package_root.join("data")).unwrap();

        let project_path = project_root.join("plugin.zrp");
        fs::write(
            &project_path,
            concat!(
                "{\n",
                "  \"name\": \"native_host_roundtrip\",\n",
                "  \"source\": \"src\",\n",
                "  \"binary\": \"bin\",\n",
                "  \"entry\": \"main\"\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(source_root.join("main.zr"), zr_vm_source()).unwrap();
        fs::write(
            package_root.join("plugin.toml"),
            format!(
                concat!(
                    "name = \"{name}\"\n",
                    "version = \"{version}\"\n",
                    "entry = \"main\"\n",
                    "backend = \"zr_vm:project\"\n",
                    "\n",
                    "[capabilities]\n",
                    "capabilities = [\"foundation.time\", \"foundation.log\"]\n",
                    "\n",
                    "[zr_vm]\n",
                    "project = \"script/plugin.zrp\"\n",
                    "entry_module = \"main\"\n",
                    "execution_mode = \"binary\"\n",
                ),
                name = name,
                version = version,
            ),
        )
        .unwrap();

        Self { root, project_path }
    }
}

impl DocumentedZrVmExampleFixture {
    pub(super) fn copy_from_docs() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zircon-zr-vm-docs-example-{nonce}"));
        let package_root = root.join("zr_vm_minimal");
        fs::create_dir_all(&package_root).unwrap();

        let docs_example = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../docs/zircon_runtime/script/vm/examples/zr_vm_minimal");
        for file_name in ["plugin.toml", "plugin.zrp", "main.zr"] {
            fs::copy(docs_example.join(file_name), package_root.join(file_name)).unwrap();
        }

        Self { root }
    }
}

impl Drop for ZrVmProjectFixture {
    fn drop(&mut self) {
        remove_dir_all_if_exists(&self.root);
    }
}

impl Drop for DocumentedZrVmExampleFixture {
    fn drop(&mut self) {
        remove_dir_all_if_exists(&self.root);
    }
}

fn zr_vm_source() -> &'static str {
    concat!(
        "var math = %import(\"zr.zircon.math\");\n",
        "var foundation = %import(\"zr.zircon.foundation\");\n",
        "var savedState = \"created\";\n",
        "\n",
        "pub activate(): void {\n",
        "    var now = foundation.time_unix_millis();\n",
        "    var dot = math.vec3_dot(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);\n",
        "    foundation.log_info(\"activated\");\n",
        "    savedState = \"activated\";\n",
        "}\n",
        "\n",
        "pub deactivate(): void {\n",
        "    savedState = savedState + \":deactivated\";\n",
        "}\n",
        "\n",
        "pub saveState(): string {\n",
        "    return savedState;\n",
        "}\n",
        "\n",
        "pub restoreState(state: string): void {\n",
        "    savedState = state + \":restored\";\n",
        "}\n",
        "\n",
        "return 0;\n",
    )
}

fn remove_dir_all_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}
