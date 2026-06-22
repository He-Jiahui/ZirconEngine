use std::sync::atomic::{AtomicBool, Ordering};

use super::{
    execute_export_wizard_stage_with_output_and_cancel, ExportWizardCommandRunner,
    ExportWizardJobSnapshot, ExportWizardJobState, ExportWizardJobStatus,
    ExportWizardPipelineExecution, ExportWizardPipelinePlan, ExportWizardProgressState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWizardJobEventKind {
    Created,
    Started,
    StageStarted,
    StageOutput,
    StageFinished,
    Cancelled,
    Failed,
    Finished,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardJobEvent {
    pub kind: ExportWizardJobEventKind,
    pub snapshot: ExportWizardJobSnapshot,
}

pub trait ExportWizardCancelSignal {
    fn is_cancel_requested(&self) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ExportWizardNeverCancel;

impl ExportWizardCancelSignal for ExportWizardNeverCancel {
    fn is_cancel_requested(&self) -> bool {
        false
    }
}

impl ExportWizardCancelSignal for AtomicBool {
    fn is_cancel_requested(&self) -> bool {
        self.load(Ordering::SeqCst)
    }
}

pub fn run_export_wizard_job(
    job_id: impl Into<String>,
    plan: &ExportWizardPipelinePlan,
    runner: &mut impl ExportWizardCommandRunner,
    cancel_signal: &impl ExportWizardCancelSignal,
    emit_event: &mut impl FnMut(ExportWizardJobEvent),
) -> ExportWizardJobSnapshot {
    let mut job = ExportWizardJobState::new(job_id, plan);
    emit_job_event(emit_event, ExportWizardJobEventKind::Created, &job);

    if job.snapshot().is_terminal() {
        emit_terminal_event(emit_event, &job);
        return job.into_snapshot();
    }

    if cancel_signal.is_cancel_requested() {
        job.request_cancel();
        job.mark_cancelled("export wizard cancelled before pipeline start");
        emit_job_event(emit_event, ExportWizardJobEventKind::Cancelled, &job);
        return job.into_snapshot();
    }

    job.begin();
    emit_job_event(emit_event, ExportWizardJobEventKind::Started, &job);

    let mut progress =
        ExportWizardProgressState::for_stages(plan.stages.iter().map(|command| command.stage));
    for command in &plan.stages {
        if cancel_signal.is_cancel_requested() {
            job.request_cancel();
            job.mark_cancelled(format!(
                "export wizard cancelled before {:?} started",
                command.stage
            ));
            emit_job_event(emit_event, ExportWizardJobEventKind::Cancelled, &job);
            return job.into_snapshot();
        }

        job.begin_stage(command.stage, progress.clone());
        emit_job_event(emit_event, ExportWizardJobEventKind::StageStarted, &job);

        let stage_execution = execute_export_wizard_stage_with_output_and_cancel(
            command,
            runner,
            &mut progress,
            &mut |output, progress| {
                job.record_stage_output(command.stage, output, progress.clone());
                emit_job_event(emit_event, ExportWizardJobEventKind::StageOutput, &job);
            },
            &mut || cancel_signal.is_cancel_requested(),
        );
        let stage_was_cancelled = stage_execution.cancelled;
        let stage_was_fatal = stage_execution.fatal;
        job.record_stage_execution(stage_execution);
        emit_job_event(emit_event, ExportWizardJobEventKind::StageFinished, &job);

        if stage_was_cancelled {
            job.request_cancel();
            job.mark_cancelled(format!(
                "export wizard cancelled while {:?} was running",
                command.stage
            ));
            emit_job_event(emit_event, ExportWizardJobEventKind::Cancelled, &job);
            return job.into_snapshot();
        }

        if stage_was_fatal {
            emit_job_event(emit_event, ExportWizardJobEventKind::Failed, &job);
            return job.into_snapshot();
        }

        if cancel_signal.is_cancel_requested() {
            job.request_cancel();
            job.mark_cancelled(format!(
                "export wizard cancelled after {:?} finished",
                command.stage
            ));
            emit_job_event(emit_event, ExportWizardJobEventKind::Cancelled, &job);
            return job.into_snapshot();
        }
    }

    let execution = ExportWizardPipelineExecution {
        stages: job.snapshot().stages.clone(),
        progress,
        diagnostics: job.snapshot().diagnostics.clone(),
        fatal: job.snapshot().fatal,
    };
    job.finish_from_pipeline(execution);
    emit_terminal_event(emit_event, &job);
    job.into_snapshot()
}

fn emit_terminal_event(
    emit_event: &mut impl FnMut(ExportWizardJobEvent),
    job: &ExportWizardJobState,
) {
    let kind = match job.snapshot().status {
        ExportWizardJobStatus::Cancelled => ExportWizardJobEventKind::Cancelled,
        ExportWizardJobStatus::Failed => ExportWizardJobEventKind::Failed,
        ExportWizardJobStatus::Finished => ExportWizardJobEventKind::Finished,
        ExportWizardJobStatus::Pending
        | ExportWizardJobStatus::Running
        | ExportWizardJobStatus::Cancelling => return,
    };
    emit_job_event(emit_event, kind, job);
}

fn emit_job_event(
    emit_event: &mut impl FnMut(ExportWizardJobEvent),
    kind: ExportWizardJobEventKind,
    job: &ExportWizardJobState,
) {
    emit_event(ExportWizardJobEvent {
        kind,
        snapshot: job.snapshot().clone(),
    });
}
