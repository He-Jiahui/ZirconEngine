use crossbeam_channel::Sender;

use crate::core::jobs::{EditorJob, JobContext, JobError};

use super::super::{
    run_export_wizard_job, EditorExportBuildError, ExportWizardCancelSignal,
    ExportWizardCommandRunner, ExportWizardJobEvent, ExportWizardJobSnapshot,
    ExportWizardJobStatus, ExportWizardPipelinePlan,
};

use super::event_transport::send_job_event;

#[cfg(test)]
use super::super::ExportWizardJobEventKind;

#[cfg(test)]
#[path = "job/tests/astra_plan_failure_tests.rs"]
mod astra_plan_failure_tests;

/// Adapts the export wizard runner to the shared editor job contract.
pub(super) struct ExportWizardEditorJob<R> {
    job_id: String,
    plan: ExportWizardPipelinePlan,
    runner: R,
    events: Sender<ExportWizardJobEvent>,
}

impl<R> ExportWizardEditorJob<R> {
    pub(super) fn new(
        job_id: String,
        plan: ExportWizardPipelinePlan,
        runner: R,
        events: Sender<ExportWizardJobEvent>,
    ) -> Self {
        Self {
            job_id,
            plan,
            runner,
            events,
        }
    }
}

impl<R> EditorJob for ExportWizardEditorJob<R>
where
    R: ExportWizardCommandRunner + Send + 'static,
{
    type Output = ExportWizardJobSnapshot;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        let Self {
            job_id,
            plan,
            mut runner,
            events,
        } = self;
        let cancel = ExportWizardJobCancelSignal { context: &context };
        let mut coalesced_output_events = 0_u64;
        let snapshot = run_export_wizard_job(job_id, &plan, &mut runner, &cancel, &mut |event| {
            report_event_progress(&context, &event);
            send_job_event(&events, event, &mut coalesced_output_events);
        });
        match snapshot.status {
            ExportWizardJobStatus::Finished => Ok(snapshot),
            ExportWizardJobStatus::Cancelled => Err(JobError::Cancelled),
            ExportWizardJobStatus::Failed => Err(snapshot
                .stages
                .iter()
                .rev()
                .find_map(|stage| stage.failure.clone())
                .map(JobError::Failed)
                .unwrap_or_else(|| {
                    if let Some(stage) = snapshot.stages.last() {
                        JobError::failed(EditorExportBuildError::WizardStageFailed {
                            stage: stage.stage,
                            exit_code: stage.exit_code,
                        })
                    } else {
                        JobError::failed(EditorExportBuildError::WizardPlanFailed {
                            profile: snapshot.profile,
                            diagnostics: snapshot.diagnostics,
                        })
                    }
                })),
            status => Err(JobError::failed(
                EditorExportBuildError::WizardNonTerminal {
                    job_id: snapshot.job_id.clone(),
                    status: non_terminal_status_name(status),
                },
            )),
        }
    }
}

fn non_terminal_status_name(status: ExportWizardJobStatus) -> &'static str {
    match status {
        ExportWizardJobStatus::Pending => "pending",
        ExportWizardJobStatus::Running => "running",
        ExportWizardJobStatus::Cancelling => "cancelling",
        ExportWizardJobStatus::Finished => "finished",
        ExportWizardJobStatus::Cancelled => "cancelled",
        ExportWizardJobStatus::Failed => "failed",
    }
}

struct ExportWizardJobCancelSignal<'a> {
    context: &'a JobContext,
}

impl ExportWizardCancelSignal for ExportWizardJobCancelSignal<'_> {
    fn is_cancel_requested(&self) -> bool {
        self.context.is_cancelled()
    }
}

fn report_event_progress(context: &JobContext, event: &ExportWizardJobEvent) {
    let completed = bounded_stage_count(event.snapshot.stages.len());
    let total = bounded_stage_count(event.snapshot.progress.snapshots().len());
    let message = match event.snapshot.current_stage {
        Some(stage) => format!("export wizard {:?}: {:?}", event.kind, stage),
        None => format!("export wizard {:?}", event.kind),
    };
    context.report_progress(completed, total, message);
}

fn bounded_stage_count(value: usize) -> u32 {
    match u32::try_from(value) {
        Ok(value) => value,
        Err(_) => u32::MAX,
    }
}
