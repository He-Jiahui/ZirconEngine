use super::*;
use crate::core::jobs::{test_job_system, CancellationToken, EditorJobSpec, JobCategory};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportProfile, ExportTargetPlatform, RuntimeProfileId,
};
use zircon_runtime::plugin::ExportBuildPlan;

struct FatalReportExecutor;

impl DesktopExportExecutor for FatalReportExecutor {
    fn execute(
        &self,
        _project: &std::path::Path,
        _output: &std::path::Path,
        _manifest: &ProjectManifest,
        _profile: &str,
        _cancel: &CancellationToken,
        _progress: &mut dyn FnMut(EditorExportBuildProgress),
    ) -> Result<EditorExportBuildReport, EditorExportBuildError> {
        Ok(EditorExportBuildReport {
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
            invoked_cargo: false,
            cargo_invocation: None,
            native_cargo_invocations: vec![],
            generated_files: vec![PathBuf::from("partial-manifest")],
            copied_packages: vec![],
            diagnostics: vec!["partial materialization log".into()],
            fatal_diagnostics: vec!["fatal materialization".into()],
        })
    }
}

#[test]
fn astra_m5_worker_rejects_ok_report_containing_fatal_diagnostics() {
    let jobs = test_job_system();
    let (sender, _receiver) = std::sync::mpsc::channel();
    let job = DesktopExportQueuedJob {
        id: 7,
        profile_name: "astra".into(),
        project_root: PathBuf::from("project"),
        output_root: PathBuf::from("output"),
        manifest: ProjectManifest::new(
            "test",
            AssetUri::parse("res://main.scene.toml").unwrap(),
            1,
        ),
        cancel: CancellationToken::default(),
    };
    let ticket = jobs
        .submit(
            EditorJobSpec::new("astra fatal report", JobCategory::Export),
            DesktopExportEditorJob::with_executor(job, Arc::new(FatalReportExecutor), sender),
        )
        .unwrap();
    let error = ticket.wait().unwrap_err();
    assert!(matches!(error, JobError::Failed(_)));
    let EditorExportBuildError::ReportFailed { report } =
        error.downcast_ref::<EditorExportBuildError>().unwrap()
    else {
        panic!("typed report must survive job boundary")
    };
    assert_eq!(report.generated_files.len(), 1);
    assert_eq!(report.diagnostics, ["partial materialization log"]);
    assert_eq!(report.fatal_diagnostics, ["fatal materialization"]);
}

struct CancelledReportExecutor;

impl DesktopExportExecutor for CancelledReportExecutor {
    fn execute(
        &self,
        project: &std::path::Path,
        output: &std::path::Path,
        manifest: &ProjectManifest,
        profile: &str,
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(EditorExportBuildProgress),
    ) -> Result<EditorExportBuildReport, EditorExportBuildError> {
        let report =
            FatalReportExecutor.execute(project, output, manifest, profile, cancel, progress)?;
        cancel.cancel();
        report.into_result()
    }
}

#[test]
fn astra_m5_worker_keeps_running_cancellation_when_executor_returns_failed_report() {
    let jobs = test_job_system();
    let (sender, _receiver) = std::sync::mpsc::channel();
    let job = DesktopExportQueuedJob {
        id: 8,
        profile_name: "astra".into(),
        project_root: PathBuf::from("project"),
        output_root: PathBuf::from("output"),
        manifest: ProjectManifest::new(
            "test",
            AssetUri::parse("res://main.scene.toml").unwrap(),
            1,
        ),
        cancel: CancellationToken::default(),
    };
    let ticket = jobs
        .submit(
            EditorJobSpec::new("astra cancelled failed report", JobCategory::Export),
            DesktopExportEditorJob::with_executor(job, Arc::new(CancelledReportExecutor), sender),
        )
        .unwrap();
    assert!(matches!(ticket.wait(), Err(JobError::Cancelled)));
}
