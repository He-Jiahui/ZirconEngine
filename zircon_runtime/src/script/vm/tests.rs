#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::super::{
        backend::MockVmBackend, module_descriptor, BuiltinVmBackendFamily, CapabilitySet,
        HostExportFunction, HostExportRegistry, HostRegistry, HotReloadCoordinator,
        PluginHostDriver, UnavailableVmBackend, VmBackend, VmBackendFamily, VmError,
        VmPluginHostContext, VmPluginInstance, VmPluginManager, VmPluginManifest, VmPluginPackage,
        VmPluginPackageSource, VmPluginSlotLifecycle, VmPluginSlotRecord, PLUGIN_HOST_DRIVER_NAME,
        SCRIPT_MODULE_NAME, VM_PLUGIN_MANAGER_NAME, VM_PLUGIN_RUNTIME_NAME,
    };
    use crate::core::framework::script::{
        ScriptHostFunctionDescriptor, ScriptHostModuleDescriptor, ScriptHostParameterDescriptor,
        ScriptHostPrototypeKind, ScriptHostTypeDescriptor, ScriptHostTypeRef, ScriptHostValue,
        ScriptHostValueKind, ZirconScriptType,
    };
    use crate::core::{CoreRuntime, PluginContext};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ObservedHostContext {
        plugin_name: String,
        backend_selector: String,
        package_root: Option<PathBuf>,
        source_root: Option<PathBuf>,
        data_root: Option<PathBuf>,
        package_source: VmPluginPackageSource,
        capabilities: CapabilitySet,
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
    struct RecordingVmBackendFamily {
        observations: Arc<Mutex<Vec<ObservedHostContext>>>,
    }

    impl RecordingVmBackendFamily {
        fn new(observations: Arc<Mutex<Vec<ObservedHostContext>>>) -> Self {
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

        fn selectors(&self) -> Vec<String> {
            vec!["recording:capture".to_string(), "capture".to_string()]
        }
    }

    #[test]
    fn host_handles_are_stable_and_valid() {
        let registry = HostRegistry::default();
        let handle = registry.register_capability("RenderingManager");
        assert!(registry.is_valid(handle));
    }

    #[test]
    fn host_registry_exposes_stable_capability_records_without_concrete_objects() {
        let registry = HostRegistry::default();
        let ui_shell = registry.register_capability("editor.host.ui_shell");
        let asset_core = registry.register_capability("editor.host.asset_core");

        let ui_record = registry.capability(ui_shell).unwrap();
        assert_eq!(ui_record.handle, ui_shell);
        assert_eq!(ui_record.label, "editor.host.ui_shell");

        let records = registry.capabilities();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].handle, ui_shell);
        assert_eq!(records[1].handle, asset_core);
        assert!(records.iter().all(|record| !record.label.is_empty()));
        assert!(registry
            .capability(super::super::HostHandle::new(999))
            .is_none());
    }

    #[test]
    fn host_export_registry_validates_descriptors_and_dispatches_callbacks() {
        let registry = HostRegistry::default();
        let exports = HostExportRegistry::new(registry.clone());
        let descriptor = ScriptHostModuleDescriptor::new("test.host", "0.1.0")
            .with_capability("test.add")
            .with_function(
                ScriptHostFunctionDescriptor::new("add", 2, 2, ScriptHostValueKind::Int)
                    .with_parameter(ScriptHostParameterDescriptor::new(
                        "left",
                        ScriptHostValueKind::Int,
                    ))
                    .with_parameter(ScriptHostParameterDescriptor::new(
                        "right",
                        ScriptHostValueKind::Int,
                    ))
                    .with_required_capability("test.add"),
            );
        let handle = exports
            .register_module(
                descriptor,
                [HostExportFunction::new("add", |context| {
                    let left = match context.arguments[0] {
                        ScriptHostValue::Int(value) => value,
                        _ => 0,
                    };
                    let right = match context.arguments[1] {
                        ScriptHostValue::Int(value) => value,
                        _ => 0,
                    };
                    Ok(ScriptHostValue::Int(left + right))
                })],
            )
            .unwrap();

        assert!(registry.is_valid(handle));
        assert_eq!(exports.modules().len(), 1);
        let granted = CapabilitySet::default().with("test.add");
        let value = exports
            .call_with_capabilities(
                "test.host",
                "add",
                vec![ScriptHostValue::Int(2), ScriptHostValue::Int(5)],
                &granted,
            )
            .unwrap();
        assert_eq!(value, ScriptHostValue::Int(7));
    }

    #[test]
    fn host_export_registry_preserves_precise_type_refs_for_zr_vm_registration() {
        let exports = HostExportRegistry::default();
        let descriptor = ScriptHostModuleDescriptor::new("test.types", "0.1.0")
            .with_type(
                ScriptHostTypeDescriptor::new("Vec3", ScriptHostValueKind::Float)
                    .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "Vec3"))
                    .with_prototype_kind(ScriptHostPrototypeKind::Struct)
                    .allow_value_construction(true),
            )
            .with_function(
                ScriptHostFunctionDescriptor::new("identity", 1, 1, ScriptHostValueKind::Float)
                    .with_return_type(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "Vec3"))
                    .with_parameter(
                        ScriptHostParameterDescriptor::new("value", ScriptHostValueKind::Float)
                            .with_type_ref(ScriptHostTypeRef::new(ScriptHostValueKind::Float, "Vec3")),
                    ),
            );

        exports
            .register_module(
                descriptor,
                [HostExportFunction::new("identity", |context| {
                    Ok(context.arguments[0].clone())
                })],
            )
            .unwrap();

        let module = exports.module("test.types").unwrap();
        assert_eq!(module.descriptor.types[0].type_ref.type_name, "Vec3");
        assert_eq!(module.descriptor.types[0].prototype_kind, ScriptHostPrototypeKind::Struct);
        assert!(module.descriptor.types[0].allow_value_construction);
        assert_eq!(module.descriptor.functions[0].return_type.type_name, "Vec3");
        assert_eq!(module.descriptor.functions[0].parameters[0].type_ref.type_name, "Vec3");
    }

    #[test]
    fn host_export_registry_rejects_duplicates_invalid_callbacks_and_missing_capabilities() {
        let exports = HostExportRegistry::default();
        let descriptor = ScriptHostModuleDescriptor::new("test.host", "0.1.0")
            .with_capability("test.read")
            .with_function(
                ScriptHostFunctionDescriptor::new("read", 0, 0, ScriptHostValueKind::Null)
                    .with_required_capability("test.read"),
            );

        exports
            .register_module(
                descriptor.clone(),
                [HostExportFunction::new("read", |_| {
                    Ok(ScriptHostValue::Null)
                })],
            )
            .unwrap();
        assert!(matches!(
            exports.register_module(
                descriptor.clone(),
                [HostExportFunction::new("read", |_| Ok(ScriptHostValue::Null))]
            ),
            Err(VmError::Operation(message)) if message.contains("already registered")
        ));
        assert!(matches!(
            HostExportRegistry::default().register_module(
                descriptor.clone(),
                [HostExportFunction::new("unknown", |_| Ok(ScriptHostValue::Null))]
            ),
            Err(VmError::Operation(message)) if message.contains("callback missing")
        ));
        assert!(matches!(
            exports.call_with_capabilities("test.host", "read", Vec::new(), &CapabilitySet::default()),
            Err(VmError::Operation(message)) if message.contains("missing capability")
        ));
        let mismatched = ScriptHostModuleDescriptor::new("test.mismatch", "0.1.0")
            .with_function(
                ScriptHostFunctionDescriptor::new("bad", 0, 0, ScriptHostValueKind::Int)
                    .with_return_type(ScriptHostTypeRef::new(ScriptHostValueKind::String, "string")),
            );
        assert!(matches!(
            HostExportRegistry::default().register_module(
                mismatched,
                [HostExportFunction::new("bad", |_| Ok(ScriptHostValue::Null))]
            ),
            Err(VmError::Operation(message)) if message.contains("value kind mismatch")
        ));
    }

    #[test]
    fn rust_reflection_macros_generate_type_function_and_module_descriptors() {
        #[derive(crate::ZirconScriptType)]
        #[zircon_script(
            name = "TestVec3",
            value_kind = ScriptHostValueKind::Float,
            prototype = ScriptHostPrototypeKind::Struct,
            allow_value_construction = true,
            documentation = "test vector"
        )]
        #[allow(dead_code)]
        struct TestVec3 {
            #[zircon_script(type_name = "float", documentation = "x axis")]
            x: f64,
            #[zircon_script(type_name = "float")]
            y: f64,
            #[zircon_script(type_name = "float")]
            z: f64,
        }

        let type_descriptor = TestVec3::script_host_type_descriptor();
        assert_eq!(type_descriptor.name, "TestVec3");
        assert_eq!(type_descriptor.type_ref.type_name, "TestVec3");
        assert_eq!(type_descriptor.fields[0].type_ref.type_name, "float");
        assert_eq!(type_descriptor.fields[0].documentation.as_deref(), Some("x axis"));
        assert!(type_descriptor.allow_value_construction);

        #[crate::zircon_host_module(
            name = "test.macro.math",
            version = "0.1.0",
            capability = "test.math",
            documentation = "macro math"
        )]
        mod macro_math {
            use super::*;

            #[derive(crate::ZirconScriptType)]
            #[zircon_script(
                name = "Point",
                value_kind = ScriptHostValueKind::Float,
                allow_value_construction = true
            )]
            #[allow(dead_code)]
            struct Point {
                #[zircon_script(type_name = "float")]
                x: f64,
            }

            #[crate::zircon_host_function(
                name = "double",
                return_type_name = "float",
                capability = "test.math",
                documentation = "double input"
            )]
            fn double(value: f64) -> f64 {
                value * 2.0
            }
        }

        let descriptor = macro_math::macro_math_host_module_descriptor();
        assert_eq!(descriptor.name, "test.macro.math");
        assert_eq!(descriptor.capabilities, vec!["test.math".to_string()]);
        assert_eq!(descriptor.types[0].name, "Point");
        assert_eq!(descriptor.functions[0].name, "double");
        assert_eq!(descriptor.functions[0].parameters[0].type_ref.type_name, "f64");
        assert_eq!(descriptor.functions[0].return_type.type_name, "float");

        let exports = HostExportRegistry::default();
        macro_math::register_macro_math_host_module(&exports).unwrap();
        let value = exports
            .call_with_capabilities(
                "test.macro.math",
                "double",
                vec![ScriptHostValue::Float(2.5)],
                &CapabilitySet::default().with("test.math"),
            )
            .unwrap();
        assert_eq!(value, ScriptHostValue::Float(5.0));
    }

    #[test]
    fn builtin_backend_family_accepts_qualified_and_legacy_backend_names() {
        let registry = super::super::VmBackendRegistry::new();
        registry.register_family(Arc::new(BuiltinVmBackendFamily));

        assert!(registry.resolve("builtin:mock").is_ok());
        assert!(registry.resolve("mock").is_ok());
        assert!(registry.resolve("builtin:unavailable").is_ok());
        assert!(registry.resolve("unavailable").is_ok());
    }

    #[test]
    fn hot_reload_coordinator_tracks_slot_lifecycle_records() {
        let coordinator = HotReloadCoordinator::new();
        let package_root = std::env::temp_dir().join("zircon-script-slot-lifecycle");
        let source = VmPluginPackageSource {
            package_root: Some(package_root.clone()),
            manifest_path: Some(package_root.join("plugin.toml")),
            bytecode_path: Some(package_root.join("plugin.bin")),
            zr_vm_project_path: None,
        };
        let package = test_package("sample", "0.1.0");
        let host = test_host_context(
            VM_PLUGIN_RUNTIME_NAME,
            "mock",
            source.clone(),
            package.manifest.capabilities.clone(),
        );

        let slot = coordinator
            .load_package("mock", &MockVmBackend, package, &host)
            .unwrap();
        let initial = coordinator.slot(slot).unwrap();
        assert_eq!(initial.backend_name, "mock");
        assert_eq!(initial.source, source);
        assert_eq!(initial.manifest.version, "0.1.0");

        coordinator
            .hot_reload(
                slot,
                "mock",
                &MockVmBackend,
                test_package("sample", "0.2.0"),
                &host,
            )
            .unwrap();

        let reloaded = coordinator.slot(slot).unwrap();
        assert_eq!(reloaded.manifest.version, "0.2.0");
        assert_eq!(coordinator.list_slots(), vec![reloaded.clone()]);

        let unloaded = coordinator.unload_slot(slot).unwrap();
        assert_eq!(unloaded.version, "0.2.0");
        assert!(matches!(
            coordinator.slot(slot),
            Err(VmError::MissingSlot(missing)) if missing == slot.get()
        ));
    }

    #[test]
    fn vm_plugin_manager_discovers_packages_selects_backends_and_loads_slots() {
        let fixture = PluginFixture::new("sample", "0.1.0", "mock", &[1, 2, 3]);
        let manager = VmPluginManager::with_builtin_backends(HostRegistry::default());
        let packages = manager.discover_packages(&fixture.root).unwrap();

        assert_eq!(packages.len(), 1);
        assert!(manager.backend_names().contains(&"mock".to_string()));
        assert!(manager.backend_names().contains(&"unavailable".to_string()));

        let discovered = &packages[0];
        assert_eq!(discovered.backend_name, "mock");
        assert_eq!(discovered.package.manifest.name, "sample");
        assert_eq!(
            discovered.source.manifest_path.as_deref(),
            Some(fixture.manifest_path.as_path())
        );
        assert_eq!(
            discovered.source.bytecode_path.as_deref(),
            Some(fixture.bytecode_path.as_path())
        );
        assert!(discovered.source.zr_vm_project_path.is_none());

        let slot = manager.load_discovered_package(discovered).unwrap();
        let loaded = manager.slot(slot).unwrap();
        assert_eq!(loaded.backend_name, "mock");
        assert_eq!(loaded.manifest.version, "0.1.0");
        assert_eq!(loaded.source, discovered.source);

        manager.select_default_backend("unavailable").unwrap();
        manager
            .hot_reload_slot(slot, test_package("sample", "0.2.0"))
            .unwrap();
        let reloaded = manager.slot(slot).unwrap();
        assert_eq!(reloaded.backend_name, "mock");
        assert_eq!(reloaded.manifest.version, "0.2.0");

        manager.unload_slot(slot).unwrap();
        assert!(manager.list_slots().is_empty());
    }

    #[test]
    fn unavailable_backend_reports_error() {
        let backend = UnavailableVmBackend;
        let source = VmPluginPackageSource::default();
        let package = test_package("sample", "0.1.0");
        let host = test_host_context(
            VM_PLUGIN_RUNTIME_NAME,
            "builtin:unavailable",
            source,
            package.manifest.capabilities.clone(),
        );
        let error = match backend.load_package(&package, &host) {
            Ok(_) => panic!("expected unavailable backend to reject package"),
            Err(error) => error,
        };
        assert!(matches!(error, VmError::BackendUnavailable(_)));
    }

    #[test]
    fn script_module_descriptor_registers_vm_plugin_runtime_before_manager_facade() {
        let descriptor = module_descriptor();

        let plugin = descriptor
            .plugins
            .iter()
            .find(|plugin| plugin.name.as_str() == VM_PLUGIN_RUNTIME_NAME)
            .expect("vm plugin runtime descriptor");
        assert_eq!(plugin.startup_mode, crate::core::StartupMode::Immediate);
        assert!(plugin
            .dependencies
            .iter()
            .any(|dependency| dependency.name.as_str() == PLUGIN_HOST_DRIVER_NAME));

        let manager = descriptor
            .managers
            .iter()
            .find(|manager| manager.name.as_str() == VM_PLUGIN_MANAGER_NAME)
            .expect("vm plugin manager descriptor");
        assert!(manager
            .dependencies
            .iter()
            .any(|dependency| dependency.name.as_str() == VM_PLUGIN_RUNTIME_NAME));
    }

    #[test]
    fn core_resolve_plugin_exposes_vm_plugin_runtime_and_manager_facade_shares_it() {
        let runtime = CoreRuntime::new();
        let core = runtime.handle();
        core.register_module(module_descriptor())
            .expect("register script module");
        core.activate_module(SCRIPT_MODULE_NAME)
            .expect("activate script module");

        let plugin = core
            .resolve_plugin::<VmPluginManager>(VM_PLUGIN_RUNTIME_NAME)
            .expect("resolve vm plugin runtime");
        let manager = core
            .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
            .expect("resolve vm plugin manager facade");
        let driver = core
            .resolve_driver::<PluginHostDriver>(PLUGIN_HOST_DRIVER_NAME)
            .expect("resolve plugin host driver");

        assert!(Arc::ptr_eq(&plugin, &manager));

        let capability = driver.registry().register_capability("RenderingManager");
        assert!(plugin.host_registry().is_valid(capability));
        assert!(driver.host_exports().module("zr.zircon.math").is_some());
        assert!(plugin
            .host_exports()
            .module("zr.zircon.foundation")
            .is_some());
        assert_eq!(
            plugin.base_plugin_context().plugin_name,
            VM_PLUGIN_RUNTIME_NAME
        );

        plugin.select_default_backend("builtin:mock").unwrap();
        let slot = plugin.load_package(test_package("core", "0.1.0")).unwrap();
        assert_eq!(plugin.slot(slot).unwrap().backend_name, "builtin:mock");
    }

    #[test]
    fn vm_plugin_manager_propagates_host_context_roots_and_backend_selector() {
        let fixture = PluginFixture::new("sample", "0.1.0", "recording:capture", &[1, 2, 3]);
        let observations = Arc::new(Mutex::new(Vec::<ObservedHostContext>::new()));
        let runtime = CoreRuntime::new();
        let base_plugin_context = PluginContext {
            plugin_name: VM_PLUGIN_RUNTIME_NAME.to_string(),
            core: runtime.handle().downgrade(),
            package_root: None,
            source_root: None,
            data_root: None,
        };
        let manager =
            VmPluginManager::with_plugin_context(base_plugin_context, HostRegistry::default());
        manager.register_family(Arc::new(RecordingVmBackendFamily::new(Arc::clone(
            &observations,
        ))));

        let packages = manager.discover_packages(&fixture.root).unwrap();
        let slot = manager.load_discovered_package(&packages[0]).unwrap();
        let record = manager.slot(slot).unwrap();
        let expected_data_root = fixture.package_root.join("data");

        assert_eq!(record.backend_name, "recording:capture");
        assert_eq!(
            record.source.manifest_path.as_deref(),
            Some(fixture.manifest_path.as_path())
        );
        assert_eq!(
            record.source.bytecode_path.as_deref(),
            Some(fixture.bytecode_path.as_path())
        );
        assert!(record.source.zr_vm_project_path.is_none());

        let observed = observations.lock().unwrap().clone();
        assert_eq!(observed.len(), 2);
        for host in observed {
            assert_eq!(host.plugin_name, VM_PLUGIN_RUNTIME_NAME);
            assert_eq!(host.backend_selector, "recording:capture");
            assert_eq!(
                host.package_root.as_deref(),
                Some(fixture.package_root.as_path())
            );
            assert_eq!(
                host.source_root.as_deref(),
                Some(fixture.package_root.as_path())
            );
            assert_eq!(
                host.data_root.as_deref(),
                Some(expected_data_root.as_path())
            );
            assert_eq!(host.package_source, record.source);
            assert_eq!(host.capabilities, record.manifest.capabilities);
        }
    }

    #[test]
    fn vm_plugin_discovery_supports_zr_vm_project_packages_without_bytecode() {
        let fixture = ZrVmProjectFixture::new("sample_zr", "0.1.0");
        let manager = VmPluginManager::with_builtin_backends(HostRegistry::default());
        let packages = manager.discover_packages(&fixture.root).unwrap();

        assert_eq!(packages.len(), 1);
        let discovered = &packages[0];
        assert_eq!(discovered.backend_name, "zr_vm:project");
        assert!(discovered.package.bytecode.is_empty());
        assert_eq!(
            discovered.source.zr_vm_project_path.as_deref(),
            Some(fixture.project_path.as_path())
        );
        assert_eq!(
            discovered
                .package
                .zr_vm_project
                .as_ref()
                .unwrap()
                .entry_module,
            "main"
        );
        assert_eq!(
            discovered
                .package
                .zr_vm_project
                .as_ref()
                .unwrap()
                .project_path,
            fixture.project_path
        );
        assert!(discovered.source.bytecode_path.is_none());
    }

    #[test]
    fn vm_plugin_protocol_types_live_in_script_subsystem() {
        let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let manager_root = runtime_root.join("src/core/manager");
        let script_mod_source = include_str!("../mod.rs");
        let vm_mod_source = include_str!("mod.rs");
        let manifest_source = include_str!("plugin/vm_plugin_manifest.rs");
        let host_registry_source = include_str!("host/host_registry.rs");
        let package_discovery_source = include_str!("plugin/vm_plugin_package_discovery.rs");
        let hot_reload_source = include_str!("runtime/hot_reload_coordinator.rs");
        let manager_mod_source = include_str!("../../core/manager/mod.rs");
        let manager_resolver_source = include_str!("../../core/manager/resolver.rs");
        let manager_records_root = manager_root.join("records");

        for required in ["CapabilitySet", "HostHandle", "PluginSlotId"] {
            assert!(
                script_mod_source.contains(required) || vm_mod_source.contains(required),
                "zircon_runtime::script should publicly export {required}"
            );
        }

        for source in [
            manifest_source,
            host_registry_source,
            package_discovery_source,
            hot_reload_source,
        ] {
            assert!(
                !source.contains("use crate::core::manager::"),
                "vm runtime files should not source script protocol types from zircon_manager"
            );
        }

        for forbidden in ["CapabilitySet", "HostHandle", "PluginSlotId"] {
            assert!(
                !manager_mod_source.contains(forbidden),
                "core manager mod.rs should not re-export {forbidden} after vm plugin boundary cleanup"
            );
            assert!(
                !manager_resolver_source.contains(forbidden),
                "core manager resolver should not re-export {forbidden} after vm plugin boundary cleanup"
            );
        }
        assert!(
            !runtime_root.join("src/manager").exists(),
            "runtime root should not keep a legacy manager module after vm plugin boundary cleanup"
        );
        assert!(
            !manager_records_root.exists(),
            "core manager should not grow a records subtree after vm plugin boundary cleanup"
        );
    }

    #[test]
    fn vm_subsystem_is_grouped_by_module_backend_host_plugin_and_runtime() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("script")
            .join("vm");

        for relative in [
            "module/mod.rs",
            "module/script_module.rs",
            "module/module_descriptor.rs",
            "backend/mod.rs",
            "backend/backend_registry.rs",
            "backend/builtin_vm_backend_family.rs",
            "backend/vm_backend.rs",
            "backend/vm_backend_family.rs",
            "backend/unavailable_vm_backend.rs",
            "backend/mock_vm_backend.rs",
            "backend/vm_error.rs",
            "host/mod.rs",
            "host/builtin_host_modules.rs",
            "host/host_export_registry.rs",
            "host/host_registry.rs",
            "host/plugin_host_driver.rs",
            "host/constants.rs",
            "host/vm_plugin_host_context.rs",
            "host/vm_plugin_slot_lifecycle.rs",
            "plugin/mod.rs",
            "plugin/vm_plugin_manifest.rs",
            "plugin/vm_plugin_package.rs",
            "plugin/vm_plugin_package_source.rs",
            "plugin/vm_plugin_package_discovery.rs",
            "plugin/vm_plugin_instance.rs",
            "plugin/vm_state_blob.rs",
            "runtime/mod.rs",
            "runtime/hot_reload_coordinator.rs",
            "runtime/vm_plugin_slot_record.rs",
            "runtime/vm_plugin_manager.rs",
        ] {
            assert!(
                root.join(relative).exists(),
                "expected vm module {relative} under {:?}",
                root
            );
        }
    }

    fn test_package(name: &str, version: &str) -> VmPluginPackage {
        VmPluginPackage {
            manifest: VmPluginManifest {
                name: name.to_string(),
                version: version.to_string(),
                entry: "main".to_string(),
                capabilities: CapabilitySet::default().with("render"),
            },
            zr_vm_project: None,
            bytecode: vec![1, 2, 3],
        }
    }

    fn test_host_context(
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

        VmPluginHostContext {
            plugin: PluginContext {
                plugin_name: plugin_name.to_string(),
                core: runtime.handle().downgrade(),
                package_root,
                source_root,
                data_root,
            },
            capabilities,
            backend_selector: backend_selector.to_string(),
            package_source: source,
            host_registry: HostRegistry::default(),
            host_exports: HostExportRegistry::default(),
            slot_lifecycle: Arc::new(NoopSlotLifecycle),
        }
    }

    struct PluginFixture {
        root: PathBuf,
        package_root: PathBuf,
        manifest_path: PathBuf,
        bytecode_path: PathBuf,
    }

    impl PluginFixture {
        fn new(name: &str, version: &str, backend: &str, bytecode: &[u8]) -> Self {
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

    struct ZrVmProjectFixture {
        root: PathBuf,
        project_path: PathBuf,
    }

    impl ZrVmProjectFixture {
        fn new(name: &str, version: &str) -> Self {
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
}
