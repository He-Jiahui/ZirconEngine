#[test]
fn runtime_module_assembly_keeps_specialized_flows_in_child_owners() {
    let runtime_modules_source = include_str!("../../../runtime_modules.rs");
    let builtin_source = include_str!("../../../mod.rs");
    let registration_mod_source = include_str!("mod.rs");
    let behavior_source = include_str!("behavior.rs");
    let assembly_source = include_str!("../../assembly.rs");
    let availability_source = include_str!("../../availability.rs");
    let core_modules_source = include_str!("../../core_modules.rs");
    let extension_inputs_source = include_str!("../../assembly/extension_inputs.rs");
    let feature_reports_source = include_str!("../../assembly/feature_reports.rs");
    let ids_source = include_str!("../../ids.rs");
    let manifest_source = include_str!("../../manifest.rs");
    let plugin_id_source = include_str!("../../ids/plugin_id.rs");
    let profile_modules_source = include_str!("../../assembly/profile_modules.rs");
    let plugin_modules_source = include_str!("../../plugin_modules.rs");
    let plugin_module_loader_source = include_str!("../../plugin_modules/loader.rs");
    let registration_inputs_source = include_str!("../../assembly/registration_inputs.rs");
    let registration_reports_source = include_str!("../../assembly/registration_reports.rs");
    let load_report_source = include_str!("../../load_report.rs");
    let load_report_diagnostics_source = include_str!("../../load_report/diagnostics.rs");
    let load_report_report_source = include_str!("../../load_report/report.rs");
    let target_mode_source =
        include_str!("../../../../core/framework/platform/runtime_target_mode.rs");
    let target_modules_source = include_str!("../../assembly/target_modules.rs");

    assert!(registration_mod_source.contains("mod behavior;"));
    assert!(registration_mod_source.contains("mod structure;"));
    assert!(!registration_mod_source.contains("#[test]"));
    assert!(
        behavior_source.contains("target_linked_plugin_report_surfaces_structured_availability")
    );
    assert!(behavior_source.contains(
        "runtime_profile_manifest_bootstrap_reports_manifest_optional_provider_availability"
    ));
    assert!(!behavior_source.contains("include_str!"));

    assert!(runtime_modules_source.contains("mod assembly;"));
    assert!(runtime_modules_source.contains("mod availability;"));
    assert!(runtime_modules_source.contains("mod core_modules;"));
    assert!(runtime_modules_source.contains("mod ids;"));
    assert!(runtime_modules_source.contains("mod load_report;"));
    assert!(runtime_modules_source.contains("mod manifest;"));
    assert!(runtime_modules_source.contains("mod plugin_modules;"));
    assert!(
        runtime_modules_source.contains("use crate::core::framework::platform::RuntimeTargetMode;")
    );
    assert!(!runtime_modules_source.contains("pub use ids::{RuntimePluginId, RuntimeTargetMode};"));
    assert!(!builtin_source.contains("RuntimePluginId, RuntimeTargetMode"));
    assert!(!runtime_modules_source.contains("mod extensions;"));
    assert!(assembly_source.contains("mod extension_inputs;"));
    assert!(assembly_source.contains("mod feature_reports;"));
    assert!(assembly_source.contains("mod profile_modules;"));
    assert!(assembly_source.contains("mod registration_inputs;"));
    assert!(assembly_source.contains("mod registration_reports;"));
    assert!(assembly_source.contains("mod target_modules;"));
    assert!(assembly_source.contains("use crate::core::framework::platform::RuntimeTargetMode;"));
    assert!(assembly_source.contains("use super::load_report::RuntimeModuleLoadReport;"));
    assert!(
        availability_source.contains("use crate::core::framework::platform::RuntimeTargetMode;")
    );
    assert!(
        core_modules_source.contains("use crate::core::framework::platform::RuntimeTargetMode;")
    );
    assert!(manifest_source.contains("use super::ids::RuntimePluginId;"));
    assert!(manifest_source.contains("use crate::core::framework::platform::RuntimeTargetMode;"));
    assert!(extension_inputs_source.contains("RuntimeModuleExtensionInputs"));
    assert!(extension_inputs_source.contains("extension_inputs_from_extension_registries"));
    assert!(extension_inputs_source.contains("RuntimeExtensionRegistry"));
    assert!(extension_inputs_source.contains("asset_importers_from_extension_registries"));
    assert!(!extension_inputs_source.contains("super::super::extensions"));
    assert!(!extension_inputs_source.contains("runtime_modules::extensions"));
    assert!(extension_inputs_source.contains("collect_render_features"));
    assert!(extension_inputs_source.contains("collect_shading_models"));
    assert!(extension_inputs_source.contains("collect_render_pass_executors"));
    assert!(extension_inputs_source.contains("collect_runtime_prepare_collectors"));
    assert!(extension_inputs_source.contains("collect_hybrid_gi_runtime_providers"));
    assert!(extension_inputs_source.contains("collect_solari_runtime_providers"));
    assert!(extension_inputs_source.contains("collect_virtual_geometry_runtime_providers"));
    assert!(feature_reports_source
        .contains("pub(super) fn feature_reports_for_plugin_and_feature_registration_reports"));
    assert!(feature_reports_source.contains("RuntimePluginCatalog::from_registration_reports"));
    assert!(feature_reports_source.contains("feature_dependency_report"));
    assert!(feature_reports_source.contains("blocked_features"));
    assert!(feature_reports_source.contains("active_feature_registration_refs"));
    assert!(
        feature_reports_source.contains("use crate::core::framework::platform::RuntimeTargetMode;")
    );
    assert!(feature_reports_source.contains(
        "use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};"
    ));
    assert!(ids_source.contains("mod plugin_id;"));
    assert!(ids_source.contains("pub use plugin_id::RuntimePluginId;"));
    assert!(!ids_source.contains("target_mode"));
    assert!(plugin_id_source.contains("pub struct RuntimePluginId"));
    assert!(!plugin_id_source.contains("pub enum RuntimePluginId"));
    assert!(plugin_id_source.contains("pub fn key"));
    assert!(plugin_id_source.contains("pub fn label"));
    assert!(plugin_id_source.contains("pub fn parse_key"));
    assert!(plugin_id_source.contains("Static(&'static str)"));
    assert!(plugin_id_source.contains("Dynamic(Arc<str>)"));
    assert!(!plugin_id_source.contains(concat!("Box", "::leak")));
    assert!(!plugin_id_source.contains(concat!("static INTERNED", "_KEYS")));
    assert!(target_mode_source.contains("pub enum RuntimeTargetMode"));
    assert!(profile_modules_source.contains("pub(super) fn runtime_modules_for_runtime_profile"));
    assert!(profile_modules_source.contains("minimal_profile_runtime_modules"));
    assert!(profile_modules_source.contains("runtime_profile_availability"));
    assert!(profile_modules_source.contains("runtime_profile_manifest_availability"));
    assert!(profile_modules_source.contains("RuntimeProfileDescriptor"));
    assert!(profile_modules_source.contains("RuntimeModuleRegistrationInputs::empty"));
    assert!(
        profile_modules_source.contains("use super::super::load_report::RuntimeModuleLoadReport;")
    );
    assert!(registration_inputs_source
        .contains("pub(super) fn registration_inputs_for_plugin_and_feature_reports"));
    assert!(registration_inputs_source.contains("from_extension_inputs"));
    assert!(!registration_inputs_source.contains("package_manifest.id.as_str()"));
    assert!(registration_inputs_source.contains("extension_inputs_from_extension_registries"));
    assert!(registration_reports_source
        .contains("pub(super) fn runtime_modules_for_target_with_plugin_registration_reports"));
    assert!(registration_reports_source.contains(
        "pub(super) fn runtime_modules_for_profile_manifest_with_plugin_registration_reports"
    ));
    assert!(registration_reports_source.contains(
        "pub(super) fn runtime_modules_for_target_with_plugin_and_feature_registration_reports"
    ));
    assert!(registration_reports_source.contains("active_plugin_registration_refs"));
    assert!(registration_reports_source.contains("extend_asset_importer_errors"));
    assert!(registration_reports_source
        .contains("target_manifest_availability_for_registration_reports"));
    assert!(registration_reports_source
        .contains("use crate::core::framework::platform::RuntimeTargetMode;"));
    assert!(registration_reports_source.contains(
        "use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};"
    ));
    assert!(target_modules_source.contains(
        "pub(super) fn runtime_modules_for_target_with_registration_inputs_for_manifest"
    ));
    assert!(target_modules_source.contains("module_for_plugin"));
    assert!(
        target_modules_source.contains("use crate::core::framework::platform::RuntimeTargetMode;")
    );
    assert!(target_modules_source.contains(
        "use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};"
    ));
    assert!(plugin_modules_source.contains("mod loader;"));
    assert!(!plugin_modules_source.contains("mod availability;"));
    assert!(plugin_modules_source.contains("pub(super) use loader::module_for_plugin;"));
    assert!(target_modules_source.contains("RuntimePluginAvailabilityCategory::Linked"));
    assert!(target_modules_source.contains("RuntimePluginAvailabilityCategory::NativeDynamic"));
    assert!(!target_modules_source.contains("linked_plugin_is_available"));
    assert!(plugin_module_loader_source.contains("module_for_plugin"));
    assert!(!plugin_module_loader_source.contains("externalized_runtime_plugin_module"));
    assert!(!plugin_module_loader_source.contains("externalized_runtime_plugin_message"));
    assert!(plugin_module_loader_source.contains("use super::super::ids::RuntimePluginId;"));
    assert!(load_report_source.contains("mod diagnostics;"));
    assert!(load_report_source.contains("mod report;"));
    assert!(!load_report_source.contains("mod missing;"));
    assert!(load_report_source.contains("pub use diagnostics::RuntimeModuleLoadDiagnostic;"));
    assert!(load_report_source.contains("pub use report::RuntimeModuleLoadReport;"));
    assert!(load_report_report_source.contains("pub struct RuntimeModuleLoadReport"));
    assert!(load_report_report_source.contains("diagnostics: Vec<RuntimeModuleLoadDiagnostic>"));
    assert!(!load_report_report_source.contains("pub warnings: Vec<String>"));
    assert!(!load_report_report_source.contains("pub errors: Vec<String>"));
    assert!(!load_report_report_source.contains("required_missing:"));
    assert!(load_report_diagnostics_source.contains("required_missing_summary"));
    assert!(load_report_diagnostics_source.contains("fatal_messages"));
    assert!(load_report_diagnostics_source.contains("warning_messages"));
    assert!(!load_report_diagnostics_source.contains("effective_required_missing"));
    assert!(!load_report_diagnostics_source.contains("effective_errors"));
    assert!(!target_modules_source.contains("report.push_required_missing"));
    assert!(!plugin_modules_source.contains("use std::collections::HashSet;"));
    assert!(!plugin_modules_source.contains("use std::sync::Arc;"));
    assert!(!plugin_modules_source.contains("fn externalized_runtime_plugin_message"));
    assert!(!plugin_modules_source.contains("match id"));
    assert!(!ids_source.contains("pub enum RuntimePluginId"));
    assert!(!ids_source.contains("pub enum RuntimeTargetMode"));
    assert!(!ids_source.contains("parse_key"));
    assert!(!ids_source.contains("match self"));
    assert!(!assembly_source.contains("use super::{RuntimeModuleLoadReport, RuntimeTargetMode};"));
    assert!(!availability_source.contains("use super::RuntimeTargetMode;"));
    assert!(!core_modules_source.contains("use super::RuntimeTargetMode;"));
    assert!(!manifest_source.contains("use super::{RuntimePluginId, RuntimeTargetMode};"));
    assert!(!feature_reports_source
        .contains("use super::super::{RuntimeModuleLoadReport, RuntimeTargetMode};"));
    assert!(!profile_modules_source.contains("use super::super::RuntimeModuleLoadReport;"));
    assert!(!registration_reports_source
        .contains("use super::super::{RuntimeModuleLoadReport, RuntimeTargetMode};"));
    assert!(!target_modules_source.contains("use super::super::{RuntimeModuleLoadReport,"));
    assert!(!plugin_module_loader_source.contains("use super::super::RuntimePluginId;"));
    assert!(!load_report_source.contains("pub struct RuntimeModuleLoadReport"));
    assert!(!load_report_source.contains("pub struct RuntimeRequiredPluginMissing"));
    assert!(!load_report_source.contains("effective_errors"));
    assert!(!target_modules_source.contains("required_missing.push"));
    assert!(!target_modules_source.contains("RuntimeRequiredPluginMissing"));
    assert!(!assembly_source.contains("asset_importers_from_extension_registries"));
    assert!(!assembly_source.contains("RuntimeExtensionRegistry"));
    assert!(!assembly_source.contains(".extensions"));
    assert!(!assembly_source.contains("manifest.enabled_for_target"));
    assert!(!assembly_source.contains("module_for_plugin"));
    assert!(!assembly_source.contains("RuntimeRequiredPluginMissing"));
    assert!(!assembly_source.contains("RuntimePluginCatalog"));
    assert!(!assembly_source.contains("feature_dependency_report"));
    assert!(!assembly_source.contains("blocked_features"));
    assert!(!assembly_source.contains("active_feature_registration_refs"));
    assert!(!assembly_source.contains("target_manifest_availability_for_registration_reports"));
    assert!(!assembly_source.contains("active_plugin_registration_refs"));
    assert!(!assembly_source.contains("asset_importer_errors"));
    assert!(!assembly_source.contains("minimal_profile_runtime_modules"));
    assert!(!assembly_source.contains("runtime_profile_availability"));
    assert!(!assembly_source.contains("runtime_profile_manifest_availability"));
    assert!(!assembly_source.contains("RuntimeProfileDescriptor"));
    assert!(!assembly_source.contains("RuntimeModuleRegistrationInputs::empty"));
    assert!(!registration_inputs_source.contains("RuntimeTargetMode"));
    assert!(!registration_inputs_source.contains("project_selection.supports_target"));
    assert!(!registration_inputs_source.contains("RuntimeExtensionRegistry"));
    assert!(!registration_inputs_source.contains("asset_importers_from_extension_registries"));
    assert!(!registration_inputs_source.contains("collect_render_features"));
    assert!(!registration_inputs_source.contains("collect_shading_models"));
    assert!(!registration_inputs_source.contains("collect_render_pass_executors"));
    assert!(!registration_inputs_source.contains("collect_runtime_prepare_collectors"));
    assert!(!registration_inputs_source.contains("collect_hybrid_gi_runtime_providers"));
    assert!(!registration_inputs_source.contains("collect_solari_runtime_providers"));
    assert!(!registration_inputs_source.contains("collect_virtual_geometry_runtime_providers"));
}
