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
    let fixture = ManagedConfigFixture::new("sdk-modules");
    let plugin = SdkTestRuntimePlugin::new();
    let runtime = TestRuntime::builder()
        .with_config_file_path(fixture.config_path())
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
    let transferred = runtime.into_runtime();
    assert!(fixture.root.operation_path().is_dir());
    transferred
        .resolve_manager::<SdkTestManager>(TEST_MANAGER_NAME)
        .unwrap();
    drop(transferred);
}

#[test]
fn test_runtime_builder_can_build_scene_levels_with_extensions_installed() {
    let fixture = ManagedConfigFixture::new("sdk-levels");
    let plugin = SdkTestRuntimePlugin::new();
    let runtime = TestRuntime::builder()
        .with_config_file_path(fixture.config_path())
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

struct ManagedConfigFixture {
    root: zircon_runtime::asset::project::ResolvedProjectPathIdentity,
}

impl ManagedConfigFixture {
    fn new(label: &str) -> Self {
        use zircon_runtime::asset::ProjectPaths;
        static NEXT_FIXTURE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let approved = ProjectPaths::resolve_existing("E:/cargo-targets/zircon-local")
            .expect("the managed E drive fixture root must already exist");
        let display = ProjectPaths::display_path(approved.operation_path());
        assert!(
            display
                .to_str()
                .expect("managed root must be Unicode")
                .replace('\\', "/")
                .eq_ignore_ascii_case("E:/cargo-targets/zircon-local"),
            "the approved root must resolve to the physical E drive root"
        );
        let approved = ProjectPaths::resolve_identity(approved.operation_path()).unwrap();
        let executable = std::env::current_exe().unwrap();
        let binary = ProjectPaths::resolve_identity(executable.parent().unwrap()).unwrap();
        assert!(
            binary.is_within(&approved),
            "test binary must physically live under the managed E root"
        );
        let id = NEXT_FIXTURE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = ProjectPaths::resolve_identity(binary.operation_path().join(format!(
            "runtime55-{label}-{}-{nanos}-{id}",
            std::process::id()
        )))
        .unwrap();
        assert!(root.is_within(&binary));
        // Every authority check precedes mkdir/write. create_dir never adopts an existing root.
        std::fs::create_dir(root.operation_path()).unwrap();
        let fixture = Self { root };
        assert_eq!(
            ProjectPaths::resolve_identity(fixture.root.operation_path()).unwrap(),
            fixture.root
        );
        fixture
    }

    fn config_path(&self) -> std::path::PathBuf {
        self.root.operation_path().join("config.json")
    }
}

impl Drop for ManagedConfigFixture {
    fn drop(&mut self) {
        // The declaring scope/wrapper closes all runtimes, handles and worker owners first.
        let current =
            zircon_runtime::asset::ProjectPaths::resolve_identity(self.root.operation_path())
                .unwrap();
        assert_eq!(
            current, self.root,
            "cleanup must retain this fixture's physical identity"
        );
        std::fs::remove_dir_all(self.root.operation_path()).unwrap();
    }
}
