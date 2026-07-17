use std::error::Error as _;

use zircon_runtime::plugin::ExportBuildPlanError;

use crate::core::jobs::JobError;
use crate::ui::host::EditorExportBuildError;

#[test]
fn failed_job_preserves_typed_export_error_for_downcast() {
    let job_error = JobError::failed(EditorExportBuildError::Plan(
        ExportBuildPlanError::MissingProfile {
            profile_name: "desktop".to_string(),
        },
    ));

    let export_error = job_error
        .downcast_ref::<EditorExportBuildError>()
        .expect("job failure must preserve the editor export error type");
    assert!(matches!(
        export_error,
        EditorExportBuildError::Plan(ExportBuildPlanError::MissingProfile { profile_name })
            if profile_name == "desktop"
    ));
    assert!(job_error.source().is_some());
}
