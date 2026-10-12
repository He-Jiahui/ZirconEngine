use std::io;
use std::sync::mpsc;

use super::*;
use crate::core::jobs::{test_job_system, CancellationToken, EditorJobSpec, JobCategory, JobError};
use zircon_runtime::asset::AssetUri;

struct FailingDesktopExportExecutor;

impl DesktopExportExecutor for FailingDesktopExportExecutor {
    fn execute(
        &self,
        _project_root: &std::path::Path,
        _output_root: &std::path::Path,
        _manifest: &ProjectManifest,
        _profile_name: &str,
        _cancel: &CancellationToken,
        _progress: &mut dyn FnMut(EditorExportBuildProgress),
    ) -> Result<EditorExportBuildReport, EditorExportBuildError> {
        Err(EditorExportBuildError::Materialize {
            source: io::Error::new(io::ErrorKind::WriteZero, "retained worker source"),
        })
    }
}

#[test]
fn retained_export_worker_ticket_preserves_typed_editor_export_error() {
    let jobs = test_job_system();
    let (progress_sender, _progress_receiver) = mpsc::channel();
    let job = DesktopExportQueuedJob {
        id: 7,
        profile_name: "desktop_windows".to_string(),
        project_root: PathBuf::from("Project"),
        manifest: ProjectManifest::new(
            "Project",
            AssetUri::parse("res://main.scene.toml").expect("test asset URI is valid"),
            1,
        ),
        output_root: PathBuf::from("Builds/windows"),
        cancel: CancellationToken::default(),
    };
    let ticket = jobs
        .submit(
            EditorJobSpec::new("retained export source", JobCategory::Export),
            DesktopExportEditorJob::with_executor(
                job,
                Arc::new(FailingDesktopExportExecutor),
                progress_sender,
            ),
        )
        .expect("retained export job should submit");

    let error = ticket.wait().expect_err("retained export job should fail");
    let export_error = error
        .downcast_ref::<EditorExportBuildError>()
        .expect("job ticket must retain the typed editor export error");
    assert!(matches!(
        export_error,
        EditorExportBuildError::Materialize { source }
            if source.kind() == io::ErrorKind::WriteZero
    ));
    assert!(matches!(error, JobError::Failed(_)));
}
