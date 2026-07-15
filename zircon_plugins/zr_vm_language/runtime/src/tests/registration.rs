use std::sync::{Arc, Mutex};

use crate::{
    module_descriptor, package_manifest, plugin_registration, register_zr_vm_backend,
    ZrVmLanguageBackendRegistration, ZR_VM_BEHAVIOR_BRIDGE_BIND_SYSTEM, ZR_VM_GC_STEP_SYSTEM,
    ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME, ZR_VM_LANGUAGE_DIST_CRATE_NAME,
    ZR_VM_LANGUAGE_DIST_RUNTIME_ENTRY, ZR_VM_LANGUAGE_MODULE_NAME, ZR_VM_PROJECT_BACKEND_SELECTOR,
};

#[derive(Debug)]
struct BudgetRecordingFamily {
    calls: Arc<Mutex<Vec<(zircon_runtime::script::PluginSlotId, u64)>>>,
}

impl zircon_runtime::script::VmBackendFamily for BudgetRecordingFamily {
    fn family_name(&self) -> &str {
        "budget-recording"
    }

    fn resolve(
        &self,
        selector: &str,
    ) -> Result<Arc<dyn zircon_runtime::script::VmBackend>, zircon_runtime::script::VmError> {
        if selector != "budget-recording:test" {
            return Err(zircon_runtime::script::VmError::UnknownBackend(
                selector.to_string(),
            ));
        }
        Ok(Arc::new(BudgetRecordingBackend {
            calls: Arc::clone(&self.calls),
        }))
    }

    fn selectors(&self) -> Vec<String> {
        vec!["budget-recording:test".to_string()]
    }
}

#[derive(Debug)]
struct BudgetRecordingBackend {
    calls: Arc<Mutex<Vec<(zircon_runtime::script::PluginSlotId, u64)>>>,
}

impl zircon_runtime::script::VmBackend for BudgetRecordingBackend {
    fn backend_name(&self) -> &str {
        "budget-recording:test"
    }

    fn load_package(
        &self,
        package: &zircon_runtime::script::VmPluginPackage,
        host: &zircon_runtime::script::VmPluginHostContext,
    ) -> Result<Box<dyn zircon_runtime::script::VmPluginInstance>, zircon_runtime::script::VmError>
    {
        let (slot, _) = host.vm_owner().ok_or_else(|| {
            zircon_runtime::script::VmError::Operation(
                "budget recording backend requires a slot owner".to_string(),
            )
        })?;
        let pause_micros = package
            .manifest
            .version
            .parse::<u64>()
            .map_err(|error| zircon_runtime::script::VmError::Operation(error.to_string()))?;
        Ok(Box::new(BudgetRecordingInstance {
            manifest: package.manifest.clone(),
            slot,
            pause_micros,
            calls: Arc::clone(&self.calls),
        }))
    }
}

#[derive(Debug)]
struct BudgetRecordingInstance {
    manifest: zircon_runtime::script::VmPluginManifest,
    slot: zircon_runtime::script::PluginSlotId,
    pause_micros: u64,
    calls: Arc<Mutex<Vec<(zircon_runtime::script::PluginSlotId, u64)>>>,
}

impl zircon_runtime::script::VmPluginInstance for BudgetRecordingInstance {
    fn manifest(&self) -> &zircon_runtime::script::VmPluginManifest {
        &self.manifest
    }

    fn gc_step(
        &mut self,
        budget: zircon_runtime::script::VmGcBudget,
    ) -> Result<zircon_runtime::script::VmGcStepOutcome, zircon_runtime::script::VmError> {
        self.calls
            .lock()
            .unwrap()
            .push((self.slot, budget.max_micros_per_frame));
        Ok(zircon_runtime::script::VmGcStepOutcome {
            pause_micros: self.pause_micros,
            root_count: 1,
            cross_boundary_reference_count: 1,
        })
    }
}

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
        .extensions
        .frozen_bridge_table()
        .resolve_slot(zircon_runtime::core::framework::script::SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID)
        .is_some());
    assert!(report
        .package_manifest
        .provides_interfaces
        .iter()
        .any(|interface| interface.id
            == zircon_runtime::core::framework::script::SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID
            && interface.methods.is_empty()));
    assert!(report.extensions.scene_hooks().iter().any(|hook| {
        hook.descriptor().id == "zr_vm_language.script.scene.fixed_update"
            && hook.descriptor().stage == zircon_runtime::scene::SystemStage::FixedUpdate
    }));
    assert!(report.extensions.scene_hooks().iter().any(|hook| {
        hook.descriptor().id == "zr_vm_language.script.scene.update"
            && hook.descriptor().stage == zircon_runtime::scene::SystemStage::Update
    }));
    assert!(report
        .package_manifest
        .capabilities
        .contains(&"runtime.script.backend.zr_vm_project".to_string()));
    assert_eq!(report.package_manifest.category, "runtime");
    assert_eq!(
        report.package_manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Experimental
    );
    for capability in [
        "runtime.plugin.zr_vm_language",
        "runtime.script.backend.zr_vm_project",
    ] {
        assert!(report
            .package_manifest
            .capabilities
            .contains(&capability.to_string()));
        assert!(report
            .package_manifest
            .capability_statuses
            .iter()
            .any(|status| {
                status.capability == capability
                    && status.status == zircon_runtime::plugin::CapabilityStatus::Partial
            }));
    }
}

#[test]
fn zr_vm_language_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));
    assert!(manifest.modules.iter().any(|module| {
        module.name == "zr_vm_language.dist"
            && module.kind == zircon_runtime::plugin::PluginModuleKind::Native
            && module.crate_name == ZR_VM_LANGUAGE_DIST_CRATE_NAME
    }));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("zr_vm_language package manifest declares distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, ZR_VM_LANGUAGE_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(
        distribution.runtime_entry,
        ZR_VM_LANGUAGE_DIST_RUNTIME_ENTRY
    );
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
        .register_module(zircon_runtime::scene::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::script::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime
        .activate_module(zircon_runtime::scene::SCENE_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(zircon_runtime::script::SCRIPT_MODULE_NAME)
        .unwrap();
    runtime.activate_module(ZR_VM_LANGUAGE_MODULE_NAME).unwrap();

    let registration = runtime
        .handle()
        .resolve_plugin::<ZrVmLanguageBackendRegistration>(ZR_VM_LANGUAGE_BACKEND_REGISTRATION_NAME)
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

#[test]
fn zr_vm_backend_has_one_plugin_owned_dense_production_path() {
    let plugin_manifest = include_str!("../../Cargo.toml");
    let runtime_manifest = include_str!("../../../../../zircon_runtime/Cargo.toml");
    let runtime_backend_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../zircon_runtime/src/script/vm/backend/zr_vm_project_backend.rs");
    let host_modules = include_str!("../real_backend/host_modules.rs");
    let instance = include_str!("../real_backend/instance.rs");
    let runtime_lock = include_str!("../real_backend/lock.rs");

    assert!(!plugin_manifest.contains("zircon_runtime/backend-zr-vm"));
    assert!(!runtime_manifest.contains("backend-zr-vm ="));
    assert!(
        !runtime_backend_root.exists(),
        "concrete ZrVM backend must not survive under zircon_runtime"
    );
    assert!(host_modules.contains("script_call_table"));
    assert!(host_modules.contains("ScriptCallSite"));
    assert!(!host_modules.contains(".call_with_capabilities("));
    assert!(!instance.contains("vampire_lifecycle_or_export"));

    let call_table_position = host_modules
        .find("let call_table = host.host_exports.script_call_table()?;")
        .expect("real backend resolves the dense call table once");
    let resolve_position = host_modules
        .find(".resolve(&module.descriptor.name, &function.name)")
        .expect("real backend resolves each host callback before registration");
    let callback_position = host_modules
        .find("builder = builder.add_function(build_native_function(")
        .expect("real backend registers the resolved callback");
    assert!(call_table_position < resolve_position && resolve_position < callback_position);

    let native_callback = host_modules
        .split_once("fn build_native_function(")
        .expect("real backend owns a native callback builder")
        .1
        .split_once("fn zr_prototype_type(")
        .expect("native callback builder has a bounded source section")
        .0;
    assert!(native_callback.contains("call_site.call(arguments, &capabilities)"));
    assert!(!native_callback.contains("host_exports"));
    assert!(!native_callback.contains("call_with_capabilities"));
    assert!(!native_callback.contains(".resolve("));

    assert!(runtime_lock.contains("unwrap_or_else(|poisoned| poisoned.into_inner())"));
    assert!(runtime_lock.contains("zr_vm_real_backend_runtime_lock_recovers_after_poison"));
    assert!(!runtime_lock.contains(".expect(\"zr_vm runtime lock should not be poisoned\")"));
}

#[test]
fn vm_registered_system_enters_schedule_conservatively() {
    let report = plugin_registration();
    let dispatchers = report
        .extensions
        .plugin_runtime_systems()
        .map(|(_, registration)| registration)
        .filter(|registration| registration.id.starts_with("zr_vm_language.systems."))
        .collect::<Vec<_>>();

    assert_eq!(dispatchers.len(), 3);
    let runtime_module = report
        .package_manifest
        .modules
        .iter()
        .find(|module| module.name == "zr_vm_language.runtime")
        .expect("runtime module is declared");
    assert_eq!(
        runtime_module.system_anchors,
        [
            ZR_VM_BEHAVIOR_BRIDGE_BIND_SYSTEM,
            crate::vm_system_dispatcher_id(zircon_runtime::script::VmSystemStage::FixedUpdate),
            crate::vm_system_dispatcher_id(zircon_runtime::script::VmSystemStage::Update),
            crate::vm_system_dispatcher_id(zircon_runtime::script::VmSystemStage::Last),
            ZR_VM_GC_STEP_SYSTEM,
        ]
        .map(str::to_string)
    );
    for registration in dispatchers {
        assert!(registration
            .build()
            .access()
            .has_conservative_world_access());
    }
}

#[test]
fn gc_step_resources_and_last_stage_system_are_registered() {
    let report = plugin_registration();
    let resource_types = report
        .extensions
        .plugin_resources()
        .map(|(_, registration)| registration.type_name())
        .collect::<Vec<_>>();
    assert!(resource_types.contains(&std::any::type_name::<zircon_runtime::script::VmGcBudget>()));
    assert!(resource_types.contains(&std::any::type_name::<
        zircon_runtime::script::VmGcDiagnostics,
    >()));

    let gc_system = report
        .extensions
        .plugin_runtime_systems()
        .map(|(_, registration)| registration)
        .find(|registration| registration.id == ZR_VM_GC_STEP_SYSTEM)
        .expect("script.gc_step runtime scene system is registered");
    assert_eq!(gc_system.stage, zircon_runtime::scene::SystemStage::Last);
    assert!(gc_system.constraints.iter().any(|constraint| matches!(
        constraint,
        zircon_runtime::scene::ecs::SystemOrderingConstraint::After(
            zircon_runtime::scene::ecs::SystemRef::System(system)
        ) if system == crate::vm_system_dispatcher_id(
            zircon_runtime::script::VmSystemStage::Last
        )
    )));
    assert!(gc_system.build().access().has_conservative_world_access());

    let runtime_module = report
        .package_manifest
        .modules
        .iter()
        .find(|module| module.name == "zr_vm_language.runtime")
        .expect("runtime module is declared");
    assert!(runtime_module
        .system_anchors
        .contains(&ZR_VM_GC_STEP_SYSTEM.to_string()));
}

#[test]
fn gc_step_respects_frame_budget() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let manager = zircon_runtime::script::VmPluginManager::mock();
    manager.register_family(Arc::new(BudgetRecordingFamily {
        calls: Arc::clone(&calls),
    }));
    let first = manager
        .load_package_with_backend("budget-recording:test", budget_test_package("first", 4))
        .unwrap();
    let second = manager
        .load_package_with_backend("budget-recording:test", budget_test_package("second", 8))
        .unwrap();
    manager
        .load_package_with_backend("budget-recording:test", budget_test_package("third", 1))
        .unwrap();

    let report = manager
        .gc_step(zircon_runtime::script::VmGcBudget {
            max_micros_per_frame: 10,
        })
        .unwrap();

    assert_eq!(
        calls.lock().unwrap().as_slice(),
        &[(first, 10), (second, 6)]
    );
    assert_eq!(report.pause_micros, 12);
    assert_eq!(report.overrun_micros, 2);
    assert_eq!(report.slots.len(), 2);
}

fn budget_test_package(name: &str, pause_micros: u64) -> zircon_runtime::script::VmPluginPackage {
    zircon_runtime::script::VmPluginPackage {
        manifest: zircon_runtime::script::VmPluginManifest {
            name: name.to_string(),
            version: pause_micros.to_string(),
            entry: "main".to_string(),
            capabilities: zircon_runtime::script::CapabilitySet::default(),
            management: zircon_runtime::script::VmPluginManagementPolicy::default()
                .with_garbage_collection(
                    zircon_runtime::script::VmPluginGarbageCollectionPolicy::cooperative(None),
                ),
        },
        zr_vm_project: None,
        bytecode: vec![1],
    }
}
