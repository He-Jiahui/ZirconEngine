use super::{Duration, EditorJob, Instant, JobContext, JobError, Receiver, Sender};

pub(super) struct ValueJob(pub(super) u32);

impl EditorJob for ValueJob {
    type Output = u32;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Ok(self.0 + 1)
    }
}

pub(super) struct MergeCancellationJob {
    started: Sender<()>,
    cancelled: Sender<()>,
}

impl MergeCancellationJob {
    pub(super) fn new(started: Sender<()>, cancelled: Sender<()>) -> Self {
        Self { started, cancelled }
    }
}

impl EditorJob for MergeCancellationJob {
    type Output = ();

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        let _ = self.started.send(());
        let deadline = Instant::now() + Duration::from_secs(1);
        while !context.is_cancelled() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        if !context.is_cancelled() {
            return Err(JobError::failed(std::io::Error::other(
                "latest merged cancellation token was not reached",
            )));
        }
        let _ = self.cancelled.send(());
        context.check_cancelled()
    }
}

pub(super) struct GateJob {
    started: Sender<()>,
    release: Receiver<()>,
}

impl GateJob {
    pub(super) fn new(started: Sender<()>, release: Receiver<()>) -> Self {
        Self { started, release }
    }
}

impl EditorJob for GateJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        let _ = self.started.send(());
        self.release.recv().map_err(JobError::failed)
    }
}
