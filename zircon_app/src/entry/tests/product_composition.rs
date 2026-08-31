use zircon_runtime::core::framework::project::{RuntimeProfileId, RuntimeProfileId::Minimal};

use super::super::{EntryConfig, ProductCompositionRequest, ProductRoleRequest};

#[test]
fn composition_retains_config_receipt_core_and_plugin_owners_together() {
    let composition = ProductCompositionRequest::new(EntryConfig::for_runtime_profile(Minimal))
        .compose()
        .expect("minimal product composition should compile and bootstrap once");

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
    assert!(composition.contains("pub(crate) const fn core("));
    assert!(!engine_entry.contains("pub trait EngineEntry"));
    assert!(!engine_entry.contains("pub struct BuiltinEngineEntry"));
    assert!(!request.contains("eprintln!"));
    let core_owner = composition
        .find("core: CoreHandle")
        .expect("composition must retain Core");
    let bridge_owner = composition
        .find("plugin_bridge_lifecycle_state:")
        .expect("composition must retain plugin bridge lifecycle state");
    let compiled_plan_owner = composition
        .find("compiled_project_plugin_plan:")
        .expect("composition must retain the compiled plugin plan");
    let native_owner = composition
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
