#![cfg(feature = "runtime")]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use zircon_plugin_sdk::{TestRuntime, TestRuntimeBaseModule, TestRuntimeBuilder, TestRuntimeError};
use zircon_runtime::asset::project::{ProjectPaths, ResolvedProjectPathIdentity};
use zircon_runtime::asset::ASSET_MODULE_NAME;
use zircon_runtime::builtin::{RuntimeModuleCompositionCompiler, RuntimePluginId};
use zircon_runtime::core::framework::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::core::manager::{
    config_manager_handle, resolve_manager_service, CONFIG_MANAGER_NAME,
};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreError, CoreHandle, InitLevel, ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor,
    ServiceKind, StartupMode,
};
use zircon_runtime::engine_module::{dependency_on, factory, qualified_name};
use zircon_runtime::plugin::{
    PluginModuleKind, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

const CONSUMER_PACKAGE: &str = "sdk_order_a_consumer";
const CONSUMER_MODULE: &str = "sdk_order_a_consumer.runtime";
const CONSUMER_MANAGER: &str = "sdk_order_a_consumer.runtime.Manager.ConfigRead";
const PROVIDER_PACKAGE: &str = "sdk_order_z_provider";
const PROVIDER_MODULE: &str = "sdk_order_z_provider.runtime";
const EXTRA_MODULE: &str = "SdkTestRuntimeModule";
const EXTRA_MANAGER: &str = "SdkTestRuntimeModule.Manager.SdkTestManager";
const CONFIG_KEY: &str = "sdk_host_order_probe";
const TARGET: RuntimeTargetMode = RuntimeTargetMode::ClientRuntime;

#[derive(Debug)]
struct ConfigRead(Value);

fn read_fixture_config(core: &CoreHandle) -> Result<Value, CoreError> {
    resolve_manager_service(core, config_manager_handle(core)?)?
        .get_value(CONFIG_KEY)
        .ok_or_else(|| {
            CoreError::Initialization(
                CONFIG_MANAGER_NAME.to_owned(),
                format!("missing seeded key {CONFIG_KEY}"),
            )
        })
}

fn descriptor(package: &str, module: ModuleDescriptor) -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        package,
        module.description.clone(),
        RuntimePluginId::new(package),
        format!("zircon_plugin_{package}_runtime"),
    )
    .with_target_modes([TARGET])
    .with_capability(format!("runtime.plugin.{package}"))
    .with_required_by_default(true)
    .with_module_descriptor(module.with_init_level(InitLevel::Post))
    .build()
}

fn registration(plugin: &dyn RuntimePlugin) -> RuntimePluginRegistrationReport {
    let report = RuntimePluginRegistrationReport::from_plugin(plugin);
    assert!(
        report.is_success(),
        "registration diagnostics: {:?}",
        report.diagnostics
    );
    report
}

fn host_consumer_report() -> RuntimePluginRegistrationReport {
    let dependency = ModuleDependencySpec::named(FOUNDATION_MODULE_NAME);
    let module = ModuleDescriptor::new(CONSUMER_MODULE, "SDK host config consumer")
        .with_module_dependency(dependency.clone())
        .with_manager(ManagerDescriptor::new(
            qualified_name(CONSUMER_MODULE, ServiceKind::Manager, "ConfigRead"),
            StartupMode::Immediate,
            vec![dependency_on(
                FOUNDATION_MODULE_NAME,
                ServiceKind::Manager,
                "ConfigManager",
            )],
            factory(|weak| {
                let core = weak.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
                Ok(Arc::new(ConfigRead(read_fixture_config(&core)?)) as ServiceObject)
            }),
        ));
    let report = registration(&descriptor(CONSUMER_PACKAGE, module));
    let [row] = report.package_manifest.modules.as_slice() else {
        panic!("the package must declare just the consumer row");
    };
    assert_eq!(row.name, CONSUMER_MODULE);
    assert_eq!(row.kind, PluginModuleKind::Runtime);
    assert_eq!(row.module_dependencies, [dependency.clone()]);
    let [registered] = report.extensions.modules() else {
        panic!("the report must contribute just the actual consumer module");
    };
    assert_eq!(registered.name, CONSUMER_MODULE);
    assert_eq!(registered.module_dependencies, [dependency]);
    report
}

struct LegacyProvider(RuntimePluginDescriptor);
struct SdkTestManager;

impl RuntimePlugin for LegacyProvider {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.0
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        // Same compatibility shape as the existing SDK unit fixture: this is not a manifest row.
        registry.register_module(
            ModuleDescriptor::new(EXTRA_MODULE, "Legacy SDK extra module").with_manager(
                ManagerDescriptor::new(
                    qualified_name(EXTRA_MODULE, ServiceKind::Manager, "SdkTestManager"),
                    StartupMode::Immediate,
                    Vec::new(),
                    factory(|_| Ok(Arc::new(SdkTestManager) as ServiceObject)),
                ),
            ),
        )
    }
}

fn foundation_only(fixture: &ManagedConfig) -> TestRuntimeBuilder {
    TestRuntime::builder()
        .with_base_modules([TestRuntimeBaseModule::Foundation])
        .with_config_file_path(fixture.path())
        // No Scene/Asset in this SORT control; this is not the #214 fixture.
        .without_scene_runtime_extension_plan()
}

#[test]
#[ignore = "requires an isolated physical SDK test root; run explicitly"]
fn host_order_rejects_genuinely_missing_foundation() {
    let fixture = ManagedConfig::new("missing_host");
    let error = foundation_only(&fixture)
        .without_base_modules()
        .with_registration_report(host_consumer_report())
        .build()
        .expect_err("the same report has no Foundation provider in this runtime");
    println!("SDK_SORT_V1 case=missing_host base=[] build={error:?}");
    let TestRuntimeError::RuntimeExtensionCatalog {
        diagnostics,
        fatal_diagnostics,
    } = &error
    else {
        panic!("expected the catalog order gate, not a manager or activation error: {error:?}");
    };
    let missing = CoreError::MissingModuleDependency {
        module: CONSUMER_MODULE.to_owned(),
        dependency: FOUNDATION_MODULE_NAME.to_owned(),
    };
    assert_eq!(
        fatal_diagnostics,
        &vec![format!(
            "runtime plugin module descriptor ordering failed: {missing}"
        )]
    );
    assert_eq!(diagnostics, fatal_diagnostics);
}

#[test]
#[ignore = "requires an isolated physical SDK test root; run explicitly"]
fn host_order_accepts_foundation_already_registered_by_sdk() {
    let fixture = ManagedConfig::new("present_host");
    let report = host_consumer_report();
    let built = foundation_only(&fixture)
        .with_registration_report(report.clone())
        .build();
    println!(
        "SDK_SORT_V1 case=present_host base=[Foundation] build={:?}",
        built.as_ref().map(TestRuntime::activated_modules)
    );

    // This is a separate graph control, never a substitute for the SDK result above.
    assert_product_orders_foundation(report);
    let runtime =
        built.expect("a valid SDK host dependency must reach activation in this TestRuntime");
    assert_eq!(
        runtime.activated_modules(),
        &[FOUNDATION_MODULE_NAME, CONSUMER_MODULE]
    );
    assert_eq!(extension_module_names(&runtime), [CONSUMER_MODULE]);
    assert_foundation_only_base(&runtime);
    let expected = json!("present_host");
    assert_eq!(read_fixture_config(&runtime.handle()).unwrap(), expected);
    let read = runtime
        .resolve_manager::<ConfigRead>(CONSUMER_MANAGER)
        .expect("load the consumer's real startup observation");
    assert_eq!(read.0, expected);
    drop(read);
    assert_live_edge(
        &runtime,
        "present_host",
        FOUNDATION_MODULE_NAME,
        CONSUMER_MODULE,
    );
    println!("SDK_SORT_V1 case=present_host fixture_config={expected} config_consumer=resolved");
}

fn assert_product_orders_foundation(report: RuntimePluginRegistrationReport) {
    let manifest = ProjectPluginManifest {
        selections: vec![report.project_selection.clone()],
    };
    let catalog = RuntimePluginCatalog::from_registration_reports([report], []);
    let plan = catalog.compiled_project_plan(&manifest, TARGET);
    let composition = RuntimeModuleCompositionCompiler::new(&plan)
        .compile()
        .expect("the product graph admits the same consumer report");
    let modules = composition.module_descriptors();
    let names = modules
        .iter()
        .map(|module| module.name.as_str())
        .collect::<Vec<_>>();
    println!("SDK_SORT_V1 case=present_host product_graph={names:?}");
    let foundation = module_index(&names, FOUNDATION_MODULE_NAME);
    let consumer = module_index(&names, CONSUMER_MODULE);
    assert!(foundation < consumer);
    assert!(names.contains(&ASSET_MODULE_NAME) && names.contains(&SCENE_MODULE_NAME));
    assert!(modules[foundation]
        .managers
        .iter()
        .any(|manager| manager.name.as_str() == CONFIG_MANAGER_NAME));
    assert_eq!(
        modules[consumer].module_dependencies,
        [ModuleDependencySpec::named(FOUNDATION_MODULE_NAME)]
    );
}

#[test]
#[ignore = "requires an isolated physical SDK test root; run explicitly"]
fn host_order_preserves_legacy_extra_and_sorts_two_reports() {
    let fixture = ManagedConfig::new("legacy_order");
    let consumer = registration(&descriptor(
        CONSUMER_PACKAGE,
        ModuleDescriptor::new(CONSUMER_MODULE, "SDK report-order consumer")
            .with_module_dependency(ModuleDependencySpec::named(PROVIDER_MODULE)),
    ));
    let provider = registration(&LegacyProvider(descriptor(
        PROVIDER_PACKAGE,
        ModuleDescriptor::new(PROVIDER_MODULE, "SDK report-order provider"),
    )));
    assert!(!provider
        .package_manifest
        .modules
        .iter()
        .any(|module| module.name == EXTRA_MODULE));
    assert!(provider
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == EXTRA_MODULE));
    assert!(CONSUMER_MODULE < PROVIDER_MODULE);
    // Both Post; both input order and lexical order would put the consumer first.
    let built = foundation_only(&fixture)
        .with_registration_report(consumer)
        .with_registration_report(provider)
        .build();
    println!(
        "SDK_SORT_V1 case=legacy_order input=[{CONSUMER_MODULE},{PROVIDER_MODULE}] build={:?}",
        built.as_ref().map(TestRuntime::activated_modules)
    );
    let runtime = built.expect("a legal inter-report edge and an extra module must be admitted");
    let names = extension_module_names(&runtime);
    println!("SDK_SORT_V1 case=legacy_order extension_modules={names:?}");
    assert_eq!(names.len(), 3);
    assert!(module_index(&names, PROVIDER_MODULE) < module_index(&names, CONSUMER_MODULE));
    assert!(names.contains(&EXTRA_MODULE));
    let activated = runtime
        .activated_modules()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert!(module_index(&activated, PROVIDER_MODULE) < module_index(&activated, CONSUMER_MODULE));
    assert!(activated.contains(&EXTRA_MODULE));
    runtime
        .resolve_manager::<SdkTestManager>(EXTRA_MANAGER)
        .expect("resolve the manager owned by the nonmanifest module");
    assert_live_edge(&runtime, "legacy_order", PROVIDER_MODULE, CONSUMER_MODULE);
}

#[test]
#[ignore = "requires an isolated physical SDK test root; run explicitly"]
fn host_order_builds_foundation_with_no_registrations() {
    let fixture = ManagedConfig::new("no_reports");
    let built = foundation_only(&fixture).build();
    println!(
        "SDK_SORT_V1 case=no_reports base=[Foundation] build={:?}",
        built.as_ref().map(TestRuntime::activated_modules)
    );
    let runtime = built.expect("Foundation-only with zero reports must start");
    assert_eq!(runtime.activated_modules(), &[FOUNDATION_MODULE_NAME]);
    assert!(extension_module_names(&runtime).is_empty());
    assert_foundation_only_base(&runtime);
    assert_eq!(
        runtime.runtime().deactivate_module(CONSUMER_MODULE),
        Err(CoreError::MissingModule(CONSUMER_MODULE.to_owned()))
    );
    let value = read_fixture_config(&runtime.handle()).unwrap();
    assert_eq!(value, json!("no_reports"));
    println!("SDK_SORT_V1 case=no_reports fixture_config={value}");
}

fn extension_module_names(runtime: &TestRuntime) -> Vec<&str> {
    let report = runtime.extension_report();
    assert!(report.is_success(), "{:?}", report.fatal_diagnostics);
    report
        .registry
        .modules()
        .iter()
        .map(|module| module.name.as_str())
        .collect()
}

fn module_index(names: &[&str], name: &str) -> usize {
    names
        .iter()
        .position(|entry| *entry == name)
        .unwrap_or_else(|| panic!("{name} missing from {names:?}"))
}

fn assert_foundation_only_base(runtime: &TestRuntime) {
    for name in [ASSET_MODULE_NAME, SCENE_MODULE_NAME] {
        assert_eq!(
            runtime.runtime().deactivate_module(name),
            Err(CoreError::MissingModule(name.to_owned()))
        );
    }
}

fn assert_live_edge(runtime: &TestRuntime, case: &str, provider: &str, consumer: &str) {
    let blocked = runtime
        .runtime()
        .deactivate_module(provider)
        .expect_err("Core must protect the provider while its consumer runs");
    println!("SDK_SORT_V1 case={case} unload_gate={blocked:?}");
    assert!(
        matches!(&blocked, CoreError::ModuleUnloadBlocked { module, dependents } if module == provider && dependents.iter().any(|name| name == consumer)),
        "{blocked:?}"
    );
}

// These cases never mutate cwd, environment, or a shared fixture counter.
struct ManagedConfig(ResolvedProjectPathIdentity);

impl ManagedConfig {
    fn new(case: &str) -> Self {
        const APPROVED: &str = "E:/cargo-targets/zircon-local";
        let approved =
            ProjectPaths::resolve_existing(APPROVED).expect("the managed E root must exist");
        assert!(
            approved.operation_path().is_dir()
                && approved
                    .display_path()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/")
                    .eq_ignore_ascii_case(APPROVED),
            "the approved root must resolve to the physical E location"
        );
        let approved = ResolvedProjectPathIdentity::from(approved);
        let executable = ProjectPaths::resolve_existing(std::env::current_exe().unwrap()).unwrap();
        let binary =
            ProjectPaths::resolve_identity(executable.operation_path().parent().unwrap()).unwrap();
        assert!(
            binary.is_within(&approved),
            "test executable must physically live under the managed E root"
        );
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = ProjectPaths::resolve_identity(
            binary
                .operation_path()
                .join(format!("sdk-sort-{case}-{}-{nonce}", std::process::id())),
        )
        .unwrap();
        assert!(root != binary && root.is_within(&binary));
        fs::create_dir(root.operation_path())
            .expect("reserve a new fixture directory without adopting an existing path");
        assert_eq!(
            ResolvedProjectPathIdentity::from(
                ProjectPaths::resolve_existing(root.operation_path()).unwrap()
            ),
            root
        );
        let fixture = Self(root);
        fs::File::create_new(fixture.path())
            .unwrap()
            .write_all(json!({ (CONFIG_KEY): case }).to_string().as_bytes())
            .unwrap();
        fixture
    }

    fn path(&self) -> PathBuf {
        self.0.operation_path().join("config.json")
    }
}

impl Drop for ManagedConfig {
    fn drop(&mut self) {
        // The test releases all of its Core/service guards before the fixture drops.
        assert_eq!(
            ResolvedProjectPathIdentity::from(
                ProjectPaths::resolve_existing(self.0.operation_path()).unwrap()
            ),
            self.0
        );
        fs::remove_dir_all(self.0.operation_path()).expect("remove only this owned fixture");
    }
}

mod strict_host {
    use zircon_runtime::core::{CoreRuntime, TasksModule, TASKS_MODULE_NAME};
    use zircon_runtime::engine_module::EngineModule;
    use zircon_runtime::plugin::RuntimeExtensionCatalogReport;
    use zircon_runtime::{asset, foundation};

    use super::*;

    #[test]
    #[ignore = "requires explicit host-sort candidate qualification and the managed fixture"]
    fn sdk_host_strict_projects_only_reached_host_prerequisites() {
        let fixture = ManagedConfig::new("strict_host_slice");
        let runtime = CoreRuntime::new();
        let host = runtime.handle();
        let catalog_for = |provider: &str| {
            let plugin = descriptor(
                CONSUMER_PACKAGE,
                ModuleDescriptor::new(CONSUMER_MODULE, "SDK strict host consumer")
                    .with_module_dependency(ModuleDependencySpec::named(provider)),
            );
            RuntimePluginCatalog::from_registration_reports([registration(&plugin)], [])
        };
        let via_foundation = catalog_for(FOUNDATION_MODULE_NAME);
        let via_asset = catalog_for(ASSET_MODULE_NAME);
        let missing_foundation = || CoreError::MissingModuleDependency {
            module: CONSUMER_MODULE.to_owned(),
            dependency: FOUNDATION_MODULE_NAME.to_owned(),
        };
        assert_order_failure(
            &via_foundation.runtime_extensions_for_registered_host(&host),
            missing_foundation(),
        );

        let mut foundation_descriptor = foundation::module_descriptor();
        foundation::bind_config_file_path(&mut foundation_descriptor, fixture.path()).unwrap();
        runtime.register_module(foundation_descriptor).unwrap();
        runtime.register_module(asset::module_descriptor()).unwrap();

        assert_order_success(
            &via_foundation.runtime_extensions_for_registered_host(&host),
            &[CONSUMER_MODULE],
        );
        assert_order_failure(&via_foundation.runtime_extensions(), missing_foundation());
        let reached = via_asset.runtime_extensions_for_registered_host(&host);
        println!(
            "SDK_HOST_STRICT_V1 slice reached={:?}",
            reached.fatal_diagnostics
        );
        assert_order_failure(
            &reached,
            CoreError::MissingModuleDependency {
                module: ASSET_MODULE_NAME.to_owned(),
                dependency: TASKS_MODULE_NAME.to_owned(),
            },
        );

        // The three snapshots above may not freeze this Core, even when they fail.
        runtime
            .register_module(TasksModule.descriptor())
            .expect("the same Core must still accept the real Tasks provider");
        assert_order_success(
            &via_asset.runtime_extensions_for_registered_host(&host),
            &[CONSUMER_MODULE],
        );
        assert_order_failure(
            &via_asset.runtime_extensions(),
            CoreError::MissingModuleDependency {
                module: CONSUMER_MODULE.to_owned(),
                dependency: ASSET_MODULE_NAME.to_owned(),
            },
        );
        // Resolve only after all registration/ordering probes; resolving can activate Foundation.
        let value = read_fixture_config(&host).unwrap();
        assert_eq!(value, json!("strict_host_slice"));
        println!("SDK_HOST_STRICT_V1 slice late_tasks=registered fixture_config={value}");
    }

    #[test]
    #[ignore = "requires explicit host-sort candidate qualification"]
    fn sdk_host_strict_rejects_isolated_host_name_collisions() {
        let runtime = CoreRuntime::new();
        let host = runtime.handle();
        let consumer = descriptor(
            CONSUMER_PACKAGE,
            ModuleDescriptor::new(CONSUMER_MODULE, "SDK strict host collision consumer"),
        );
        let provider = descriptor(
            PROVIDER_PACKAGE,
            ModuleDescriptor::new(PROVIDER_MODULE, "SDK strict host collision provider"),
        );
        let consumer_report = registration(&consumer);
        let provider_report = registration(&provider);
        let isolated =
            RuntimePluginCatalog::from_registration_reports([provider_report.clone()], []);
        assert_order_success(
            &isolated.runtime_extensions_for_registered_host(&host),
            &[PROVIDER_MODULE],
        );
        runtime
            .register_module(consumer.module_descriptor().clone())
            .unwrap();
        runtime
            .register_module(provider.module_descriptor().clone())
            .unwrap();

        let result = isolated.runtime_extensions_for_registered_host(&host);
        println!(
            "SDK_HOST_STRICT_V1 collision isolated={:?}",
            result.fatal_diagnostics
        );
        assert_order_failure(
            &result,
            CoreError::DuplicateModule(PROVIDER_MODULE.to_owned()),
        );
        assert_order_success(&isolated.runtime_extensions(), &[PROVIDER_MODULE]);

        // Neither name has any incoming edge. The first report, not name or Core order, wins.
        for (reports, names) in [
            (
                [provider_report.clone(), consumer_report.clone()],
                [PROVIDER_MODULE, CONSUMER_MODULE],
            ),
            (
                [consumer_report, provider_report],
                [CONSUMER_MODULE, PROVIDER_MODULE],
            ),
        ] {
            let catalog = RuntimePluginCatalog::from_registration_reports(reports, []);
            assert_order_success(&catalog.runtime_extensions(), &names);
            let result = catalog.runtime_extensions_for_registered_host(&host);
            println!(
                "SDK_HOST_STRICT_V1 collision reports={names:?} fatal={:?}",
                result.fatal_diagnostics
            );
            assert_order_failure(&result, CoreError::DuplicateModule(names[0].to_owned()));
        }
        runtime
            .register_module(foundation::module_descriptor())
            .expect("a duplicate report may not freeze the Core");
    }

    fn assert_order_success(report: &RuntimeExtensionCatalogReport, names: &[&str]) {
        assert!(report.registry.is_finalized());
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        assert!(
            report.fatal_diagnostics.is_empty(),
            "{:?}",
            report.fatal_diagnostics
        );
        let actual = report
            .registry
            .modules()
            .iter()
            .map(|module| module.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(actual.as_slice(), names);
    }

    fn assert_order_failure(report: &RuntimeExtensionCatalogReport, cause: CoreError) {
        let expected = [format!("runtime plugin module descriptor ordering failed: {cause}")];
        assert_eq!(report.fatal_diagnostics.as_slice(), &expected[..]);
        assert_eq!(report.diagnostics.as_slice(), &expected[..]);
        assert!(report.registry.is_finalized());
        assert!(report.registry.modules().is_empty());
    }

    #[test]
    #[ignore = "requires explicit host-sort candidate qualification"]
    fn sdk_host_strict_bridge_preserves_order_cycle_and_init_level() {
        const BRIDGE_MODULE: &str = "SdkOrderBridgeHost";
        let project_through_bridge = |consumer_level, closes_cycle| {
            let runtime = CoreRuntime::new();
            runtime
                .register_module(
                    ModuleDescriptor::new(BRIDGE_MODULE, "SDK strict host bridge")
                        .with_init_level(InitLevel::Post)
                        .with_module_dependency(ModuleDependencySpec::named(PROVIDER_MODULE)),
                )
                .expect("commit the host bridge before contributing either report");

            let consumer_module =
                ModuleDescriptor::new(CONSUMER_MODULE, "SDK strict bridge consumer")
                    .with_init_level(consumer_level)
                    .with_module_dependency(ModuleDependencySpec::named(BRIDGE_MODULE));
            // The shared `descriptor` helper fixes Post; preserve the early level in this probe.
            let consumer = RuntimePluginDescriptor::builder(
                CONSUMER_PACKAGE,
                consumer_module.description.clone(),
                RuntimePluginId::new(CONSUMER_PACKAGE),
                format!("zircon_plugin_{CONSUMER_PACKAGE}_runtime"),
            )
            .with_target_modes([TARGET])
            .with_capability(format!("runtime.plugin.{CONSUMER_PACKAGE}"))
            .with_required_by_default(true)
            .with_module_descriptor(consumer_module)
            .build();
            let mut provider_module =
                ModuleDescriptor::new(PROVIDER_MODULE, "SDK strict bridge provider");
            if closes_cycle {
                provider_module = provider_module
                    .with_module_dependency(ModuleDependencySpec::named(CONSUMER_MODULE));
            }
            let provider = descriptor(PROVIDER_PACKAGE, provider_module);
            RuntimePluginCatalog::from_registration_reports(
                [registration(&consumer), registration(&provider)],
                [],
            )
            .runtime_extensions_for_registered_host(&runtime.handle())
        };

        // Both input and lexical order put the consumer first; only the host bridge orders it last.
        let ordered = project_through_bridge(InitLevel::Post, false);
        println!(
            "SDK_HOST_STRICT_V1 bridge modules={:?}",
            ordered
                .registry
                .modules()
                .iter()
                .map(|module| module.name.as_str())
                .collect::<Vec<_>>()
        );
        assert_order_success(&ordered, &[PROVIDER_MODULE, CONSUMER_MODULE]);

        let cycle = project_through_bridge(InitLevel::Post, true);
        println!("SDK_HOST_STRICT_V1 bridge cycle={:?}", cycle.fatal_diagnostics);
        assert_order_failure(
            &cycle,
            CoreError::ModuleDependencyCycle {
                path: [CONSUMER_MODULE, BRIDGE_MODULE, PROVIDER_MODULE, CONSUMER_MODULE]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
        );

        let late = project_through_bridge(InitLevel::Services, false);
        println!("SDK_HOST_STRICT_V1 bridge late={:?}", late.fatal_diagnostics);
        assert_order_failure(
            &late,
            CoreError::ModuleInitLevelViolation {
                module: CONSUMER_MODULE.to_owned(),
                module_level: InitLevel::Services.as_str().to_owned(),
                dependency: BRIDGE_MODULE.to_owned(),
                dependency_level: InitLevel::Post.as_str().to_owned(),
            },
        );
    }
}
