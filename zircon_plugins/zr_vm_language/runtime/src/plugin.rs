use std::sync::Arc;

use crate::capability::{RUNTIME_CRATE_NAME, ZR_VM_LANGUAGE_DECLARATION};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::core::framework::script::{
    ScriptBehaviorBridge, SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID,
};
use zircon_runtime::plugin::{
    CapabilityStatus, CapabilityStatusManifest, PluginDistributionManifest,
    PluginInterfaceManifest, PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};
use zircon_runtime::script::{VmGcBudget, VmGcDiagnostics, VmSystemStage};

use crate::{
    module_descriptor, PLUGIN_ID, RUNTIME_CAPABILITIES, ZR_VM_LANGUAGE_MODULE_NAME,
    ZR_VM_LANGUAGE_RUNTIME_CAPABILITY, ZR_VM_PROJECT_BACKEND_CAPABILITY,
};

pub const ZR_VM_LANGUAGE_DIST_CRATE_NAME: &str = "zircon_plugin_zr_vm_language_dist";
pub const ZR_VM_LANGUAGE_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_zr_vm_language_runtime_entry_v3";
pub const ZR_VM_LANGUAGE_MAIN_SYSTEM_SET: &str =
    zircon_runtime::script::SCRIPT_SCENE_RUNTIME_SYSTEM_SET;
pub const ZR_VM_BEHAVIOR_BRIDGE_BIND_SYSTEM: &str = "zr_vm_language.script.behavior_bridge.bind";
pub const ZR_VM_GC_STEP_SYSTEM: &str = "zr_vm_language.script.gc_step";
const ZR_VM_LANGUAGE_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

#[derive(Clone, Debug)]
pub struct ZrVmLanguageRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
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

impl RuntimePlugin for ZrVmLanguageRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut manifest = self.descriptor().package_manifest();
        manifest = manifest.with_native_module(
            PluginModuleManifest::native("zr_vm_language.dist", ZR_VM_LANGUAGE_DIST_CRATE_NAME)
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ServerRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(RUNTIME_CAPABILITIES.iter().copied()),
        );
        manifest.with_distribution(PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: ZR_VM_LANGUAGE_DIST_ENGINE_COMPAT.to_string(),
            dist_crate: ZR_VM_LANGUAGE_DIST_CRATE_NAME.to_string(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: ZR_VM_LANGUAGE_DIST_RUNTIME_ENTRY.to_string(),
            ..PluginDistributionManifest::default()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
            .module(ZR_VM_LANGUAGE_MODULE_NAME)?;
        let behavior_bridge = Arc::new(zircon_runtime::script::VmScriptBehaviorBridge::new());
        let exported_behavior_bridge: Arc<dyn ScriptBehaviorBridge> = behavior_bridge.clone();
        module.export_interface::<dyn ScriptBehaviorBridge>(exported_behavior_bridge)?;
        module
            .runtime_scene_system(
                ZR_VM_BEHAVIOR_BRIDGE_BIND_SYSTEM,
                zircon_runtime::scene::SystemStage::First,
                move || {
                    let behavior_bridge = Arc::clone(&behavior_bridge);
                    move |context| {
                        let manager = context
                            .core
                            .resolve_manager::<zircon_runtime::script::VmPluginManager>(
                                zircon_runtime::script::VM_PLUGIN_MANAGER_NAME,
                            )?;
                        behavior_bridge.bind_manager(&manager);
                        Ok(())
                    }
                },
            )
            .in_set(ZR_VM_LANGUAGE_MAIN_SYSTEM_SET)
            .register()?;
        module.resource(VmGcBudget::default)?;
        module.resource(VmGcDiagnostics::default)?;
        for stage in zircon_runtime::script::VmSystemStage::ALL {
            let system_id = vm_system_dispatcher_id(stage);
            module
                .runtime_scene_system(system_id, stage.system_stage(), move || {
                    move |context| {
                        let manager = context
                            .core
                            .resolve_manager::<zircon_runtime::script::VmPluginManager>(
                                zircon_runtime::script::VM_PLUGIN_MANAGER_NAME,
                            )?;
                        manager
                            .run_registered_systems(stage, context.tick().delta_seconds())
                            .map(|_| ())
                            .map_err(|error| {
                                zircon_runtime::core::CoreError::Initialization(
                                    vm_system_dispatcher_id(stage).to_string(),
                                    error.to_string(),
                                )
                            })
                    }
                })
                .in_set(ZR_VM_LANGUAGE_MAIN_SYSTEM_SET)
                .register()?;
        }
        module
            .runtime_scene_system(
                ZR_VM_GC_STEP_SYSTEM,
                zircon_runtime::scene::SystemStage::Last,
                || {
                    |context| {
                        let manager = context
                            .core
                            .resolve_manager::<zircon_runtime::script::VmPluginManager>(
                                zircon_runtime::script::VM_PLUGIN_MANAGER_NAME,
                            )?;
                        let budget = context.level.with_world(|world| {
                            world.get_resource::<VmGcBudget>().copied().ok_or_else(|| {
                                zircon_runtime::core::CoreError::Initialization(
                                    ZR_VM_GC_STEP_SYSTEM.to_string(),
                                    "VmGcBudget resource is not registered".to_string(),
                                )
                            })
                        })?;
                        let report = manager.gc_step(budget).map_err(|error| {
                            zircon_runtime::core::CoreError::Initialization(
                                ZR_VM_GC_STEP_SYSTEM.to_string(),
                                error.to_string(),
                            )
                        })?;
                        context.level.with_world_mut(|world| {
                            let diagnostics =
                                world.get_resource_mut::<VmGcDiagnostics>().ok_or_else(|| {
                                    zircon_runtime::core::CoreError::Initialization(
                                        ZR_VM_GC_STEP_SYSTEM.to_string(),
                                        "VmGcDiagnostics resource is not registered".to_string(),
                                    )
                                })?;
                            diagnostics.push(report);
                            Ok(())
                        })
                    }
                },
            )
            .in_set(ZR_VM_LANGUAGE_MAIN_SYSTEM_SET)
            .after(zircon_runtime::scene::ecs::SystemRef::System(
                vm_system_dispatcher_id(VmSystemStage::Last).to_string(),
            ))
            .register()?;
        for system in [
            zircon_runtime::script::ScriptSceneRuntimeSystem::fixed_update(),
            zircon_runtime::script::ScriptSceneRuntimeSystem::update(),
        ] {
            let id = system.id();
            let stage = system.stage();
            module
                .runtime_scene_system(id, stage, move || {
                    let system = system.clone();
                    move |context| system.run(context)
                })
                .in_set(ZR_VM_LANGUAGE_MAIN_SYSTEM_SET)
                .with_order(10)
                .register()?;
        }
        Ok(())
    }
}

/// Returns the fixed runtime dispatcher identifier for a VM system stage.
pub const fn vm_system_dispatcher_id(stage: zircon_runtime::script::VmSystemStage) -> &'static str {
    match stage {
        zircon_runtime::script::VmSystemStage::FixedUpdate => "zr_vm_language.systems.fixed_update",
        zircon_runtime::script::VmSystemStage::Update => "zr_vm_language.systems.update",
        zircon_runtime::script::VmSystemStage::Last => "zr_vm_language.systems.last",
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    ZR_VM_LANGUAGE_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .with_system_sets([ZR_VM_LANGUAGE_MAIN_SYSTEM_SET])
        .with_enabled_by_default(false)
        // This is a linked Rust typed interface. NativeDynamic method ABI bindings
        // remain empty until a byte-level ScriptHostValue protocol is specified.
        .with_provided_interface(PluginInterfaceManifest::new(
            SCRIPT_BEHAVIOR_BRIDGE_INTERFACE_ID,
        ))
        .with_system_anchors([
            ZR_VM_BEHAVIOR_BRIDGE_BIND_SYSTEM,
            vm_system_dispatcher_id(VmSystemStage::FixedUpdate),
            vm_system_dispatcher_id(VmSystemStage::Update),
            vm_system_dispatcher_id(VmSystemStage::Last),
            ZR_VM_GC_STEP_SYSTEM,
            zircon_runtime::script::SCRIPT_SCENE_FIXED_UPDATE_SYSTEM,
            zircon_runtime::script::SCRIPT_SCENE_UPDATE_SYSTEM,
        ])
        .with_capability_status(CapabilityStatusManifest::new(
            ZR_VM_LANGUAGE_RUNTIME_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .with_capability_status(CapabilityStatusManifest::new(
            ZR_VM_PROJECT_BACKEND_CAPABILITY,
            CapabilityStatus::Partial,
        ))
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(ZrVmLanguageRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    RUNTIME_CAPABILITIES
}
