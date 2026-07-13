use std::sync::{Arc, Mutex};

use super::super::*;
use super::fixtures::RecordedPlugin;
use crate::core::framework::bridge::{BridgeError, BridgeOwnerTransitionMode, PluginInterface};
use crate::core::framework::project::{ExportPackagingStrategy, ProjectPluginSelection};
use crate::core::runtime::ServiceObject;
use crate::core::{CoreError, LifecycleState, ServiceKind, StartupMode};
use crate::plugin::{
    PluginDependencyManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimePluginBridgeLifecycleEvent, RuntimePluginBridgeLifecycleOutcome,
    RuntimePluginBridgeLifecycleState, RuntimePluginCatalog, RuntimePluginRegistrationReport,
};

#[test]
fn plugin_resolution_builds_plugin_context_instead_of_passing_only_core_handle() {
    let seen = Arc::new(Mutex::new(None::<PluginContext>));
    let seen_for_factory = Arc::clone(&seen);
    let runtime = CoreRuntime::new();
    let core = runtime.handle();

    core.register_module(
        ModuleDescriptor::new("PluginContextSpec", "plugin context test").with_plugin(
            PluginDescriptor::new(
                RegistryName::from_parts(
                    "PluginContextSpec",
                    ServiceKind::Plugin,
                    "RecordedPlugin",
                ),
                StartupMode::Immediate,
                Vec::new(),
                Arc::new(move |context: &PluginContext| {
                    *seen_for_factory.lock().unwrap() = Some(context.clone());
                    Ok(Arc::new(RecordedPlugin(context.clone())) as ServiceObject)
                }),
            ),
        ),
    )
    .unwrap();

    core.activate_module("PluginContextSpec").unwrap();
    let resolved = core
        .resolve_plugin::<RecordedPlugin>("PluginContextSpec.Plugin.RecordedPlugin")
        .unwrap();
    let context = seen.lock().unwrap().clone().unwrap();

    assert_eq!(resolved.0.plugin_name, context.plugin_name);
    assert_eq!(
        context.plugin_name,
        "PluginContextSpec.Plugin.RecordedPlugin"
    );
    assert!(context.core.upgrade().is_some());
    assert!(context.package_root.is_none());
    assert!(context.source_root.is_none());
    assert!(context.data_root.is_none());
}

#[test]
fn plugin_bridge_lifecycle_state_applies_explicit_provider_events() {
    let mut physics_extensions = RuntimeExtensionRegistry::default();
    let owner = physics_extensions
        .intern_plugin_module("physics.runtime")
        .unwrap();
    physics_extensions
        .export_interface::<dyn CoreRuntimePhysicsQuery>(owner, Arc::new(CoreRuntimePhysics))
        .unwrap();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            core_runtime_registration_with_extensions(
                core_runtime_package_with_runtime("physics", "Physics")
                    .with_provided_interface_id("physics.query.v1"),
                physics_extensions,
            ),
            core_runtime_registration(core_runtime_package("weather", "Weather").with_dependency(
                PluginDependencyManifest::new("physics", false).with_interface("physics.query.v1"),
            )),
        ],
        [],
    );
    let state = RuntimePluginBridgeLifecycleState::from_catalog(catalog);
    let bridge = state
        .bridge_table()
        .resolve_weak::<dyn CoreRuntimePhysicsQuery>();
    let disable_event = RuntimePluginBridgeLifecycleEvent::disable_provider("physics");

    let disable = state.apply_provider_lifecycle_event(disable_event);
    let RuntimePluginBridgeLifecycleOutcome::Applied(disable_report) = disable else {
        panic!("optional bridge dependent should not block disable");
    };

    assert_eq!(disable_report.mode, BridgeOwnerTransitionMode::Disable);
    assert_eq!(
        bridge.call(|provider| provider.query_count()),
        Err(BridgeError::NotEnabled)
    );

    let activate = state.apply_provider_lifecycle_event(
        RuntimePluginBridgeLifecycleEvent::activate_provider("physics"),
    );
    assert!(activate.is_applied());
    assert_eq!(bridge.call(|provider| provider.query_count()), Ok(7));
}

#[test]
fn core_runtime_module_deactivation_drives_plugin_bridge_lifecycle() {
    let mut physics_extensions = RuntimeExtensionRegistry::default();
    let owner = physics_extensions
        .intern_plugin_module("physics.runtime")
        .unwrap();
    physics_extensions
        .export_interface::<dyn CoreRuntimePhysicsQuery>(owner, Arc::new(CoreRuntimePhysics))
        .unwrap();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [core_runtime_registration_with_extensions(
            core_runtime_package_with_runtime("physics", "Physics")
                .with_provided_interface_id("physics.query.v1"),
            physics_extensions,
        )],
        [],
    );
    let state = RuntimePluginBridgeLifecycleState::from_catalog(catalog);
    let bridge = state
        .bridge_table()
        .resolve_weak::<dyn CoreRuntimePhysicsQuery>();
    let runtime = CoreRuntime::new();
    runtime
        .register_module(ModuleDescriptor::new("physics.runtime", "physics runtime"))
        .unwrap();
    runtime.install_runtime_module_lifecycle_observer(Arc::new(state.clone()));

    assert_eq!(bridge.call(|provider| provider.query_count()), Ok(7));

    runtime.deactivate_module("physics.runtime").unwrap();

    assert_eq!(
        bridge.call(|provider| provider.query_count()),
        Err(BridgeError::NotEnabled)
    );
    let deactivated = state
        .bridge_table()
        .interface_snapshot_by_id("physics.query.v1")
        .expect("physics interface should remain in the bridge table");
    assert!(!deactivated.provider_installed);

    runtime.activate_module("physics.runtime").unwrap();

    assert_eq!(bridge.call(|provider| provider.query_count()), Ok(7));
    let reactivated = state
        .bridge_table()
        .interface_snapshot_by_id("physics.query.v1")
        .expect("physics interface should remain in the bridge table");
    assert!(reactivated.provider_installed);
}

#[test]
fn core_runtime_module_deactivation_rejects_strong_bridge_dependents_before_unload() {
    let mut physics_extensions = RuntimeExtensionRegistry::default();
    let owner = physics_extensions
        .intern_plugin_module("physics.runtime")
        .unwrap();
    physics_extensions
        .export_interface::<dyn CoreRuntimePhysicsQuery>(owner, Arc::new(CoreRuntimePhysics))
        .unwrap();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            core_runtime_registration_with_extensions(
                core_runtime_package_with_runtime("physics", "Physics")
                    .with_provided_interface_id("physics.query.v1"),
                physics_extensions,
            ),
            core_runtime_registration(core_runtime_package("weather", "Weather").with_dependency(
                PluginDependencyManifest::new("physics", true).with_interface("physics.query.v1"),
            )),
        ],
        [],
    );
    let state = RuntimePluginBridgeLifecycleState::from_catalog(catalog);
    let bridge = state
        .bridge_table()
        .resolve_weak::<dyn CoreRuntimePhysicsQuery>();
    let runtime = CoreRuntime::new();
    runtime
        .register_module(ModuleDescriptor::new("physics.runtime", "physics runtime"))
        .unwrap();
    runtime.activate_module("physics.runtime").unwrap();
    runtime.install_runtime_module_lifecycle_observer(Arc::new(state));

    let error = runtime.deactivate_module("physics.runtime").unwrap_err();

    let CoreError::RuntimeModuleLifecycleBlocked(detail) = error else {
        panic!("strong bridge dependents should block module deactivation");
    };
    assert!(detail.contains("bridge.provider_lifecycle_blocked"));
    assert_eq!(bridge.call(|provider| provider.query_count()), Ok(7));
    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("physics.runtime")
        .expect("blocked deactivation should keep the module registered");
    assert_eq!(module.lifecycle, LifecycleState::Running);
}

fn core_runtime_registration(manifest: PluginPackageManifest) -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_native_package_manifest(manifest)
}

fn core_runtime_registration_with_extensions(
    manifest: PluginPackageManifest,
    extensions: RuntimeExtensionRegistry,
) -> RuntimePluginRegistrationReport {
    let project_selection = ProjectPluginSelection {
        id: manifest.id.clone(),
        enabled: true,
        required: false,
        target_modes: Vec::new(),
        packaging: ExportPackagingStrategy::SourceTemplate,
        runtime_crate: None,
        editor_crate: None,
        features: Vec::new(),
    };
    RuntimePluginRegistrationReport {
        package_manifest: manifest,
        project_selection,
        extensions,
        diagnostics: Vec::new(),
    }
}

fn core_runtime_package(id: &str, display_name: &str) -> PluginPackageManifest {
    PluginPackageManifest::new(id, display_name).with_capability(format!("runtime.plugin.{id}"))
}

fn core_runtime_package_with_runtime(id: &str, display_name: &str) -> PluginPackageManifest {
    core_runtime_package(id, display_name).with_runtime_crate(format!("{id}_runtime"))
}

trait CoreRuntimePhysicsQuery: Send + Sync {
    fn query_count(&self) -> u32;
}

impl PluginInterface for dyn CoreRuntimePhysicsQuery {
    const INTERFACE_ID: &'static str = "physics.query.v1";
}

#[derive(Debug)]
struct CoreRuntimePhysics;

impl CoreRuntimePhysicsQuery for CoreRuntimePhysics {
    fn query_count(&self) -> u32 {
        7
    }
}
