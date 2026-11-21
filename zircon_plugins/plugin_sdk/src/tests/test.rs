use std::sync::Arc;

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

mod base_dependencies {
    use zircon_runtime::asset::AssetManager;
    use zircon_runtime::core::runtime::modules::{TASKS_MODULE_NAME, TIME_MODULE_NAME};
    use zircon_runtime::core::LifecycleState;

    use super::*;

    const BASE_MODULE_NAMES: [&str; 5] = [
        foundation::FOUNDATION_MODULE_NAME,
        asset::ASSET_MODULE_NAME,
        SCENE_MODULE_NAME,
        TASKS_MODULE_NAME,
        TIME_MODULE_NAME,
    ];

    #[test]
    fn test_runtime_builder_base_default_ticks_without_registrations() {
        let fixture = ManagedConfigFixture::new("sdk-base-default");
        let built = TestRuntime::builder()
            .with_config_file_path(fixture.config_path())
            .build();
        println!(
            "SDK_BASE_V1 case=default build={:?}",
            built.as_ref().map(TestRuntime::activated_modules)
        );
        let runtime = built.expect("zero registrations must not prevent default SDK startup");
        let explicit = runtime
            .activated_modules()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(explicit.as_slice(), &BASE_MODULE_NAMES[..3]);
        assert!(runtime.extension_report().is_success());
        assert!(runtime.extension_report().registry.modules().is_empty());

        // The SDK list is only the explicit activation requests. Probe the real Core
        // unload gate to distinguish running prerequisites from matching names in a list.
        for (provider, consumer) in [
            (TASKS_MODULE_NAME, asset::ASSET_MODULE_NAME),
            (TIME_MODULE_NAME, SCENE_MODULE_NAME),
        ] {
            let error = runtime
                .runtime()
                .deactivate_module(provider)
                .expect_err("a running base root must protect its kernel prerequisite");
            println!("SDK_BASE_V1 case=default provider={provider} unload_gate={error:?}");
            assert!(
                matches!(&error, CoreError::ModuleUnloadBlocked { module, dependents }
                    if module.as_str() == provider && dependents.iter().any(|name| name.as_str() == consumer)),
                "{error:?}"
            );
        }

        let core = runtime.handle();
        let handle = asset::project_asset_manager_handle(&core).expect("typed asset manager handle");
        let manager = zircon_runtime::core::manager::resolve_manager_service(&core, handle)
            .expect("admit the real ProjectAssetManager");
        assert!(manager.current_project_snapshot().is_none());
        drop(manager);

        let level = runtime.create_default_level().expect("registered Scene");
        let before = level.world_time();
        // Exactly 31.25 ms: longer than one SDK default fixed step but shorter than two.
        runtime.tick_level_seconds(&level, 0.03125).unwrap();
        let after = level.world_time();
        println!(
            "SDK_BASE_V1 case=default asset_manager=resolved before={before:?} after={after:?}"
        );
        assert_eq!(before.last_outer_frame_index(), None);
        assert_eq!(
            after.last_outer_frame_index(),
            Some(core.real_time().frame_index())
        );
        assert_eq!(after.virtual_time().delta(), Duration::from_micros(31_250));
        assert_eq!(
            after.fixed_time().frame_index(),
            before.fixed_time().frame_index() + 1
        );
    }

    #[test]
    fn test_runtime_builder_base_explicit_missing_asset_still_rejects() {
        let fixture = ManagedConfigFixture::new("sdk-base-no-asset");
        let error = TestRuntime::builder()
            .with_config_file_path(fixture.config_path())
            .with_base_modules([
                TestRuntimeBaseModule::Foundation,
                TestRuntimeBaseModule::Scene,
            ])
            .build()
            .expect_err("explicit selection without Asset must reject Scene");
        println!("SDK_BASE_V1 case=missing_asset error={error:?}");
        assert!(
            matches!(
                &error,
                TestRuntimeError::Core {
                    action: "install world runtime extensions",
                    target,
                    source: CoreError::MissingModuleDependency { module, dependency },
                } if target == scene::WORLD_DRIVER_NAME
                    && module == SCENE_MODULE_NAME
                    && dependency == asset::ASSET_MODULE_NAME
            ),
            "{error:?}"
        );
    }

    #[test]
    fn test_runtime_builder_base_omission_does_not_register_kernels_or_roots() {
        let fixture = ManagedConfigFixture::new("sdk-base-none");
        let runtime = TestRuntime::builder()
            .with_config_file_path(fixture.config_path())
            .without_base_modules()
            .without_scene_runtime_extension_plan()
            .build()
            .expect("bare fixture with no scene installation");
        assert!(runtime.activated_modules().is_empty());
        for name in BASE_MODULE_NAMES {
            let result = runtime.runtime().deactivate_module(name);
            println!("SDK_BASE_V1 case=omitted module={name} unload_gate={result:?}");
            assert_eq!(result, Err(CoreError::MissingModule(name.to_owned())));
        }
    }

    #[test]
    fn test_runtime_builder_base_dormant_roots_register_but_do_not_activate() {
        let fixture = ManagedConfigFixture::new("sdk-base-dormant");
        let runtime = TestRuntime::builder()
            .with_config_file_path(fixture.config_path())
            // Also skip plan installation, which would resolve and lazily activate Scene.
            .without_scene_runtime_extension_plan()
            .without_base_module_activation()
            .build()
            .expect("register a dormant base closure");
        assert!(runtime.activated_modules().is_empty());
        for name in BASE_MODULE_NAMES {
            let result = runtime.runtime().deactivate_module(name);
            println!("SDK_BASE_V1 case=dormant module={name} unload_gate={result:?}");
            assert_eq!(
                result,
                Err(CoreError::InvalidModuleLifecycleTransition {
                    module: name.to_owned(),
                    command: "deactivate",
                    state: LifecycleState::Registered,
                })
            );
        }
    }

    #[test]
    fn sdk_base_selected_foundation_has_no_kernels() {
        const KEY: &str = "sdk_base_selected_sentinel";
        for activate in [true, false] {
            let fixture = ManagedConfigFixture::new("sdk-base-selected-foundation");
            let path = fixture.config_path();
            std::fs::write(&path, r#"{"sdk_base_selected_sentinel":true}"#).unwrap();
            let builder = TestRuntime::builder()
                .with_config_file_path(path)
                .with_base_modules([TestRuntimeBaseModule::Foundation])
                .without_scene_runtime_extension_plan();
            let builder = if activate {
                builder
            } else {
                builder.without_base_module_activation()
            };
            let runtime = builder.build().expect("Foundation-only SDK runtime");
            // Read the Core store directly: resolving the config manager could activate Foundation.
            let sentinel = runtime.handle().load_config_value(KEY);
            println!(
                "SDK_BASE_SELECTED_V1 case=foundation activate={activate} explicit={:?} sentinel={sentinel:?}",
                runtime.activated_modules()
            );
            if activate {
                assert_eq!(
                    runtime.activated_modules(),
                    &[foundation::FOUNDATION_MODULE_NAME.to_owned()]
                );
                assert_eq!(sentinel, Some(serde_json::Value::Bool(true)));
            } else {
                assert!(runtime.activated_modules().is_empty());
                assert_eq!(sentinel, None);
                assert_selected_stop_states(&runtime, &[foundation::FOUNDATION_MODULE_NAME], &[]);
            }
            assert_selected_stop_states(
                &runtime,
                &[],
                &[
                    TASKS_MODULE_NAME,
                    TIME_MODULE_NAME,
                    asset::ASSET_MODULE_NAME,
                    SCENE_MODULE_NAME,
                ],
            );
        }
    }

    #[test]
    fn sdk_base_selected_asset_has_only_tasks_kernel() {
        for activate in [true, false] {
            let fixture = ManagedConfigFixture::new("sdk-base-selected-asset");
            let builder = TestRuntime::builder()
                .with_config_file_path(fixture.config_path())
                .with_base_modules([TestRuntimeBaseModule::Asset])
                .without_scene_runtime_extension_plan();
            let builder = if activate {
                builder
            } else {
                builder.without_base_module_activation()
            };
            let runtime = builder.build().expect("Asset-only SDK runtime");
            let missing = [
                foundation::FOUNDATION_MODULE_NAME,
                TIME_MODULE_NAME,
                SCENE_MODULE_NAME,
            ];
            if activate {
                assert_eq!(
                    runtime.activated_modules(),
                    &[asset::ASSET_MODULE_NAME.to_owned()]
                );
                let tasks_unload = runtime.runtime().deactivate_module(TASKS_MODULE_NAME);
                println!("SDK_BASE_SELECTED_V1 case=asset activate=true tasks={tasks_unload:?}");
                assert_eq!(
                    tasks_unload,
                    Err(CoreError::ModuleUnloadBlocked {
                        module: TASKS_MODULE_NAME.to_owned(),
                        dependents: vec![asset::ASSET_MODULE_NAME.to_owned()],
                    })
                );
                let core = runtime.handle();
                let handle = asset::project_asset_manager_handle(&core).unwrap();
                let manager = zircon_runtime::core::manager::resolve_manager_service(&core, handle)
                    .expect("admit the real ProjectAssetManager without Foundation or Scene");
                assert!(manager.current_project_snapshot().is_none());
            } else {
                assert!(runtime.activated_modules().is_empty());
                assert_selected_stop_states(
                    &runtime,
                    &[TASKS_MODULE_NAME, asset::ASSET_MODULE_NAME],
                    &[],
                );
            }
            println!(
                "SDK_BASE_SELECTED_V1 case=asset activate={activate} explicit={:?}",
                runtime.activated_modules()
            );
            assert_selected_stop_states(&runtime, &[], &missing);
        }
    }

    fn assert_selected_stop_states(runtime: &TestRuntime, registered: &[&str], missing: &[&str]) {
        for &module in registered {
            assert_eq!(
                runtime.runtime().deactivate_module(module),
                Err(CoreError::InvalidModuleLifecycleTransition {
                    module: module.to_owned(),
                    command: "deactivate",
                    state: LifecycleState::Registered,
                })
            );
        }
        for &module in missing {
            assert_eq!(
                runtime.runtime().deactivate_module(module),
                Err(CoreError::MissingModule(module.to_owned()))
            );
        }
    }
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
        use zircon_runtime::asset::project::ProjectPaths;
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
        let current = zircon_runtime::asset::project::ProjectPaths::resolve_identity(
            self.root.operation_path(),
        )
        .unwrap();
        assert_eq!(
            current, self.root,
            "cleanup must retain this fixture's physical identity"
        );
        std::fs::remove_dir_all(self.root.operation_path()).unwrap();
    }
}
