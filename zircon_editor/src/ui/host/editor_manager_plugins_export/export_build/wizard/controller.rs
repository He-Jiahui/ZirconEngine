mod completion;
mod job;
mod poll;

use std::sync::mpsc::{channel, Receiver};

use crate::core::jobs::{
    CancellationToken, EditorJobSpec, EditorJobSystem, JobCategory, JobSubmitError, JobTicket,
};

pub use self::completion::ExportWizardJobCompletion;
use self::job::ExportWizardEditorJob;
pub(super) use self::poll::ExportWizardJobPoll;
use super::{
    ExportWizardCommandRunner, ExportWizardJobEvent, ExportWizardJobSnapshot,
    ExportWizardPipelinePlan,
};

/// Owns the export domain event stream and the typed ticket submitted to the editor job service.
pub struct ExportWizardJobController {
    jobs: EditorJobSystem,
    job_id: String,
    cancel: CancellationToken,
    events: Receiver<ExportWizardJobEvent>,
    ticket: JobTicket<ExportWizardJobSnapshot>,
}

impl ExportWizardJobController {
    pub fn submit(
        jobs: &EditorJobSystem,
        job_id: impl Into<String>,
        plan: ExportWizardPipelinePlan,
        runner: impl ExportWizardCommandRunner + Send + 'static,
    ) -> Result<Self, JobSubmitError> {
        let job_id = job_id.into();
        let cancel = CancellationToken::default();
        let (event_sender, events) = channel();
        let spec =
            EditorJobSpec::new(job_id.clone(), JobCategory::Export).with_cancel(cancel.clone());
        let ticket = jobs.submit(
            spec,
            ExportWizardEditorJob::new(job_id.clone(), plan, runner, event_sender),
        )?;
        Ok(Self {
            jobs: jobs.clone(),
            job_id,
            cancel,
            events,
            ticket,
        })
    }

    pub fn job_id(&self) -> &str {
        &self.job_id
    }

    pub fn request_cancel(&self) {
        self.cancel.cancel();
        let _ = self.jobs.cancel(self.ticket.id());
    }

    pub fn is_cancel_requested(&self) -> bool {
        self.cancel.is_cancelled()
    }

    pub fn events(&self) -> &Receiver<ExportWizardJobEvent> {
        &self.events
    }

    pub(super) fn poll(&self) -> ExportWizardJobPoll {
        match self.ticket.try_take() {
            None => ExportWizardJobPoll::Pending,
            Some(result) => {
                let events = self.events.try_iter().collect();
                match result {
                    Ok(snapshot) => ExportWizardJobPoll::Completed { events, snapshot },
                    Err(error) => ExportWizardJobPoll::Failed { events, error },
                }
            }
        }
    }

    pub fn finish(self) -> ExportWizardJobCompletion {
        let result = self.ticket.wait();
        let events = self.events.try_iter().collect();
        ExportWizardJobCompletion { events, result }
    }
}
