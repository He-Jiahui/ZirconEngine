use crate::builtin::{
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports,
    runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports,
    runtime_modules_for_target_with_linked_plugins,
    runtime_modules_for_target_with_plugin_and_feature_registration_reports, RuntimePluginId,
    RuntimeTargetMode,
};
use crate::plugin::{
    PluginModuleManifest, PluginPackageManifest, ProjectPluginManifest, ProjectPluginSelection,
    RuntimePluginAvailabilityCategory, RuntimePluginRegistrationReport, RuntimeProfileId,
};

use super::support::{availability_contains, linked_runtime_registration};

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
    );

    assert!(availability_contains(
        &report.runtime_plugin_availability.linked,
        RuntimePluginId::VirtualGeometry
    ));
    assert!(report.runtime_plugin_availability.contains(
        RuntimePluginAvailabilityCategory::Linked,
        RuntimePluginId::VirtualGeometry
    ));
    assert_eq!(
        report
            .runtime_plugin_availability
            .category_count(RuntimePluginAvailabilityCategory::Linked),
        1
    );
    assert_eq!(
        report
            .runtime_plugin_availability
            .entry_for(
                RuntimePluginAvailabilityCategory::Linked,
                RuntimePluginId::VirtualGeometry
            )
            .map(|entry| entry.id.as_str()),
        Some(RuntimePluginId::VirtualGeometry.key())
    );
    let diagnostic_lines = report.runtime_plugin_availability.diagnostic_lines();
    assert!(diagnostic_lines
        .iter()
        .any(|line| line == "runtime_plugin_availability.linked.count=1"));
    assert!(diagnostic_lines
        .iter()
        .any(|line| line.contains("runtime_plugin_availability.linked=virtual_geometry")));
    assert!(!availability_contains(
        &report.runtime_plugin_availability.missing_required,
        RuntimePluginId::VirtualGeometry
    ));
    assert!(!report.runtime_plugin_availability.has_missing_required());
    assert!(report.effective_required_missing().is_empty());
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
    );

    assert!(report.runtime_plugin_availability.contains(
        RuntimePluginAvailabilityCategory::NativeDynamic,
        RuntimePluginId::VirtualGeometry
    ));
    assert!(!report.runtime_plugin_availability.contains(
        RuntimePluginAvailabilityCategory::Linked,
        RuntimePluginId::VirtualGeometry
    ));
    assert!(report.effective_required_missing().is_empty());
}

#[test]
fn target_required_missing_is_deduped_between_legacy_and_structured_reports() {
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
    );
    let missing = report.effective_required_missing();

    assert_eq!(
        missing
            .iter()
            .filter(|entry| entry.id == RuntimePluginId::VirtualGeometry)
            .count(),
        1
    );
    assert!(report
        .effective_errors()
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
    );

    assert!(availability_contains(
        &report.runtime_plugin_availability.linked,
        RuntimePluginId::Sound
    ));
    assert!(!availability_contains(
        &report.runtime_plugin_availability.missing_required,
        RuntimePluginId::Sound
    ));
    assert!(!report
        .effective_required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::Sound));
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
        );

    assert!(availability_contains(
        &report.runtime_plugin_availability.linked,
        RuntimePluginId::Animation
    ));
    assert!(!availability_contains(
        &report.runtime_plugin_availability.externalized_missing,
        RuntimePluginId::Animation
    ));
}

#[test]
fn runtime_module_assembly_keeps_registration_input_aggregation_in_child_owner() {
    let assembly_source = include_str!("../assembly.rs");
    let registration_inputs_source = include_str!("../assembly/registration_inputs.rs");
    let target_modules_source = include_str!("../assembly/target_modules.rs");

    assert!(assembly_source.contains("mod registration_inputs;"));
    assert!(assembly_source.contains("mod target_modules;"));
    assert!(registration_inputs_source
        .contains("pub(super) fn registration_inputs_for_plugin_and_feature_reports"));
    assert!(target_modules_source
        .contains("pub(super) fn runtime_modules_for_target_with_registration_inputs_for_manifest"));
    assert!(target_modules_source.contains("RuntimeRequiredPluginMissing"));
    assert!(target_modules_source.contains("module_for_plugin"));
    assert!(!assembly_source.contains("asset_importers_from_extension_registries"));
    assert!(!assembly_source.contains("RuntimeExtensionRegistry"));
    assert!(!assembly_source.contains(".extensions"));
    assert!(!assembly_source.contains("manifest.enabled_for_target"));
    assert!(!assembly_source.contains("module_for_plugin"));
    assert!(!assembly_source.contains("RuntimeRequiredPluginMissing"));
}
