use super::*;

use crate::core::jobs::{EditorJob, EditorJobSpec, JobContext};
use crate::core::logging::{
    EditorLogConfig, EditorLogService, LogChannel, LogFilter, LogJumpTarget, LogSeverity,
};

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use zircon_runtime::asset::ProjectImportReceipt;

fn import_records(logs: &EditorLogService) -> Vec<crate::core::logging::LogRecord> {
    logs.snapshot(&LogFilter::new(
        std::collections::BTreeSet::from([LogChannel::Import]),
        LogSeverity::Info,
    ))
}

#[test]
fn repeated_result_observation_projects_one_import_completion() {
    let jobs = test_job_system();
    let backend = Arc::new(RecordingBackend::default());
    let index = index_for("res://textures/logged.png");
    let logs = Arc::new(EditorLogService::default());
    let flow = EditorAssetImportFlow::with_backend_and_logs(
        jobs,
        Arc::clone(&backend),
        index,
        Arc::clone(&logs),
    );
    let target = uri("res://textures/logged.png");
    *backend
        .status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(imported_status(&target));

    let ticket = flow
        .submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::Manual,
        ))
        .unwrap();
    ticket.wait().unwrap();
    assert!(ticket.try_result().unwrap().is_ok());
    assert!(ticket.try_result().unwrap().is_ok());

    let records = import_records(logs.as_ref());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].entry().severity(), LogSeverity::Info);
    assert!(records[0].entry().message().contains("result=committed"));
    assert!(matches!(
        records[0].entry().jump().map(|jump| jump.target()),
        Some(LogJumpTarget::Asset(path)) if path.as_ref() == target.to_string()
    ));
}

#[test]
fn import_warning_and_failure_keep_the_import_channel_and_asset_jump() {
    let jobs = test_job_system();
    let warning_backend = Arc::new(RecordingBackend::default());
    let warning_index = index_for("res://textures/warning.png");
    let logs = Arc::new(EditorLogService::default());
    let warning_flow = EditorAssetImportFlow::with_backend_and_logs(
        jobs.clone(),
        Arc::clone(&warning_backend),
        warning_index,
        Arc::clone(&logs),
    );
    let warning_uri = uri("res://textures/warning.png");
    let mut warning_status = imported_status(&warning_uri);
    warning_status.imported = false;
    *warning_backend
        .status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(warning_status);
    warning_flow
        .submit(EditorAssetImportRequest::new(
            warning_uri.clone(),
            EditorAssetImportReason::Watch,
        ))
        .unwrap()
        .wait()
        .unwrap();

    let failure_backend = Arc::new(RecordingBackend::default());
    failure_backend.fail.store(true, Ordering::SeqCst);
    let failure_uri = uri("res://textures/failure.png");
    let failure_flow = EditorAssetImportFlow::with_backend_and_logs(
        jobs,
        failure_backend,
        index_for("res://textures/failure.png"),
        Arc::clone(&logs),
    );
    assert!(failure_flow
        .submit(EditorAssetImportRequest::new(
            failure_uri.clone(),
            EditorAssetImportReason::DigestMismatch,
        ))
        .unwrap()
        .wait()
        .is_err());

    let records = import_records(logs.as_ref());
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].entry().severity(), LogSeverity::Warning);
    assert_eq!(records[1].entry().severity(), LogSeverity::Error);
    assert!(records.iter().all(|record| matches!(
        record.entry().jump().map(|jump| jump.target()),
        Some(LogJumpTarget::Asset(_))
    )));
}

struct PerUriBackend;

impl AssetImportBackend for PerUriBackend {
    fn import(&self, uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError> {
        Ok(Some(imported_status(uri)))
    }
}

#[test]
fn import_completion_storm_uses_the_bounded_editor_log_store() {
    let entries = [
        ("storm-0", "res://textures/storm-0.png", "d0"),
        ("storm-1", "res://textures/storm-1.png", "d1"),
        ("storm-2", "res://textures/storm-2.png", "d2"),
        ("storm-3", "res://textures/storm-3.png", "d3"),
        ("storm-4", "res://textures/storm-4.png", "d4"),
        ("storm-5", "res://textures/storm-5.png", "d5"),
        ("storm-6", "res://textures/storm-6.png", "d6"),
        ("storm-7", "res://textures/storm-7.png", "d7"),
    ];
    let logs = Arc::new(EditorLogService::new(
        EditorLogConfig::new(4, 64 * 1024).unwrap(),
    ));
    let flow = EditorAssetImportFlow::with_backend_and_logs(
        test_job_system(),
        Arc::new(PerUriBackend),
        index_for_assets(&entries),
        Arc::clone(&logs),
    );

    for (_, path, _) in entries {
        flow.submit(EditorAssetImportRequest::new(
            uri(path),
            EditorAssetImportReason::Manual,
        ))
        .unwrap()
        .wait()
        .unwrap();
    }

    let records = import_records(logs.as_ref());
    assert_eq!(records.len(), 4);
    assert!(records[0].entry().message().contains("storm-4.png"));
    assert!(records[3].entry().message().contains("storm-7.png"));
}

struct ModelImportGate {
    started: Sender<()>,
    release: Receiver<()>,
}

impl EditorJob for ModelImportGate {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        let _ = self.started.send(());
        self.release.recv().map_err(JobError::failed)
    }
}

struct PendingModelImportMustNotRun {
    diagnostics: Arc<EditorModelImportDiagnostics>,
}

impl Drop for PendingModelImportMustNotRun {
    fn drop(&mut self) {
        self.diagnostics.project_result(Err(JobError::Cancelled));
    }
}

impl EditorJob for PendingModelImportMustNotRun {
    type Output = ProjectImportReceipt;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        panic!("pending model import must be cancelled before execution")
    }
}

#[test]
fn pending_model_cancel_projects_one_warning_without_result_observation() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Import, 1));
    let (started_sender, started_receiver) = std::sync::mpsc::channel();
    let (release_sender, release_receiver) = std::sync::mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("model import blocker", JobCategory::Import),
            ModelImportGate {
                started: started_sender,
                release: release_receiver,
            },
        )
        .unwrap();
    started_receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap();

    let logs = Arc::new(EditorLogService::default());
    let diagnostics = Arc::new(EditorModelImportDiagnostics::new(
        PathBuf::from("D:/imports/cancelled.glb"),
        EditorAssetImportDiagnostics::new(Arc::clone(&logs)),
    ));
    diagnostics.arm();
    let raw_ticket = jobs
        .submit(
            EditorJobSpec::new("pending model import", JobCategory::Import),
            PendingModelImportMustNotRun {
                diagnostics: Arc::clone(&diagnostics),
            },
        )
        .unwrap();
    let ticket =
        EditorModelImportTicket::new(raw_ticket, PathBuf::from("D:/imports/cancelled.glb"));

    assert!(jobs.cancel(ticket.id()));
    let records = import_records(logs.as_ref());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].entry().severity(), LogSeverity::Warning);
    assert!(records[0].entry().message().contains("cancelled.glb"));
    assert!(records[0].entry().jump().is_none());
    let deadline = Instant::now() + Duration::from_secs(5);
    let result = loop {
        if let Some(result) = ticket.try_take() {
            break result;
        }
        assert!(
            Instant::now() < deadline,
            "cancelled ticket missed its deadline"
        );
        std::thread::yield_now();
    };
    release_sender.send(()).unwrap();
    blocker.wait().unwrap();

    assert!(matches!(result, Err(JobError::Cancelled)));
    assert_eq!(ticket.try_take(), None);
    let records = import_records(logs.as_ref());
    assert_eq!(records.len(), 1);
}

#[test]
fn model_completion_before_submission_arm_is_deferred_and_emitted_once() {
    let logs = Arc::new(EditorLogService::default());
    let diagnostics = EditorModelImportDiagnostics::new(
        PathBuf::from("D:/imports/early-cancel.glb"),
        EditorAssetImportDiagnostics::new(Arc::clone(&logs)),
    );

    diagnostics.project_result(Err(JobError::Cancelled));
    assert!(import_records(logs.as_ref()).is_empty());
    diagnostics.arm();
    diagnostics.arm();
    diagnostics.project_result(Err(JobError::Cancelled));

    let records = import_records(logs.as_ref());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].entry().severity(), LogSeverity::Warning);
}

#[test]
fn submission_rejection_overrides_a_pre_arm_cancel_projection() {
    let logs = Arc::new(EditorLogService::default());
    let diagnostics = EditorModelImportDiagnostics::new(
        PathBuf::from("D:/imports/rejected.glb"),
        EditorAssetImportDiagnostics::new(Arc::clone(&logs)),
    );

    diagnostics.project_result(Err(JobError::Cancelled));
    diagnostics.reject_submission("editor job system is shutting down");
    diagnostics.arm();

    let records = import_records(logs.as_ref());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].entry().severity(), LogSeverity::Error);
    assert!(records[0].entry().message().contains("result=rejected"));
    assert!(records[0].entry().message().contains("shutting down"));
}

#[test]
fn asset_submission_rejection_overrides_pre_arm_drop_cancellation() {
    let jobs = test_job_system();
    assert!(jobs.shutdown(Instant::now()).is_empty());
    let logs = Arc::new(EditorLogService::default());
    let target = uri("res://textures/rejected.png");
    let flow = EditorAssetImportFlow::with_backend_and_logs(
        jobs,
        Arc::new(RecordingBackend::default()),
        index_for("res://textures/rejected.png"),
        Arc::clone(&logs),
    );

    let error = flow
        .submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::Manual,
        ))
        .unwrap_err();

    assert!(matches!(
        error,
        EditorAssetImportSubmitError::Job(JobSubmitError::ShuttingDown)
    ));
    let records = import_records(logs.as_ref());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].entry().severity(), LogSeverity::Error);
    assert!(records[0].entry().message().contains("result=rejected"));
    assert!(records[0].entry().message().contains("shutting down"));
    assert!(matches!(
        records[0].entry().jump().map(|jump| jump.target()),
        Some(LogJumpTarget::Asset(path)) if path.as_ref() == target.to_string()
    ));
}
