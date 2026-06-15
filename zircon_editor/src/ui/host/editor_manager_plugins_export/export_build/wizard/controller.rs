use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};

use super::{
    run_export_wizard_job, ExportWizardCancelSignal, ExportWizardCommandRunner,
    ExportWizardJobEvent, ExportWizardJobSnapshot, ExportWizardPipelinePlan,
};

#[derive(Clone, Debug)]
pub struct ExportWizardJobHandle {
    pub job_id: String,
    cancel_requested: Arc<AtomicBool>,
}

impl ExportWizardJobHandle {
    pub fn request_cancel(&self) {
        self.cancel_requested.store(true, Ordering::SeqCst);
    }

    pub fn is_cancel_requested(&self) -> bool {
        self.cancel_requested.load(Ordering::SeqCst)
    }
}

pub struct ExportWizardJobController {
    handle: ExportWizardJobHandle,
    events: Receiver<ExportWizardJobEvent>,
    worker: JoinHandle<ExportWizardJobSnapshot>,
}

impl ExportWizardJobController {
    pub fn spawn(
        job_id: impl Into<String>,
        plan: ExportWizardPipelinePlan,
        mut runner: impl ExportWizardCommandRunner + Send + 'static,
    ) -> Self {
        let job_id = job_id.into();
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let handle = ExportWizardJobHandle {
            job_id: job_id.clone(),
            cancel_requested: Arc::clone(&cancel_requested),
        };
        let cancel_signal = ExportWizardSharedCancelSignal {
            cancel_requested: Arc::clone(&cancel_requested),
        };
        let (event_sender, events) = channel();
        let worker = spawn(move || {
            run_export_wizard_job(job_id, &plan, &mut runner, &cancel_signal, &mut |event| {
                let _ = event_sender.send(event);
            })
        });
        Self {
            handle,
            events,
            worker,
        }
    }

    pub fn handle(&self) -> &ExportWizardJobHandle {
        &self.handle
    }

    pub fn request_cancel(&self) {
        self.handle.request_cancel();
    }

    pub fn events(&self) -> &Receiver<ExportWizardJobEvent> {
        &self.events
    }

    pub fn finish(self) -> Result<ExportWizardJobSnapshot, String> {
        self.worker
            .join()
            .map_err(|_| format!("export wizard job {} worker panicked", self.handle.job_id))
    }
}

#[derive(Clone, Debug)]
struct ExportWizardSharedCancelSignal {
    cancel_requested: Arc<AtomicBool>,
}

impl ExportWizardCancelSignal for ExportWizardSharedCancelSignal {
    fn is_cancel_requested(&self) -> bool {
        self.cancel_requested.load(Ordering::SeqCst)
    }
}
