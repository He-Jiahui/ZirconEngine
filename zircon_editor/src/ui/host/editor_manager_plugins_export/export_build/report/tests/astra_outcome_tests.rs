use super::*;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportProfile, ExportTargetPlatform, RuntimeProfileId,
};
use zircon_runtime::plugin::{PluginMaturity, RuntimePluginAvailabilityEntry};

fn report() -> EditorExportBuildReport {
    let plan = ExportBuildPlan {
        profile: ExportProfile::new(
            "astra",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Client2d,
        ),
        platform_policy: ExportTargetPlatform::Windows.policy(),
        enabled_runtime_plugins: vec![],
        linked_runtime_crates: vec![],
        linked_feature_source_count: 0,
        linked_feature_sources: vec![],
        admitted_plan_proof: None,
        native_dynamic_packages: vec![],
        native_dynamic_package_exports: vec![],
        runtime_plugin_availability: Default::default(),
        library_embed_compile_host: None,
        source_template_build: None,
        generated_files: vec![],
        diagnostics: vec![],
        fatal_diagnostics: vec![],
    };
    EditorExportBuildReport {
        plan,
        invoked_cargo: false,
        cargo_invocation: None,
        native_cargo_invocations: vec![],
        generated_files: vec![PathBuf::from("output/report.json")],
        copied_packages: vec![],
        diagnostics: vec!["warning is not a failure".into()],
        fatal_diagnostics: vec![],
    }
}

fn invocation(success: bool, status_code: Option<i32>) -> EditorExportCargoInvocation {
    EditorExportCargoInvocation {
        command: vec!["cargo".into(), "build".into()],
        success,
        status_code,
        stdout: "output".into(),
        stderr: "diagnostic".into(),
    }
}

#[test]
fn astra_m5_fatal_reports_move_full_diagnostics_into_typed_error() {
    let mut failed = report();
    failed
        .fatal_diagnostics
        .push("materialization failed".into());
    let diagnostics = failed.diagnostics.as_ptr();
    let generated = failed.generated_files.as_ptr();
    let error = failed.into_result().unwrap_err();
    let super::super::error::EditorExportBuildError::ReportFailed { report } = error else {
        panic!("typed report failure required")
    };
    assert_eq!(report.diagnostics.as_ptr(), diagnostics);
    assert_eq!(report.generated_files.as_ptr(), generated);
    assert_eq!(report.failure_reason(), Some("materialization failed"));

    let mut failed = self::report();
    failed.plan.fatal_diagnostics.push("plan invalid".into());
    assert_eq!(failed.failure_reason(), Some("plan invalid"));
    assert!(failed.into_result().is_err());
}

#[test]
fn astra_m5_required_plugin_missing_is_a_failure_without_materialized_fatal_strings() {
    let mut failed = report();
    failed
        .plan
        .runtime_plugin_availability
        .missing_required
        .push(RuntimePluginAvailabilityEntry {
            id: "sound".into(),
            runtime_id: zircon_runtime::builtin::RuntimePluginId::Sound,
            required: true,
            maturity: PluginMaturity::Stable,
            reason: "provider missing".into(),
        });
    assert_eq!(failed.failure_reason(), Some("provider missing"));
    assert!(failed.into_result().is_err());
}

#[test]
fn astra_m5_host_and_native_builds_require_success_and_zero_exit() {
    for success in [false, true] {
        for code in [None, Some(0), Some(2)] {
            let expected = success && code == Some(0);
            let mut host = report();
            host.invoked_cargo = true;
            host.cargo_invocation = Some(invocation(success, code));
            assert_eq!(host.into_result().is_ok(), expected);
            let mut native = report();
            native
                .native_cargo_invocations
                .push(invocation(success, code));
            assert_eq!(native.into_result().is_ok(), expected);
        }
    }
    let mut missing = report();
    missing.invoked_cargo = true;
    assert!(missing.into_result().is_err());
    let mut inconsistent = report();
    inconsistent.cargo_invocation = Some(invocation(true, Some(0)));
    assert!(inconsistent.into_result().is_err());
}

#[test]
fn astra_m5_successful_reports_keep_their_buffers() {
    let success = report();
    let pointer = success.diagnostics.as_ptr();
    let success = success.into_result().unwrap();
    assert_eq!(success.diagnostics.as_ptr(), pointer);
    assert_eq!(success.failure_reason(), None);
}
