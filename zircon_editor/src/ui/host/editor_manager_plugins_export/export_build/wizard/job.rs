use std::collections::VecDeque;

use zircon_runtime_interface::export::ExportStage;

use super::output_tail::push_bounded_output_line;
use super::{
    ExportWizardCommandOutputLine, ExportWizardCommandOutputStream, ExportWizardPipelineExecution,
    ExportWizardPipelinePlan, ExportWizardProgressState, ExportWizardStageExecution,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWizardJobStatus {
    Pending,
    Running,
    Cancelling,
    Cancelled,
    Finished,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardJobSnapshot {
    pub job_id: String,
    pub profile: String,
    pub out: String,
    pub status: ExportWizardJobStatus,
    pub current_stage: Option<ExportStage>,
    pub progress: ExportWizardProgressState,
    pub stages: Vec<ExportWizardStageExecution>,
    pub live_stage_outputs: Vec<ExportWizardStageOutputBuffer>,
    pub diagnostics: Vec<String>,
    pub fatal: bool,
    pub cancel_requested: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardStageOutputBuffer {
    pub stage: ExportStage,
    pub stdout_lines: VecDeque<String>,
    pub stderr_lines: VecDeque<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardJobState {
    snapshot: ExportWizardJobSnapshot,
}

impl ExportWizardJobState {
    pub fn new(job_id: impl Into<String>, plan: &ExportWizardPipelinePlan) -> Self {
        let fatal = !plan.diagnostics.is_empty();
        Self {
            snapshot: ExportWizardJobSnapshot {
                job_id: job_id.into(),
                profile: plan.profile.clone(),
                out: plan.out.clone(),
                status: if fatal {
                    ExportWizardJobStatus::Failed
                } else {
                    ExportWizardJobStatus::Pending
                },
                current_stage: None,
                progress: ExportWizardProgressState::for_stages(
                    plan.ordered_commands().map(|command| command.stage),
                ),
                stages: Vec::new(),
                live_stage_outputs: Vec::new(),
                diagnostics: plan.diagnostics.clone(),
                fatal,
                cancel_requested: false,
            },
        }
    }

    pub fn begin(&mut self) {
        if matches!(self.snapshot.status, ExportWizardJobStatus::Pending) {
            self.snapshot.status = ExportWizardJobStatus::Running;
        }
    }

    pub fn request_cancel(&mut self) {
        self.snapshot.cancel_requested = true;
        if matches!(
            self.snapshot.status,
            ExportWizardJobStatus::Pending | ExportWizardJobStatus::Running
        ) {
            self.snapshot.status = ExportWizardJobStatus::Cancelling;
        }
    }

    pub fn begin_stage(&mut self, stage: ExportStage, progress: ExportWizardProgressState) {
        self.begin();
        self.snapshot.current_stage = Some(stage);
        self.snapshot.progress = progress;
    }

    pub fn record_stage_output(
        &mut self,
        stage: ExportStage,
        output: ExportWizardCommandOutputLine,
        progress: ExportWizardProgressState,
    ) {
        self.snapshot.apply_stage_output(stage, output, progress);
    }

    pub fn mark_cancelled(&mut self, diagnostic: impl Into<String>) {
        self.snapshot.status = ExportWizardJobStatus::Cancelled;
        self.snapshot.cancel_requested = true;
        self.snapshot.diagnostics.push(diagnostic.into());
    }

    pub fn record_stage_execution(&mut self, stage_execution: ExportWizardStageExecution) {
        self.snapshot.current_stage = Some(stage_execution.stage);
        self.snapshot.progress = stage_execution.progress.clone();
        self.snapshot
            .diagnostics
            .extend(stage_execution.diagnostics.iter().cloned());
        self.snapshot.fatal |= stage_execution.fatal;
        if stage_execution.fatal {
            self.snapshot.status = ExportWizardJobStatus::Failed;
        }
        self.snapshot
            .live_stage_outputs
            .retain(|buffer| buffer.stage != stage_execution.stage);
        self.snapshot.stages.push(stage_execution);
    }

    pub fn finish_from_pipeline(&mut self, execution: ExportWizardPipelineExecution) {
        self.snapshot.current_stage = execution.stages.last().map(|stage| stage.stage);
        self.snapshot.progress = execution.progress;
        self.snapshot.stages = execution.stages;
        self.snapshot.live_stage_outputs.clear();
        self.snapshot.diagnostics = execution.diagnostics;
        self.snapshot.fatal = execution.fatal;
        self.snapshot.status = if self.snapshot.cancel_requested {
            ExportWizardJobStatus::Cancelled
        } else if self.snapshot.fatal {
            ExportWizardJobStatus::Failed
        } else {
            ExportWizardJobStatus::Finished
        };
    }

    pub fn snapshot(&self) -> &ExportWizardJobSnapshot {
        &self.snapshot
    }

    pub fn into_snapshot(self) -> ExportWizardJobSnapshot {
        self.snapshot
    }
}

impl ExportWizardJobSnapshot {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            ExportWizardJobStatus::Cancelled
                | ExportWizardJobStatus::Finished
                | ExportWizardJobStatus::Failed
        )
    }

    pub fn event_header(&self) -> Self {
        Self {
            job_id: self.job_id.clone(),
            profile: self.profile.clone(),
            out: self.out.clone(),
            status: self.status,
            current_stage: self.current_stage,
            progress: self.progress.clone(),
            stages: Vec::new(),
            live_stage_outputs: Vec::new(),
            diagnostics: Vec::new(),
            fatal: self.fatal,
            cancel_requested: self.cancel_requested,
        }
    }

    pub fn apply_stage_output(
        &mut self,
        stage: ExportStage,
        output: ExportWizardCommandOutputLine,
        progress: ExportWizardProgressState,
    ) {
        if matches!(self.status, ExportWizardJobStatus::Pending) {
            self.status = ExportWizardJobStatus::Running;
        }
        self.current_stage = Some(stage);
        self.progress = progress;
        let buffer = self.stage_output_buffer_mut(stage);
        match output.stream {
            ExportWizardCommandOutputStream::Stdout => {
                push_bounded_output_line(&mut buffer.stdout_lines, output.line);
            }
            ExportWizardCommandOutputStream::Stderr => {
                push_bounded_output_line(&mut buffer.stderr_lines, output.line);
            }
        }
    }

    fn stage_output_buffer_mut(
        &mut self,
        stage: ExportStage,
    ) -> &mut ExportWizardStageOutputBuffer {
        if let Some(index) = self
            .live_stage_outputs
            .iter()
            .position(|buffer| buffer.stage == stage)
        {
            return &mut self.live_stage_outputs[index];
        }
        self.live_stage_outputs.push(ExportWizardStageOutputBuffer {
            stage,
            stdout_lines: VecDeque::new(),
            stderr_lines: VecDeque::new(),
        });
        self.live_stage_outputs
            .last_mut()
            .expect("stage output buffer was just inserted")
    }
}
