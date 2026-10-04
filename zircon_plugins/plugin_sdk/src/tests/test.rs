use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{ManagerDescriptor, ServiceKind, StartupMode};
use zircon_runtime::engine_module::{factory, qualified_name};
use zircon_runtime::plugin::{RuntimeExtensionRegistry, RuntimePluginDescriptor};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use super::*;

const TEST_PACKAGE_ID: &str = "prefab_tools";
const TEST_PLUGIN_MODULE_NAME: &str = "prefab_tools.runtime";
const TEST_RUNTIME_MODULE_NAME: &str = "SdkTestRuntimeModule";
const TEST_MANAGER_NAME: &str = "SdkTestRuntimeModule.Manager.SdkTestManager";

#[derive(Debug)]
struct SdkTestManager;

#[derive(Clone, Debug)]
struct SdkTestRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl SdkTestRuntimePlugin {
    fn new() -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                TEST_PACKAGE_ID,
                "SDK Test Runtime",
                RuntimePluginId::PrefabTools,
                "zircon_plugin_sdk_test_runtime",
            )
            .with_category("runtime")
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.prefab_tools")
            .build(),
        }
    }
}

impl RuntimePlugin for SdkTestRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> std::result::Result<(), RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module(TEST_PLUGIN_MODULE_NAME)?;
        assert_eq!(
            registry.plugin_module_name(owner),
            Some(TEST_PLUGIN_MODULE_NAME)
        );
        registry.register_module(test_runtime_module_descriptor())
    }
}

fn test_runtime_module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(TEST_RUNTIME_MODULE_NAME, "SDK test runtime module").with_manager(
        ManagerDescriptor::new(
            qualified_name(
                TEST_RUNTIME_MODULE_NAME,
                ServiceKind::Manager,
                "SdkTestManager",
            ),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(SdkTestManager) as ServiceObject)),
        ),
    )
}

#[test]
fn test_runtime_builder_registers_base_and_plugin_modules() {
    let plugin = SdkTestRuntimePlugin::new();
    let runtime = TestRuntime::builder()
        .with_runtime_plugin(&plugin)
        .build()
        .expect("SDK test runtime should build");

    assert!(runtime
        .activated_modules()
        .contains(&foundation::FOUNDATION_MODULE_NAME.to_string()));
    assert!(runtime
        .activated_modules()
        .contains(&asset::ASSET_MODULE_NAME.to_string()));
    assert!(runtime
        .activated_modules()
        .contains(&SCENE_MODULE_NAME.to_string()));
    assert!(runtime
        .activated_modules()
        .contains(&TEST_RUNTIME_MODULE_NAME.to_string()));
    runtime
        .resolve_manager::<SdkTestManager>(TEST_MANAGER_NAME)
        .expect("plugin manager should resolve after module activation");
    assert!(runtime.extension_report().is_success());
}

#[test]
fn test_runtime_builder_can_build_scene_levels_with_extensions_installed() {
    let plugin = SdkTestRuntimePlugin::new();
    let runtime = TestRuntime::builder()
        .with_runtime_plugin(&plugin)
        .build()
        .expect("SDK test runtime should build");

    let level = runtime
        .create_default_level()
        .expect("default level should include runtime world extensions");
    runtime
        .tick_level_seconds(&level, 1.0 / 60.0)
        .expect("level tick should use the SDK runtime clock");
}

#[test]
fn level_tick_failures_retain_their_level_error_boundary() {
    let error = TestRuntimeError::LevelTick {
        action: "tick level",
        target: SCENE_MODULE_NAME.to_owned(),
        source: scene::LevelTickError::from(CoreError::RuntimeUnavailable),
    };

    assert!(matches!(
        error,
        TestRuntimeError::LevelTick {
            source: scene::LevelTickError::Core(CoreError::RuntimeUnavailable),
            ..
        }
    ));
}
