use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::asset::dirty::{
    DirtyExternalEffectId, DirtyRegistry, SaveDirtyViewCandidate, SaveDirtyViewCompletion,
    SaveDirtyViewExecutor, SaveDirtyViewFailure, SaveDirtyViewFailureKind,
    SaveDirtyViewOutcomeStatus, SaveDirtyViewsAdmissionError, SaveDirtyViewsJobAdapter,
    SaveDirtyViewsRequest,
};
use crate::core::editing::engine::{
    EditCommandError, EditContext, EditorTransactionEngine, HistoryContextId, SelectionSnapshot,
};
use crate::core::editor_message::{
    DocumentId, EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};
use crate::core::extension::{
    DocumentAutosavePayload, DocumentToolkit, DocumentToolkitDescriptor, DocumentToolkitRegistry,
    SaveCtx, ToolkitInstanceId, ToolkitLayout, ToolkitSaveFailure,
};
use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::jobs::{
    test_job_system_with_bus, test_job_system_with_limits, EditorJob, EditorJobAdmissionLimits,
    EditorJobLimits, EditorJobSpec, JobCategory, JobContext, JobError, JobEventKind,
    JobEventPumpBudget, JobSubmitError, MutexGroup,
};

#[test]
fn interactive_save_batch_rejects_before_mutex_or_executor_materialization() {
    let fixture = SaveBatchFixture::new(&[(1, 8), (2, 8)]);
    let jobs = test_job_system_with_limits(
        EditorJobLimits::resolved(4, [])
            .with_limit(JobCategory::Misc, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(
                2,
                1_024,
                Duration::from_secs(60),
            )),
    );
    let (started, started_receiver) = mpsc::channel();
    let (release, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("admission blocker", JobCategory::Misc),
            GateJob::new(started, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let pending = jobs
        .submit(
            EditorJobSpec::new("pending admission occupant", JobCategory::Misc),
            ImmediateJob,
        )
        .unwrap();
    let mutex_resolutions = Arc::new(AtomicUsize::new(0));
    let executor_materializations = Arc::new(AtomicUsize::new(0));
    let executor_calls = Arc::new(AtomicUsize::new(0));
    let mut adapter = SaveDirtyViewsJobAdapter::new(jobs.clone());

    let error = adapter
        .schedule(
            &fixture.request,
            {
                let mutex_resolutions = Arc::clone(&mutex_resolutions);
                move |_| {
                    mutex_resolutions.fetch_add(1, Ordering::SeqCst);
                    Ok(save_mutex("unexpected"))
                }
            },
            {
                let executor_materializations = Arc::clone(&executor_materializations);
                let executor_calls = Arc::clone(&executor_calls);
                move || {
                    executor_materializations.fetch_add(1, Ordering::SeqCst);
                    let executor_calls = Arc::clone(&executor_calls);
                    Arc::new(move |_: &super::SaveDirtyViewIntent, _: &JobContext| {
                        executor_calls.fetch_add(1, Ordering::SeqCst);
                        SaveDirtyViewCompletion::Saved { written_bytes: 1 }
                    }) as Arc<dyn SaveDirtyViewExecutor>
                }
            },
        )
        .unwrap_err();

    assert!(matches!(
        error,
        SaveDirtyViewsAdmissionError::JobSubmit(JobSubmitError::AdmissionEntryLimitExceeded {
            limit: 2
        })
    ));
    assert_eq!(mutex_resolutions.load(Ordering::SeqCst), 0);
    assert_eq!(executor_materializations.load(Ordering::SeqCst), 0);
    assert_eq!(executor_calls.load(Ordering::SeqCst), 0);
    assert!(!adapter.is_in_flight());

    assert!(jobs.cancel(pending.id()));
    assert_eq!(pending.wait(), Err(JobError::Cancelled));
    release.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn interactive_save_batch_reports_actual_bytes_before_mutex_or_executor_materialization() {
    let fixture = SaveBatchFixture::new(&[(1, 8), (2, 8)]);
    let jobs = test_job_system_with_limits(
        EditorJobLimits::resolved(4, [])
            .with_limit(JobCategory::Misc, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(
                4,
                16,
                Duration::from_secs(60),
            )),
    );
    let (started, started_receiver) = mpsc::channel();
    let (release, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("byte admission blocker", JobCategory::Misc),
            GateJob::new(started, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let pending = jobs
        .submit(
            EditorJobSpec::new("pending byte occupant", JobCategory::Misc).with_estimated_bytes(12),
            ImmediateJob,
        )
        .unwrap();
    let mutex_resolutions = Arc::new(AtomicUsize::new(0));
    let executor_materializations = Arc::new(AtomicUsize::new(0));
    let mut adapter = SaveDirtyViewsJobAdapter::new(jobs.clone());

    let error = adapter
        .schedule(
            &fixture.request,
            {
                let mutex_resolutions = Arc::clone(&mutex_resolutions);
                move |_| {
                    mutex_resolutions.fetch_add(1, Ordering::SeqCst);
                    Ok(save_mutex("unexpected"))
                }
            },
            {
                let executor_materializations = Arc::clone(&executor_materializations);
                move || {
                    executor_materializations.fetch_add(1, Ordering::SeqCst);
                    Arc::new(|_: &super::SaveDirtyViewIntent, _: &JobContext| {
                        SaveDirtyViewCompletion::Saved { written_bytes: 1 }
                    }) as Arc<dyn SaveDirtyViewExecutor>
                }
            },
        )
        .unwrap_err();

    assert!(matches!(
        error,
        SaveDirtyViewsAdmissionError::JobSubmit(JobSubmitError::AdmissionByteLimitExceeded {
            limit: 16,
            current: 12,
            requested: 16,
        })
    ));
    assert_eq!(mutex_resolutions.load(Ordering::SeqCst), 0);
    assert_eq!(executor_materializations.load(Ordering::SeqCst), 0);

    assert!(jobs.cancel(pending.id()));
    assert_eq!(pending.wait(), Err(JobError::Cancelled));
    release.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
}

#[test]
fn interactive_save_reservation_blocks_competing_admission_before_materializing_executor() {
    let fixture = SaveBatchFixture::new(&[(1, 8)]);
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(4, []).with_admission_limits(
        EditorJobAdmissionLimits::new(1, 64, Duration::from_secs(60)),
    ));
    let executor_materializations = Arc::new(AtomicUsize::new(0));
    let mut adapter = SaveDirtyViewsJobAdapter::new(jobs.clone());

    assert!(adapter
        .schedule(
            &fixture.request,
            {
                let jobs = jobs.clone();
                move |intent| {
                    assert_eq!(
                            jobs.submit(
                                EditorJobSpec::new(
                                    "save-admission-race",
                                    JobCategory::InteractiveSave,
                                ),
                                ImmediateJob,
                            )
                            .unwrap_err(),
                            JobSubmitError::AdmissionEntryLimitExceeded { limit: 1 }
                        );
                    Ok(save_mutex(intent.resource_key()))
                }
            },
            {
                let executor_materializations = Arc::clone(&executor_materializations);
                move || {
                    executor_materializations.fetch_add(1, Ordering::SeqCst);
                    Arc::new(|_: &super::SaveDirtyViewIntent, _: &JobContext| {
                        SaveDirtyViewCompletion::Saved { written_bytes: 8 }
                    }) as Arc<dyn SaveDirtyViewExecutor>
                }
            },
        )
        .unwrap());

    assert_eq!(executor_materializations.load(Ordering::SeqCst), 1);
    let batch = await_batch(&mut adapter);
    assert_eq!(
        batch.completion(document(1)),
        Some(&SaveDirtyViewCompletion::Saved { written_bytes: 8 })
    );
}

#[test]
fn interactive_save_mutex_resolution_failure_releases_the_admission_reservation() {
    let fixture = SaveBatchFixture::new(&[(1, 8)]);
    let jobs = test_job_system_with_limits(
        EditorJobLimits::resolved(4, []).with_admission_limits(EditorJobAdmissionLimits::new(
            1,
            8,
            Duration::from_secs(60),
        )),
    );
    let executor_materializations = Arc::new(AtomicUsize::new(0));
    let mut adapter = SaveDirtyViewsJobAdapter::new(jobs.clone());

    let error = adapter
        .schedule(
            &fixture.request,
            |_| Err("fixture mutex lookup failed".to_string()),
            {
                let executor_materializations = Arc::clone(&executor_materializations);
                move || {
                    executor_materializations.fetch_add(1, Ordering::SeqCst);
                    Arc::new(|_: &super::SaveDirtyViewIntent, _: &JobContext| {
                        SaveDirtyViewCompletion::Saved { written_bytes: 8 }
                    }) as Arc<dyn SaveDirtyViewExecutor>
                }
            },
        )
        .unwrap_err();
    assert!(matches!(
        error,
        SaveDirtyViewsAdmissionError::SaveMutex {
            document: failed_document,
            message
        } if failed_document == document(1) && message == "fixture mutex lookup failed"
    ));
    assert_eq!(executor_materializations.load(Ordering::SeqCst), 0);
    assert!(!adapter.is_in_flight());

    let ticket = jobs
        .submit(
            EditorJobSpec::new(
                "reservation-released-after-mutex-error",
                JobCategory::InteractiveSave,
            )
            .with_estimated_bytes(8),
            ImmediateJob,
        )
        .unwrap();
    assert_eq!(ticket.wait(), Ok(()));
}

#[test]
fn interactive_save_batch_reuses_the_caller_supplied_foreground_save_mutex() {
    let fixture = SaveBatchFixture::new(&[(1, 8)]);
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(4, []));
    let mutex = save_mutex("project://assets/document-1.zasset");
    let (started, started_receiver) = mpsc::channel();
    let (release, release_receiver) = mpsc::channel();
    let foreground = jobs
        .submit(
            EditorJobSpec::new("foreground save", JobCategory::Misc)
                .with_mutex_group(mutex.clone()),
            GateJob::new(started, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let executor_calls = Arc::new(AtomicUsize::new(0));
    let mut adapter = SaveDirtyViewsJobAdapter::new(jobs);

    assert!(adapter
        .schedule(
            &fixture.request,
            move |_| Ok(mutex.clone()),
            executor({
                let executor_calls = Arc::clone(&executor_calls);
                move |_: &super::SaveDirtyViewIntent, _: &JobContext| {
                    executor_calls.fetch_add(1, Ordering::SeqCst);
                    SaveDirtyViewCompletion::Saved { written_bytes: 8 }
                }
            }),
        )
        .unwrap());
    assert_eq!(executor_calls.load(Ordering::SeqCst), 0);

    release.send(()).unwrap();
    assert_eq!(foreground.wait(), Ok(()));
    let batch = await_batch(&mut adapter);
    assert_eq!(executor_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        batch.completion(document(1)),
        Some(&SaveDirtyViewCompletion::Saved { written_bytes: 8 })
    );
}

#[test]
fn interactive_save_batch_preserves_partial_failure_for_generation_safe_apply_and_retry() {
    let fixture = SaveBatchFixture::new(&[(1, 8), (2, 8)]);
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::resolved(4, []));
    let mut adapter = SaveDirtyViewsJobAdapter::new(jobs.clone());

    assert!(adapter
        .schedule(
            &fixture.request,
            |intent| Ok(save_mutex(intent.resource_key())),
            executor(|intent: &super::SaveDirtyViewIntent, _: &JobContext| {
                if intent.document_id() == document(1) {
                    SaveDirtyViewCompletion::Saved { written_bytes: 8 }
                } else {
                    SaveDirtyViewCompletion::Failed(SaveDirtyViewFailure::new(
                        SaveDirtyViewFailureKind::Write,
                        "fixture write failed",
                    ))
                }
            }),
        )
        .unwrap());
    let batch = await_batch(&mut adapter);
    let result = fixture
        .request
        .apply_completions(
            batch.into_completions(),
            &fixture.dirty,
            fixture.transactions.as_ref(),
        )
        .unwrap();

    assert!(matches!(
        result.outcomes()[0].status(),
        SaveDirtyViewOutcomeStatus::Saved { written_bytes: 8 }
    ));
    assert!(matches!(
        result.outcomes()[1].status(),
        SaveDirtyViewOutcomeStatus::Failed(failure)
            if failure.kind() == SaveDirtyViewFailureKind::Write
    ));
    assert_eq!(
        result.retry_documents().collect::<Vec<_>>(),
        vec![document(2)]
    );

    assert_eq!(
        jobs.pump_events_with_budget(JobEventPumpBudget::new(usize::MAX, Duration::from_secs(1),)),
        4
    );
    let terminal_events = bus
        .drain_deliveries(subscriber)
        .into_iter()
        .filter_map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::Job(event)
                if matches!(
                    event.kind(),
                    JobEventKind::Completed | JobEventKind::Failed { .. }
                ) =>
            {
                Some((event.label().to_owned(), event.kind().clone()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(terminal_events.iter().any(|(label, kind)| {
        label == "save_dirty_document_1" && matches!(kind, JobEventKind::Completed)
    }));
    assert!(terminal_events.iter().any(|(label, kind)| {
        label == "save_dirty_document_2"
            && matches!(
                kind,
                JobEventKind::Failed { message } if message.contains("fixture write failed")
            )
    }));
}

#[test]
fn interactive_save_shutdown_cancels_owned_pending_tickets_and_rejects_new_batches() {
    let fixture = SaveBatchFixture::new(&[(1, 8)]);
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::resolved(4, []));
    let mutex = save_mutex("shutdown-save");
    let (started, started_receiver) = mpsc::channel();
    let (release, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("shutdown blocker", JobCategory::Misc)
                .with_mutex_group(mutex.clone()),
            GateJob::new(started, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let executor_calls = Arc::new(AtomicUsize::new(0));
    let mut adapter = SaveDirtyViewsJobAdapter::new(jobs.clone());
    adapter
        .schedule(
            &fixture.request,
            move |_| Ok(mutex.clone()),
            executor({
                let executor_calls = Arc::clone(&executor_calls);
                move |_: &super::SaveDirtyViewIntent, _: &JobContext| {
                    executor_calls.fetch_add(1, Ordering::SeqCst);
                    SaveDirtyViewCompletion::Saved { written_bytes: 8 }
                }
            }),
        )
        .unwrap();

    assert_eq!(adapter.begin_shutdown().len(), 1);
    release.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
    let batch = await_batch(&mut adapter);
    assert_eq!(executor_calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        batch.completion(document(1)),
        Some(&SaveDirtyViewCompletion::Cancelled)
    );
    jobs.pump_events_with_budget(JobEventPumpBudget::new(usize::MAX, Duration::from_secs(1)));
    assert!(bus
        .drain_deliveries(subscriber)
        .into_iter()
        .any(|delivery| matches!(
            delivery.message().payload(),
            EditorMessagePayload::Job(event)
                if event.label() == "save_dirty_document_1"
                    && matches!(event.kind(), JobEventKind::Cancelled)
        )));
    assert!(matches!(
        adapter.schedule(
            &SaveBatchFixture::new(&[(3, 8)]).request,
            |_| Ok(save_mutex("rejected")),
            executor(|_: &super::SaveDirtyViewIntent, _: &JobContext| {
                SaveDirtyViewCompletion::Saved { written_bytes: 8 }
            }),
        ),
        Err(SaveDirtyViewsAdmissionError::ShuttingDown)
    ));
}

#[test]
fn interactive_save_completion_pump_inspects_at_most_the_explicit_ticket_budget() {
    let fixture = SaveBatchFixture::new(&(1..=100).map(|value| (value, 1)).collect::<Vec<_>>());
    let mut adapter = SaveDirtyViewsJobAdapter::new(test_job_system_with_limits(
        EditorJobLimits::resolved(16, []),
    ));
    adapter
        .schedule(
            &fixture.request,
            |intent| Ok(save_mutex(intent.resource_key())),
            executor(|_: &super::SaveDirtyViewIntent, _: &JobContext| {
                SaveDirtyViewCompletion::Saved { written_bytes: 1 }
            }),
        )
        .unwrap();

    let first = adapter.pump_completed_with_budget(8);
    assert!(first.inspected_tickets() <= 8);
    let batch = first
        .into_completed()
        .unwrap_or_else(|| await_batch(&mut adapter));
    assert_eq!(batch.len(), 100);
}

fn await_batch(adapter: &mut SaveDirtyViewsJobAdapter) -> super::SaveDirtyViewsCompletionBatch {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let poll = adapter.pump_completed();
        if let Some(batch) = poll.into_completed() {
            return batch;
        }
        assert!(
            Instant::now() < deadline,
            "interactive save batch did not terminalize"
        );
        std::thread::yield_now();
    }
}

fn executor<F>(save: F) -> impl FnOnce() -> Arc<dyn SaveDirtyViewExecutor>
where
    F: Fn(&super::SaveDirtyViewIntent, &JobContext) -> SaveDirtyViewCompletion
        + Send
        + Sync
        + 'static,
{
    move || {
        let executor: Arc<dyn SaveDirtyViewExecutor> = Arc::new(save);
        executor
    }
}

fn save_mutex(value: &str) -> MutexGroup {
    MutexGroup::parse(format!(
        "save_document_{}",
        blake3::hash(value.as_bytes()).to_hex()
    ))
    .unwrap()
}

fn document(value: u64) -> DocumentId {
    DocumentId::new(value)
}

fn instance(value: u64) -> ToolkitInstanceId {
    ToolkitInstanceId::parse(format!("view.asset.{value}")).unwrap()
}

fn effect() -> DirtyExternalEffectId {
    DirtyExternalEffectId::parse("ui.source_buffer").unwrap()
}

struct SaveBatchFixture {
    transactions: Arc<EditorTransactionEngine>,
    dirty: DirtyRegistry,
    request: SaveDirtyViewsRequest,
}

impl SaveBatchFixture {
    fn new(documents: &[(u64, u64)]) -> Self {
        let transactions = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
        let dirty = DirtyRegistry::new(Arc::clone(&transactions));
        let toolkits = DocumentToolkitRegistry::<()>::default();
        let mut candidates = Vec::with_capacity(documents.len());
        for (value, estimated_bytes) in documents {
            let document = document(*value);
            dirty.register_document(document).unwrap();
            dirty.mark_external_effect(document, effect()).unwrap();
            toolkits
                .register(Arc::new(FixtureToolkit {
                    descriptor: DocumentToolkitDescriptor::new(
                        document,
                        instance(*value),
                        format!("Document {value}"),
                        ToolkitLayout::single_tab(
                            format!("layout.document.{value}"),
                            format!("tab.document.{value}"),
                        )
                        .unwrap(),
                    ),
                }))
                .unwrap();
            candidates.push(SaveDirtyViewCandidate::new(
                dirty.snapshot(document).unwrap(),
                instance(*value),
                transactions
                    .capture_save_token(HistoryContextId::Document(document))
                    .unwrap(),
                format!("project://assets/document-{value}.zasset"),
                *estimated_bytes,
            ));
        }
        let request = SaveDirtyViewsRequest::prepare(&toolkits.snapshot(), candidates).unwrap();
        Self {
            transactions,
            dirty,
            request,
        }
    }
}

struct FixtureContext {
    gateway: EditorRuntimeGatewayHandle,
}

impl Default for FixtureContext {
    fn default() -> Self {
        Self {
            gateway: EditorRuntimeGatewayHandle::detached(),
        }
    }
}

impl EditContext for FixtureContext {
    fn runtime_gateway(&self) -> &EditorRuntimeGatewayHandle {
        &self.gateway
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }

    fn restore_selection(&mut self, _snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct FixtureToolkit {
    descriptor: DocumentToolkitDescriptor,
}

impl DocumentToolkit<()> for FixtureToolkit {
    fn descriptor(&self) -> &DocumentToolkitDescriptor {
        &self.descriptor
    }

    fn save(&self, _host: &(), _context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure> {
        Ok(())
    }

    fn autosave_source_path(&self, _host: &()) -> Result<std::path::PathBuf, ToolkitSaveFailure> {
        Ok("fixture.zdoc".into())
    }

    fn capture_autosave(&self, _host: &()) -> Result<DocumentAutosavePayload, ToolkitSaveFailure> {
        Ok(DocumentAutosavePayload::new("fixture.zdoc", Vec::new()))
    }
}

struct GateJob {
    started: Sender<()>,
    release: Receiver<()>,
}

struct ImmediateJob;

impl EditorJob for ImmediateJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Ok(())
    }
}

impl GateJob {
    fn new(started: Sender<()>, release: Receiver<()>) -> Self {
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
