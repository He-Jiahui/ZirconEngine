use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::core::jobs::test_job_system;

use super::super::super::{
    export_wizard_pipeline_plan, ExportWizardCommandExecution, ExportWizardJobController,
    ExportWizardPipelineOptions, ExportWizardPipelineStageCommand,
};
use super::*;

struct CountingRunner(Arc<AtomicUsize>);

impl ExportWizardCommandRunner for CountingRunner {
    fn run(
        &mut self,
        _command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines: Vec::new(),
            stderr_lines: Vec::new(),
        })
    }
}

#[test]
fn astra_m18_plan_rejection_preserves_diagnostics_through_job_ticket() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "invalid-pair",
        "zircon-project.toml",
        "unused-export-output",
    );
    options.previous_pack = Some("previous.zrpack".into());
    let invalid_pair = export_wizard_pipeline_plan(options);
    assert!(!invalid_pair.diagnostics.is_empty());

    let unavailable = ExportWizardPipelinePlan::unavailable(
        "missing-provider",
        "unused-export-output",
        "required runtime provider is unavailable",
    );
    for plan in [invalid_pair, unavailable] {
        let expected_profile = plan.profile.clone();
        let expected_diagnostics = plan.diagnostics.clone();
        let runs = Arc::new(AtomicUsize::new(0));
        let jobs = test_job_system();
        let completion = ExportWizardJobController::submit(
            &jobs,
            "astra-plan-rejection",
            plan,
            CountingRunner(Arc::clone(&runs)),
        )
        .expect("invalid plan is reported by the domain job")
        .finish();
        let error = completion.result.expect_err("invalid plan must fail");
        let Some(EditorExportBuildError::WizardPlanFailed {
            profile,
            diagnostics,
        }) = error.downcast_ref::<EditorExportBuildError>()
        else {
            panic!("expected typed plan failure, received {error:?}");
        };
        assert_eq!(profile, &expected_profile);
        assert_eq!(diagnostics, &expected_diagnostics);
        assert_eq!(runs.load(Ordering::Relaxed), 0);
        let terminal = completion.events.last().expect("failure event is retained");
        assert_eq!(terminal.kind, ExportWizardJobEventKind::Failed);
        assert_eq!(terminal.snapshot.diagnostics, expected_diagnostics);
        assert!(terminal.snapshot.stages.is_empty());
    }
}
