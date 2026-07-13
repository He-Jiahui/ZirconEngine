use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
};
use crate::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor};
use crate::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry, RuntimePluginCatalog,
    RuntimePluginRegistrationReport,
};

#[test]
fn registration_report_catalog_orders_runtime_extensions_by_module_descriptor() {
    let simulation = registration_report_for_module(
        "weather_simulation",
        ModuleDescriptor::new("weather_simulation.runtime", "Weather simulation runtime")
            .with_init_level(InitLevel::Scene)
            .with_module_dependency(ModuleDependencySpec::named("weather_base.runtime")),
    );
    let base = registration_report_for_module(
        "weather_base",
        ModuleDescriptor::new("weather_base.runtime", "Weather base runtime")
            .with_init_level(InitLevel::Scene),
    );
    let catalog = RuntimePluginCatalog::from_registration_reports([simulation, base], []);

    let report = catalog.runtime_extensions();

    assert_module_order_report(
        &report,
        ["weather_base.runtime", "weather_simulation.runtime"],
    );
}

#[test]
fn registration_report_catalog_rejects_invalid_module_order_before_extension_merge() {
    let (first, second) = cyclic_registration_reports();
    let catalog = RuntimePluginCatalog::from_registration_reports([second, first], []);

    assert_invalid_order_report(&catalog.runtime_extensions());
}

#[test]
fn project_registration_report_catalog_orders_enabled_runtime_extensions() {
    let simulation = registration_report_for_module(
        "weather_simulation",
        ModuleDescriptor::new("weather_simulation.runtime", "Weather simulation runtime")
            .with_init_level(InitLevel::Scene)
            .with_module_dependency(ModuleDependencySpec::named("weather_base.runtime")),
    );
    let base = registration_report_for_module(
        "weather_base",
        ModuleDescriptor::new("weather_base.runtime", "Weather base runtime")
            .with_init_level(InitLevel::Scene),
    );
    let manifest = project_manifest_for_reports([&simulation, &base]);
    let catalog = RuntimePluginCatalog::from_registration_reports([simulation, base], []);

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert_module_order_report(
        &report,
        ["weather_base.runtime", "weather_simulation.runtime"],
    );
}

#[test]
fn project_registration_report_catalog_rejects_invalid_enabled_module_order() {
    let (first, second) = cyclic_registration_reports();
    let manifest = project_manifest_for_reports([&first, &second]);
    let catalog = RuntimePluginCatalog::from_registration_reports([second, first], []);

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert_invalid_order_report(&report);
}

fn cyclic_registration_reports() -> (
    RuntimePluginRegistrationReport,
    RuntimePluginRegistrationReport,
) {
    let first = registration_report_for_module(
        "weather_first",
        ModuleDescriptor::new("weather_first.runtime", "Weather first runtime")
            .with_init_level(InitLevel::Scene)
            .with_module_dependency(ModuleDependencySpec::named("weather_second.runtime")),
    );
    let second = registration_report_for_module(
        "weather_second",
        ModuleDescriptor::new("weather_second.runtime", "Weather second runtime")
            .with_init_level(InitLevel::Scene)
            .with_module_dependency(ModuleDependencySpec::named("weather_first.runtime")),
    );
    (first, second)
}

fn project_manifest_for_reports<'a>(
    reports: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: reports
            .into_iter()
            .map(|report| report.project_selection.clone())
            .collect(),
    }
}

fn registration_report_for_module(
    package_id: &str,
    module: ModuleDescriptor,
) -> RuntimePluginRegistrationReport {
    let mut registry = RuntimeExtensionRegistry::default();
    registry.register_module(module.clone()).unwrap();
    RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new(package_id, package_id)
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_capability(format!("runtime.plugin.{package_id}"))
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    module.name.clone(),
                    format!("zircon_plugin_{package_id}_runtime"),
                )
                .with_init_level(module.init_level)
                .with_module_dependencies(module.module_dependencies.clone())
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities([format!("runtime.plugin.{package_id}")]),
            ),
        project_selection: ProjectPluginSelection {
            id: package_id.to_string(),
            enabled: true,
            required: false,
            target_modes: vec![RuntimeTargetMode::ClientRuntime],
            packaging: ExportPackagingStrategy::SourceTemplate,
            runtime_crate: Some(format!("zircon_plugin_{package_id}_runtime")),
            editor_crate: None,
            features: Vec::new(),
        },
        extensions: registry,
        diagnostics: Vec::new(),
    }
}

fn assert_module_order_report<const N: usize>(
    report: &crate::plugin::RuntimeExtensionCatalogReport,
    expected: [&str; N],
) {
    assert!(report.is_success(), "{:?}", report.fatal_diagnostics);
    assert_eq!(
        report
            .registry
            .modules()
            .iter()
            .map(|descriptor| descriptor.name.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}

fn assert_invalid_order_report(report: &crate::plugin::RuntimeExtensionCatalogReport) {
    assert!(!report.is_success());
    assert!(report.registry.modules().is_empty());
    assert_eq!(report.fatal_diagnostics.len(), 1);
    assert!(
        report.fatal_diagnostics[0].contains("runtime plugin module descriptor ordering failed")
    );
    assert!(report.fatal_diagnostics[0].contains("module dependency cycle"));
}
