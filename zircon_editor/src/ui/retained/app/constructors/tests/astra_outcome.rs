use super::*;
use crate::ui::host::{EditorExportBuildReport, EditorExportCargoInvocation};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportProfile, ExportTargetPlatform, RuntimeProfileId,
};
use zircon_runtime::plugin::ExportBuildPlan;

fn report() -> EditorExportBuildReport {
    EditorExportBuildReport {
        plan: ExportBuildPlan {
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
        },
        invoked_cargo: true,
        cargo_invocation: Some(EditorExportCargoInvocation {
            command: vec!["cargo".into()],
            status_code: Some(0),
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        }),
        native_cargo_invocations: vec![],
        generated_files: vec![PathBuf::from("manifest")],
        copied_packages: vec![],
        diagnostics: vec!["warning".into()],
        fatal_diagnostics: vec![],
    }
}

#[test]
fn astra_m5_summary_reports_failed_cargo_and_keeps_generated_file_counts() {
    let mut failed = report();
    failed.cargo_invocation.as_mut().unwrap().success = false;
    failed.cargo_invocation.as_mut().unwrap().status_code = Some(9);
    let summary = DesktopExportExecutionSummary::from_report(PathBuf::from("output"), failed);
    assert_eq!(summary.state, DesktopExportExecutionState::Failed);
    assert_eq!(summary.generated_files, 1);
    assert_eq!(summary.diagnostics, ["warning"]);
    assert_eq!(
        summary.fatal_diagnostics,
        ["export Cargo build did not exit successfully"]
    );
}

#[test]
fn astra_m5_summary_preserves_fatal_success_and_cancelled_states() {
    let mut failed = report();
    failed.fatal_diagnostics.push("fatal plan".into());
    let summary = DesktopExportExecutionSummary::from_report(PathBuf::new(), failed);
    assert_eq!(summary.state, DesktopExportExecutionState::Failed);
    assert_eq!(summary.fatal_diagnostics, ["fatal plan"]);
    let success = DesktopExportExecutionSummary::from_report(PathBuf::new(), report());
    assert_eq!(success.state, DesktopExportExecutionState::Exported);
    let cancelled =
        DesktopExportExecutionSummary::cancelled("astra", PathBuf::new(), "cancelled".into());
    assert_eq!(cancelled.state, DesktopExportExecutionState::Cancelled);
}
