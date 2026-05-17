use std::sync::Arc;

use zircon_runtime::script::VmPluginManager;

mod backend;
mod module;
#[cfg(feature = "real-zr-vm")]
mod real_backend;

pub use backend::{ZrVmBackend, ZrVmBackendFamily};
pub use module::{
    module_descriptor, ZrVmLanguageBackendRegistration, ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME,
    ZR_VM_LANGUAGE_MODULE_NAME,
};

pub const PLUGIN_ID: &str = "zr_vm_language";
pub const ZR_VM_PROJECT_BACKEND_SELECTOR: &str = "zr_vm:project";

#[derive(Clone, Debug)]
pub struct ZrVmLanguageRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl ZrVmLanguageRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for ZrVmLanguageRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_runtime::plugin::RuntimePlugin for ZrVmLanguageRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn register_zr_vm_backend(manager: &VmPluginManager) -> String {
    manager.register_family(Arc::new(ZrVmBackendFamily))
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "ZrVM Language",
        zircon_runtime::RuntimePluginId::ZrVmLanguage,
        "zircon_plugin_zr_vm_language_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::ServerRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_enabled_by_default(false)
    .with_capability("runtime.plugin.zr_vm_language")
    .with_capability("runtime.script.backend.zr_vm_project")
}

pub fn runtime_plugin() -> ZrVmLanguageRuntimePlugin {
    ZrVmLanguageRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        "runtime.plugin.zr_vm_language",
        "runtime.script.backend.zr_vm_project",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zr_vm_language_registration_reports_backend_capability() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == ZR_VM_LANGUAGE_MODULE_NAME));
        assert!(report
            .package_manifest
            .capabilities
            .contains(&"runtime.script.backend.zr_vm_project".to_string()));
    }

    #[test]
    fn zr_vm_backend_family_resolves_project_selector() {
        let manager = zircon_runtime::script::VmPluginManager::mock();
        register_zr_vm_backend(&manager);

        assert!(manager
            .backend_names()
            .contains(&ZR_VM_PROJECT_BACKEND_SELECTOR.to_string()));
        manager
            .select_default_backend(ZR_VM_PROJECT_BACKEND_SELECTOR)
            .unwrap();
        assert_eq!(
            manager.selected_backend_name(),
            ZR_VM_PROJECT_BACKEND_SELECTOR
        );
    }

    #[test]
    fn zr_vm_runtime_module_registers_backend_with_vm_manager() {
        let runtime = zircon_runtime::core::CoreRuntime::new();
        runtime
            .register_module(zircon_runtime::script::module_descriptor())
            .unwrap();
        runtime.register_module(module_descriptor()).unwrap();
        runtime
            .activate_module(zircon_runtime::script::SCRIPT_MODULE_NAME)
            .unwrap();
        runtime.activate_module(ZR_VM_LANGUAGE_MODULE_NAME).unwrap();

        let registration = runtime
            .handle()
            .resolve_plugin::<ZrVmLanguageBackendRegistration>(
                ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME,
            )
            .unwrap();
        let manager = runtime
            .handle()
            .resolve_manager::<zircon_runtime::script::VmPluginManager>(
                zircon_runtime::script::VM_PLUGIN_MANAGER_NAME,
            )
            .unwrap();

        assert_eq!(registration.selector, "zr_vm");
        assert!(manager
            .backend_names()
            .contains(&ZR_VM_PROJECT_BACKEND_SELECTOR.to_string()));
    }
}

#[cfg(all(test, feature = "real-zr-vm"))]
mod real_zr_vm_tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn real_backend_loads_native_host_modules_and_roundtrips_lifecycle() {
        let fixture = ZrVmProjectFixture::new("native_host_roundtrip", "0.1.0");
        let manager = zircon_runtime::script::VmPluginManager::mock();
        register_zr_vm_backend(&manager);

        let packages = manager
            .discover_packages(&fixture.root)
            .expect("discover zr_vm package");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].backend_name, ZR_VM_PROJECT_BACKEND_SELECTOR);

        let slot = manager
            .load_discovered_package(&packages[0])
            .expect("load and activate zr_vm package");
        let record = manager.slot(slot).expect("loaded slot record");
        assert_eq!(record.backend_name, ZR_VM_PROJECT_BACKEND_SELECTOR);
        assert_eq!(
            record.source.zr_vm_project_path,
            Some(fixture.project_path.clone())
        );

        manager
            .hot_reload_discovered_slot(slot, &packages[0])
            .expect("save, reload, restore, and reactivate zr_vm package");
        manager
            .unload_slot(slot)
            .expect("deactivate and unload slot");
        assert!(manager.list_slots().is_empty());
    }

    #[test]
    fn real_backend_session_preserves_lifecycle_state() {
        let fixture = ZrVmProjectFixture::new("native_host_session_state", "0.1.0");
        let manager = zircon_runtime::script::VmPluginManager::mock();
        let packages = manager
            .discover_packages(&fixture.root)
            .expect("discover zr_vm package");
        let host = build_real_backend_host(&manager, &packages[0]);
        let mut instance = super::real_backend::load_project_package(&packages[0].package, &host)
            .expect("load zr_vm package instance");

        instance
            .activate(&host)
            .expect("activate persistent session");
        let activated = instance
            .save_state()
            .expect("save state after activate")
            .bytes;
        assert_eq!(String::from_utf8(activated).unwrap(), "activated");

        instance
            .restore_state(&zircon_runtime::script::VmStateBlob {
                bytes: b"hot".to_vec(),
            })
            .expect("restore state in persistent session");
        let restored = instance
            .save_state()
            .expect("save state after restore")
            .bytes;
        assert_eq!(String::from_utf8(restored).unwrap(), "hot:restored");
    }

    #[test]
    fn real_backend_loads_documented_minimal_example() {
        let fixture = DocumentedZrVmExampleFixture::copy_from_docs();
        let manager = zircon_runtime::script::VmPluginManager::mock();
        let packages = manager
            .discover_packages(&fixture.root)
            .expect("discover documented zr_vm example package");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].backend_name, ZR_VM_PROJECT_BACKEND_SELECTOR);

        let host = build_real_backend_host(&manager, &packages[0]);
        let mut instance = super::real_backend::load_project_package(&packages[0].package, &host)
            .expect("load documented zr_vm example");

        instance
            .activate(&host)
            .expect("activate documented example");
        let activated = instance
            .save_state()
            .expect("save documented example state")
            .bytes;
        assert_eq!(String::from_utf8(activated).unwrap(), "activated");

        instance
            .restore_state(&zircon_runtime::script::VmStateBlob {
                bytes: b"docs".to_vec(),
            })
            .expect("restore documented example state");
        let restored = instance
            .save_state()
            .expect("save restored documented example state")
            .bytes;
        assert_eq!(String::from_utf8(restored).unwrap(), "docs:restored");
        instance
            .deactivate()
            .expect("deactivate documented example");
    }

    fn build_real_backend_host(
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
        ) -> Result<zircon_runtime::script::VmPluginSlotRecord, zircon_runtime::script::VmError>
        {
            Err(zircon_runtime::script::VmError::MissingSlot(slot.get()))
        }

        fn list_slots(&self) -> Vec<zircon_runtime::script::VmPluginSlotRecord> {
            Vec::new()
        }
    }

    struct ZrVmProjectFixture {
        root: PathBuf,
        project_path: PathBuf,
    }

    struct DocumentedZrVmExampleFixture {
        root: PathBuf,
    }

    impl ZrVmProjectFixture {
        fn new(name: &str, version: &str) -> Self {
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
        fn copy_from_docs() -> Self {
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
}
