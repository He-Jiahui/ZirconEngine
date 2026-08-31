use crate::asset::{AssetImporterDescriptor, AssetKind};
use crate::builtin::{
    runtime_modules_for_compiled_project_plugin_plan,
    runtime_modules_for_runtime_profile_compiled_project_plugin_plan,
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports,
    runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports,
    runtime_modules_for_target, runtime_modules_for_target_with_linked_plugins,
    runtime_modules_for_target_with_plugin_and_feature_registration_reports, RuntimePluginId,
};
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use crate::core::{sort_module_activation_order, ModuleDependencySpec, ModuleDescriptor};
use crate::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimePluginAvailabilityCategory, RuntimePluginCatalog, RuntimePluginRegistrationReport,
};

use super::super::support::{availability_contains, linked_runtime_registration};

const MANIFEST_FILTER_IMPORTER_ID: &str = "runtime42.manifest_filter_fixture";

fn registration_with_manifest_filter_importer(
    plugin_id: RuntimePluginId,
) -> RuntimePluginRegistrationReport {
    let mut registration = linked_runtime_registration(plugin_id.clone());
    registration
        .extensions
        .register_asset_importer_descriptor(
            AssetImporterDescriptor::new(
                MANIFEST_FILTER_IMPORTER_ID,
                plugin_id.key(),
                AssetKind::Data,
                1,
            )
            .with_source_extensions(["runtime42_manifest_filter"]),
        )
        .expect("manifest filter fixture importer registers");
    registration
}

fn registration_with_modules(
    plugin_id: RuntimePluginId,
    modules: impl IntoIterator<Item = ModuleDescriptor>,
) -> RuntimePluginRegistrationReport {
    let mut registration = linked_runtime_registration(plugin_id);
    for descriptor in modules {
        registration.package_manifest.modules.push(
            PluginModuleManifest::runtime(
                descriptor.name.clone(),
                format!("{}_runtime", descriptor.name),
            )
            .with_init_level(descriptor.init_level)
            .with_module_dependencies(descriptor.module_dependencies.clone()),
        );
        registration
            .extensions
            .register_module(descriptor)
            .expect("module fixture registers");
    }
    registration
}

#[test]
fn compiled_project_plan_assembles_only_target_selected_provider_modules() {
    let mut extensions = RuntimeExtensionRegistry::default();
    extensions
        .register_module(crate::core::ModuleDescriptor::new(
            "sound.client",
            "Client sound runtime",
        ))
        .unwrap();
    extensions
        .register_module(crate::core::ModuleDescriptor::new(
            "sound.server",
            "Server sound runtime",
        ))
        .unwrap();
    let registration = RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new("sound", "Sound")
            .with_supported_targets([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::ServerRuntime,
            ])
            .with_runtime_module(
                PluginModuleManifest::runtime("sound.client", "sound_client")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime]),
            )
            .with_runtime_module(
                PluginModuleManifest::runtime("sound.server", "sound_server")
                    .with_target_modes([RuntimeTargetMode::ServerRuntime]),
            ),
        project_selection: ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            true,
        ),
        extensions,
        diagnostics: Vec::new(),
    };
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            true,
        )],
    };
    let catalog =
        RuntimePluginCatalog::from_registration_reports([registration.clone()], std::iter::empty());
    let plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);

    let report = runtime_modules_for_compiled_project_plugin_plan(&plan)
        .expect("compiled project plan should assemble");
    let module_names = report
        .modules()
        .iter()
        .map(|module| module.module_name())
        .collect::<Vec<_>>();

    assert!(module_names.contains(&"sound.client"));
    assert!(!module_names.contains(&"sound.server"));
    assert!(report.runtime_plugin_availability().contains(
        RuntimePluginAvailabilityCategory::Linked,
        RuntimePluginId::Sound
    ));
    assert_eq!(catalog.project_plan_metrics().project_plan_builds, 1);
}

#[test]
fn compiled_project_plan_topologically_sorts_the_combined_module_graph() {
    const FIRST: &str = "runtime42.order.first";
    const MIDDLE: &str = "runtime42.order.middle";
    const LAST: &str = "runtime42.order.last";

    let first_and_last = registration_with_modules(
        RuntimePluginId::Sound,
        [
            ModuleDescriptor::new(FIRST, "First runtime module"),
            ModuleDescriptor::new(LAST, "Last runtime module")
                .with_module_dependency(ModuleDependencySpec::named(MIDDLE)),
        ],
    );
    let middle = registration_with_modules(
        RuntimePluginId::Animation,
        [ModuleDescriptor::new(MIDDLE, "Middle runtime module")
            .with_module_dependency(ModuleDependencySpec::named(FIRST))],
    );
    let manifest = ProjectPluginManifest {
        selections: vec![
            first_and_last.project_selection.clone(),
            middle.project_selection.clone(),
        ],
    };
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [first_and_last, middle],
        std::iter::empty(),
    );
    let plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);

    let report = runtime_modules_for_compiled_project_plugin_plan(&plan)
        .expect("combined module graph should compile");
    let plugin_module_names = report
        .modules()
        .iter()
        .map(|module| module.module_name())
        .filter(|name| name.starts_with("runtime42.order."))
        .collect::<Vec<_>>();

    assert_eq!(plugin_module_names, vec![FIRST, MIDDLE, LAST]);
}

#[test]
fn compiled_project_plan_rejects_a_mismatched_runtime_profile_target() {
    let catalog =
        RuntimePluginCatalog::from_registration_reports(std::iter::empty(), std::iter::empty());
    let plan = catalog.compiled_project_plan(
        &ProjectPluginManifest::default(),
        RuntimeTargetMode::ClientRuntime,
    );

    let report = runtime_modules_for_runtime_profile_compiled_project_plugin_plan(
        RuntimeProfileId::Server,
        &plan,
    )
    .expect_err("profile target mismatch must reject the composition");

    assert!(report
        .fatal_messages()
        .iter()
        .any(|message| message.contains("runtime plugin plan target mismatch")));
}

#[test]
fn target_linked_plugin_report_surfaces_structured_availability() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::VirtualGeometry,
            true,
            true,
        )],
    };
    let report = runtime_modules_for_target_with_linked_plugins(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        [RuntimePluginId::VirtualGeometry.key()],
    )
    .expect("linked target modules should compile");

    assert!(availability_contains(
        &report.runtime_plugin_availability().linked,
        RuntimePluginId::VirtualGeometry
    ));
    assert!(report.runtime_plugin_availability().contains(
        RuntimePluginAvailabilityCategory::Linked,
        RuntimePluginId::VirtualGeometry
    ));
    assert_eq!(
        report
            .runtime_plugin_availability()
            .category_count(RuntimePluginAvailabilityCategory::Linked),
        1
    );
    assert_eq!(
        report
            .runtime_plugin_availability()
            .entry_for(
                RuntimePluginAvailabilityCategory::Linked,
                RuntimePluginId::VirtualGeometry
            )
            .map(|entry| entry.id.as_str()),
        Some(RuntimePluginId::VirtualGeometry.key())
    );
    let diagnostic_lines = report.runtime_plugin_availability().diagnostic_lines();
    assert!(diagnostic_lines
        .iter()
        .any(|line| line == "runtime_plugin_availability.linked.count=1"));
    assert!(diagnostic_lines
        .iter()
        .any(|line| line.contains("runtime_plugin_availability.linked=virtual_geometry")));
    assert!(!availability_contains(
        &report.runtime_plugin_availability().missing_required,
        RuntimePluginId::VirtualGeometry
    ));
    assert!(!report.runtime_plugin_availability().has_missing_required());
}

#[cfg(feature = "ui")]
#[test]
fn target_module_loading_does_not_treat_selection_spelling_as_provider_identity() {
    let mut selection = ProjectPluginSelection::runtime_plugin(RuntimePluginId::Ui, true, true);
    selection.id = "UI".to_string();
    let manifest = ProjectPluginManifest {
        selections: vec![selection],
    };

    let report = runtime_modules_for_target_with_linked_plugins(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        ["UI"],
    )
    .expect("canonical UI provider should compile");

    assert!(report.runtime_plugin_availability().contains(
        RuntimePluginAvailabilityCategory::Available,
        RuntimePluginId::Ui
    ));
    assert!(report
        .modules()
        .iter()
        .any(|module| { module.module_name() == crate::core::framework::ui::UI_MODULE_NAME }));
}

#[test]
fn target_native_dynamic_registration_report_preserves_availability_category() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::VirtualGeometry,
            true,
            true,
        )],
    };
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("virtual_geometry", "Virtual Geometry")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.virtual_geometry")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "virtual_geometry.runtime",
                    "zircon_plugin_virtual_geometry_runtime",
                )
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.plugin.virtual_geometry"]),
            ),
    );

    let report = runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        [&registration],
        std::iter::empty(),
    )
    .expect("native dynamic provider should compile");

    assert!(report.runtime_plugin_availability().contains(
        RuntimePluginAvailabilityCategory::NativeDynamic,
        RuntimePluginId::VirtualGeometry
    ));
    assert!(!report.runtime_plugin_availability().contains(
        RuntimePluginAvailabilityCategory::Linked,
        RuntimePluginId::VirtualGeometry
    ));

    let catalog =
        RuntimePluginCatalog::from_registration_reports([registration.clone()], std::iter::empty());
    let plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);
    let compiled_report = runtime_modules_for_compiled_project_plugin_plan(&plan)
        .expect("compiled native dynamic provider should assemble");

    assert!(compiled_report.runtime_plugin_availability().contains(
        RuntimePluginAvailabilityCategory::NativeDynamic,
        RuntimePluginId::VirtualGeometry
    ));
    assert!(!compiled_report.runtime_plugin_availability().contains(
        RuntimePluginAvailabilityCategory::Linked,
        RuntimePluginId::VirtualGeometry
    ));
}

#[test]
fn target_registration_extensions_follow_effective_project_manifest() {
    let selected = registration_with_manifest_filter_importer(RuntimePluginId::Sound);
    let unselected = registration_with_manifest_filter_importer(RuntimePluginId::Animation);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    };

    let report = runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        [&selected, &unselected],
        std::iter::empty(),
    )
    .expect("selected extension should compile");

    assert!(report
        .diagnostics()
        .iter()
        .all(|diagnostic| !diagnostic.message().contains(MANIFEST_FILTER_IMPORTER_ID)));
}

#[test]
fn disabled_manifest_registration_cannot_contribute_extensions() {
    let selected = registration_with_manifest_filter_importer(RuntimePluginId::Sound);
    let disabled = registration_with_manifest_filter_importer(RuntimePluginId::Animation);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, false, false),
        ],
    };

    let report = runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        [&selected, &disabled],
        std::iter::empty(),
    )
    .expect("disabled extension should not reject composition");

    assert!(report
        .diagnostics()
        .iter()
        .all(|diagnostic| !diagnostic.message().contains(MANIFEST_FILTER_IMPORTER_ID)));
}

#[test]
fn selected_registration_extension_conflicts_remain_diagnostics() {
    let sound = registration_with_manifest_filter_importer(RuntimePluginId::Sound);
    let animation = registration_with_manifest_filter_importer(RuntimePluginId::Animation);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };

    let report = runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        [&sound, &animation],
        std::iter::empty(),
    )
    .expect_err("conflicting selected extensions must reject composition");

    assert!(report
        .fatal_messages()
        .iter()
        .any(|message| message.contains(MANIFEST_FILTER_IMPORTER_ID)));
}

#[test]
fn compiled_project_plan_extension_conflicts_are_fatal_module_load_diagnostics() {
    let sound = registration_with_manifest_filter_importer(RuntimePluginId::Sound);
    let animation = registration_with_manifest_filter_importer(RuntimePluginId::Animation);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound.clone(), animation.clone()],
        std::iter::empty(),
    );
    let plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);

    let report = runtime_modules_for_compiled_project_plugin_plan(&plan)
        .expect_err("compiled extension conflict must reject composition");

    assert!(report
        .fatal_messages()
        .iter()
        .any(|message| message.contains(MANIFEST_FILTER_IMPORTER_ID)));
}

#[test]
fn registration_extension_admission_uses_canonical_runtime_plugin_identity() {
    let sound = registration_with_manifest_filter_importer(RuntimePluginId::Sound);
    let animation = registration_with_manifest_filter_importer(RuntimePluginId::Animation);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin("audio", true, false),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };

    let report = runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        [&sound, &animation],
        std::iter::empty(),
    )
    .expect_err("canonical extension conflict must reject composition");

    assert!(report
        .fatal_messages()
        .iter()
        .any(|message| message.contains(MANIFEST_FILTER_IMPORTER_ID)));
}

#[test]
fn target_required_missing_has_one_structured_availability_owner() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::VirtualGeometry,
            true,
            true,
        )],
    };
    let report = runtime_modules_for_target_with_linked_plugins(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        std::iter::empty::<String>(),
    )
    .expect_err("missing required provider must reject composition");
    let missing = report.required_missing();

    assert_eq!(
        missing
            .iter()
            .filter(|entry| entry.runtime_id == RuntimePluginId::VirtualGeometry)
            .count(),
        1
    );
    assert!(report
        .fatal_messages()
        .iter()
        .any(|diagnostic| diagnostic.contains("required runtime plugin VirtualGeometry")));
}

#[test]
fn runtime_profile_plugin_and_feature_bootstrap_uses_profile_availability() {
    let sound_registration = linked_runtime_registration(RuntimePluginId::Sound);
    let report = runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports(
        RuntimeProfileId::Client2d,
        [&sound_registration],
        std::iter::empty::<&crate::plugin::RuntimePluginFeatureRegistrationReport>(),
    )
    .expect("profile providers should compile");

    assert!(availability_contains(
        &report.runtime_plugin_availability().linked,
        RuntimePluginId::Sound
    ));
    assert!(!availability_contains(
        &report.runtime_plugin_availability().missing_required,
        RuntimePluginId::Sound
    ));
}

#[test]
fn runtime_profile_manifest_bootstrap_reports_manifest_optional_provider_availability() {
    let profile = crate::plugin::RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client3d);
    let mut manifest = profile.project_manifest();
    manifest
        .selections
        .push(ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Animation,
            true,
            false,
        ));
    let animation_registration = linked_runtime_registration(RuntimePluginId::Animation);

    let report =
        runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports(
            RuntimeProfileId::Client3d,
            &manifest,
            [&animation_registration],
            std::iter::empty::<&crate::plugin::RuntimePluginFeatureRegistrationReport>(),
        )
        .expect("profile manifest providers should compile");

    assert!(availability_contains(
        &report.runtime_plugin_availability().linked,
        RuntimePluginId::Animation
    ));
    assert!(!availability_contains(
        &report.runtime_plugin_availability().externalized_missing,
        RuntimePluginId::Animation
    ));
}

#[test]
fn target_runtime_modules_follow_descriptor_activation_order() {
    let report = runtime_modules_for_target(
        RuntimeTargetMode::ServerRuntime,
        Some(&ProjectPluginManifest::default()),
    )
    .expect("server runtime module selection should compile");

    let module_names = report
        .modules()
        .iter()
        .map(|module| module.module_name())
        .collect::<Vec<_>>();
    let descriptors = report
        .modules()
        .iter()
        .map(|module| module.descriptor())
        .collect::<Vec<_>>();
    let sorted_names = sort_module_activation_order(&descriptors).unwrap();

    assert_eq!(
        module_names,
        sorted_names.iter().map(String::as_str).collect::<Vec<_>>()
    );
    assert_eq!(
        module_names,
        vec![
            crate::core::framework::foundation::FOUNDATION_MODULE_NAME,
            crate::core::runtime::modules::LOG_MODULE_NAME,
            crate::core::runtime::modules::TASKS_MODULE_NAME,
            crate::core::runtime::modules::TIME_MODULE_NAME,
            crate::core::runtime::modules::FRAME_COUNT_MODULE_NAME,
            crate::core::runtime::modules::DIAGNOSTICS_CORE_MODULE_NAME,
            crate::core::framework::platform::PLATFORM_MODULE_NAME,
            crate::core::framework::input::INPUT_MODULE_NAME,
            crate::asset::ASSET_MODULE_NAME,
            crate::core::framework::scene::SCENE_MODULE_NAME,
        ]
    );
}
