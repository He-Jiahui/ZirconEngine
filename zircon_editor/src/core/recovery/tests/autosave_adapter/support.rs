use super::*;

pub(super) fn wait_for_autosave_completion(
    adapter: &mut AutosaveJobAdapter,
    now: Duration,
) -> AutosaveCompletion {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let completion = adapter.pump_completed(now);
        if completion.pending() == 0 && completion.succeeded() + completion.failed() != 0 {
            return completion;
        }
        thread::sleep(Duration::from_millis(1));
    }
    panic!("autosave job did not reach a terminal result");
}

pub(super) fn wait_for_autosave_completion_state(
    adapter: &mut AutosaveJobAdapter,
    now: Duration,
    budget: usize,
    target: impl Fn(&AutosaveCompletion) -> bool,
) -> AutosaveCompletion {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let completion = adapter.pump_completed_with_budget(now, budget);
        assert!(completion.inspected_tickets() <= budget);
        if target(&completion) {
            return completion;
        }
        assert!(
            Instant::now() < deadline,
            "autosave completion state did not reach the expected result"
        );
        thread::yield_now();
    }
}

pub(super) fn wait_for_capture_count(source: &CountingSnapshotSource, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while source.capture_count() < expected {
        assert!(
            Instant::now() < deadline,
            "autosave source did not reach {expected} captures"
        );
        thread::yield_now();
    }
}

pub(super) struct GateJob {
    started: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
}

impl GateJob {
    pub(super) fn new(started: mpsc::Sender<()>, release: mpsc::Receiver<()>) -> Self {
        Self { started, release }
    }
}

impl EditorJob for GateJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).unwrap();
        self.release.recv().unwrap();
        Ok(())
    }
}

pub(super) struct CountingSnapshotSource {
    captures: AtomicUsize,
    failure: bool,
}

impl CountingSnapshotSource {
    pub(super) fn success() -> Self {
        Self {
            captures: AtomicUsize::new(0),
            failure: false,
        }
    }

    pub(super) fn failure() -> Self {
        Self {
            captures: AtomicUsize::new(0),
            failure: true,
        }
    }

    pub(super) fn capture_count(&self) -> usize {
        self.captures.load(Ordering::Acquire)
    }
}

impl AutosaveSnapshotSource for CountingSnapshotSource {
    fn source_path(&self) -> AutosaveSourcePath {
        recovery_source_path("scenes/main.zscene")
    }

    fn capture(&self, _document: &AutosaveDocumentId) -> Result<AutosaveSnapshot, JobError> {
        self.captures.fetch_add(1, Ordering::AcqRel);
        if self.failure {
            return Err(JobError::failed(std::io::Error::other("snapshot failure")));
        }
        Ok(AutosaveSnapshot::new(
            1,
            extension("zscene"),
            recovery_source_path("scenes/main.zscene"),
            AutosaveSnapshotProvenance::capture(0, AutosaveSourceDigest::missing()),
            b"autosave snapshot".to_vec(),
        ))
    }
}
