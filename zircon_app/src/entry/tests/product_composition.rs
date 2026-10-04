pub(super) mod close_owner;

use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    PluginSelectionResolutionStatus, ProjectPluginManifest, ProjectPluginSelection,
    RuntimeProfileId, RuntimeProfileId::Minimal,
};
use zircon_runtime::plugin::{
    PluginPackageRole, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

use super::super::{EntryConfig, EntryProfile, ProductCompositionRequest, ProductRoleRequest};

fn explicit_manifest(id: &str, required: bool) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::new(id),
            true,
            required,
        )
        .with_target_modes([RuntimeTargetMode::ServerRuntime])],
    }
}

fn explicit_registration(
    id: &str,
    package_role: PluginPackageRole,
) -> RuntimePluginRegistrationReport {
    let descriptor = RuntimePluginDescriptor::builder(
        id,
        format!("{id} plugin"),
        RuntimePluginId::new(id),
        format!("zircon_plugin_{id}_runtime"),
    )
    .with_target_modes([RuntimeTargetMode::ServerRuntime])
    .with_package_role(package_role)
    .build();
    RuntimePluginRegistrationReport::from_plugin(&descriptor)
}

fn explicit_headless_config(manifest: ProjectPluginManifest) -> EntryConfig {
    EntryConfig::new(EntryProfile::Headless).with_project_plugins(manifest)
}

#[test]
fn composition_retains_config_receipt_core_and_plugin_owners_together() {
    let composition = ProductCompositionRequest::new(EntryConfig::for_runtime_profile(Minimal))
        .compose()
        .expect("minimal product composition should compile and bootstrap once");
    {
        assert_eq!(
            composition.resolved_config().runtime_profile(),
            Some(RuntimeProfileId::Minimal)
        );
        assert_eq!(
            composition.resolved_config().role(),
            ProductRoleRequest::DesktopClient
        );
        assert_eq!(
            &composition
                .module_selection_report()
                .runtime_module_composition_identity,
            composition.runtime_module_composition_identity()
        );
        assert!(composition.compiled_project_plugin_plan().is_some());
        assert!(composition
            .runtime_plugin_bridge_lifecycle_state()
            .is_some());
        assert!(composition.native_plugin_host().is_none());
        let _core = composition.core();
    }
    close_owner::close_composition(composition);
}

#[test]
fn report_only_request_uses_the_same_composition_preparation_path() {
    let report = ProductCompositionRequest::new(EntryConfig::for_runtime_profile(Minimal))
        .module_selection_report()
        .expect("report-only composition should compile without activating Core");
    let diagnostics = ProductCompositionRequest::new(EntryConfig::for_runtime_profile(Minimal))
        .module_selection_diagnostics()
        .expect("report diagnostics should use the same prepared composition");
    let composition_hash = report
        .runtime_module_composition_identity
        .composition_hash_hex();

    assert_eq!(report.runtime_profile, Some(Minimal));
    assert!(diagnostics.contains(&composition_hash));
}

#[test]
fn entry_runner_has_one_composition_execution_surface() {
    let runner = include_str!("../entry_runner/bootstrap.rs");
    let request = include_str!("../product_composition/request.rs");
    let composition = include_str!("../product_composition/composition.rs");
    let engine_entry = include_str!("../engine_entry.rs");

    assert!(runner.contains("ProductCompositionRequest::new(config).compose()"));
    assert!(!runner.contains("pub fn bootstrap_with_"));
    assert!(!composition.contains("pub fn into_core"));
    assert!(!composition.contains("pub const fn core("));
    assert!(!composition.contains("pub fn core("));
    assert!(composition.contains("pub(crate) fn core("));
    assert!(!engine_entry.contains("pub trait EngineEntry"));
    assert!(!engine_entry.contains("pub struct BuiltinEngineEntry"));
    assert!(!request.contains("eprintln!"));
    let ownership = include_str!("../product_composition/ownership.rs");
    assert!(composition.contains("ownership: Option<ProductOwnership>"));
    let core_owner = ownership
        .find("core: CoreHandle")
        .expect("composition must retain Core");
    let bridge_owner = ownership
        .find("plugin_bridge_lifecycle_state:")
        .expect("composition must retain plugin bridge lifecycle state");
    let compiled_plan_owner = ownership
        .find("compiled_project_plugin_plan:")
        .expect("composition must retain the compiled plugin plan");
    let native_owner = ownership
        .find("native_plugin_host:")
        .expect("composition must retain the native plugin host");
    assert!(core_owner < bridge_owner);
    assert!(bridge_owner < compiled_plan_owner);
    assert!(compiled_plan_owner < native_owner);
    for field in [
        "runtime_plugin_registration_reports",
        "runtime_plugin_feature_registration_reports",
    ] {
        assert!(request.contains(&format!("extend(native_report.{field});")));
        assert!(!request.contains(&format!("extend(native_report.{field}.clone());")));
    }
}

#[test]
fn product_composition_can_be_retained_by_generated_platform_hosts() {
    fn assert_send<T: Send>() {}

    assert_send::<super::super::ProductComposition>();
}

#[test]
fn explicit_product_composition_retains_typed_optional_selection_outcomes() {
    let id = "optional_provider";
    let composition = ProductCompositionRequest::new(explicit_headless_config(explicit_manifest(
        id, false,
    )))
    .with_runtime_plugin_registrations(std::iter::empty::<RuntimePluginRegistrationReport>())
    .compose()
    .expect("an optional unsupported selection must not block product composition");
    {
        let outcomes = &composition
            .module_selection_report()
            .plugin_selection_outcomes;
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].selection.id, id);
        assert_eq!(
            outcomes[0].status,
            PluginSelectionResolutionStatus::Unsupported
        );
    }
    close_owner::close_composition(composition);
}

#[test]
fn explicit_product_composition_excludes_sample_and_test_fixture_reports_from_catalog() {
    for (id, role) in [
        ("sample_provider", PluginPackageRole::Sample),
        ("fixture_provider", PluginPackageRole::TestFixture),
    ] {
        let composition =
            ProductCompositionRequest::new(explicit_headless_config(explicit_manifest(id, false)))
                .with_runtime_plugin_registrations([explicit_registration(id, role)])
                .compose()
                .expect("an optional carrier package must not block product composition");
        {
            let catalog = composition
                .runtime_plugin_bridge_lifecycle_state()
                .expect("product composition should retain its runtime plugin catalog")
                .catalog();
            assert!(catalog
                .registrations()
                .iter()
                .all(|registration| registration.package_manifest.id != id));
            assert_eq!(
                composition
                    .module_selection_report()
                    .plugin_selection_outcomes[0]
                    .status,
                PluginSelectionResolutionStatus::Unsupported
            );
        }
        close_owner::close_composition(composition);
    }
}

#[test]
fn explicit_product_composition_catalog_contains_only_resolved_runtime_reports() {
    let selected_id = "selected_provider";
    let extra_id = "extra_provider";
    let composition = ProductCompositionRequest::new(explicit_headless_config(explicit_manifest(
        selected_id,
        true,
    )))
    .with_runtime_plugin_registrations([
        explicit_registration(selected_id, PluginPackageRole::Production),
        explicit_registration(extra_id, PluginPackageRole::Production),
    ])
    .compose()
    .expect("the selected production provider should satisfy composition");
    {
        let catalog = composition
            .runtime_plugin_bridge_lifecycle_state()
            .expect("product composition should retain its runtime plugin catalog")
            .catalog();
        assert_eq!(
            catalog
                .registrations()
                .iter()
                .map(|registration| registration.package_manifest.id.as_str())
                .collect::<Vec<_>>(),
            vec![selected_id]
        );
        assert_eq!(
            composition
                .module_selection_report()
                .plugin_selection_outcomes[0]
                .status,
            PluginSelectionResolutionStatus::Resolved
        );
    }
    close_owner::close_composition(composition);
}

#[test]
fn explicit_product_composition_rejects_required_test_fixture_reports() {
    let id = "required_fixture_provider";
    let error =
        ProductCompositionRequest::new(explicit_headless_config(explicit_manifest(id, true)))
            .with_runtime_plugin_registrations([explicit_registration(
                id,
                PluginPackageRole::TestFixture,
            )])
            .compose()
            .expect_err("a required test fixture must not satisfy product selection");

    assert!(error
        .to_string()
        .contains("required plugin selection `required_fixture_provider` is Unsupported"));
}

#[test]
fn retaining_editor_selection_outcomes_is_idempotent_for_explicit_products() {
    let id = "resolved_provider";
    let mut composition =
        ProductCompositionRequest::new(explicit_headless_config(explicit_manifest(id, true)))
            .with_runtime_plugin_registrations([explicit_registration(
                id,
                PluginPackageRole::Production,
            )])
            .compose()
            .expect("a production report should resolve the required selection");
    {
        let outcomes = composition
            .module_selection_report()
            .plugin_selection_outcomes
            .clone();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(
            outcomes[0].status,
            PluginSelectionResolutionStatus::Resolved
        );

        // Editor startup still appends its precomputed runtime outcomes. The append must be
        // idempotent now that explicit product composition owns the runtime outcomes too.
        composition.retain_plugin_selection_outcomes(outcomes);
        assert_eq!(
            composition
                .module_selection_report()
                .plugin_selection_outcomes
                .len(),
            1
        );
    }
    close_owner::close_composition(composition);
}
