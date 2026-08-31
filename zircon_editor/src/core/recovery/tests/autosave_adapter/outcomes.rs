use super::*;

#[test]
fn autosave_failure_completion_keeps_document_source_stage_and_retryability() {
    let root = temporary_root("adapter-document-outcome");
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(1, []));
    let document = document_id("scene_main");
    let source_path = recovery_source_path("scenes/main.zscene");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 1,
                |requested| {
                    Some(AutosaveDocumentRequest::new(
                        requested.clone(),
                        AutosaveJobPolicy::for_save_mutex(
                            MutexGroup::parse("save_document_outcome").unwrap(),
                        ),
                        Arc::new(CountingSnapshotSource::failure()),
                    ))
                },
            )
            .unwrap()
    );

    let completion = wait_for_autosave_completion(&mut adapter, Duration::from_secs(11));
    assert_eq!(completion.failed(), 1);
    assert_eq!(completion.outcomes().len(), 1);
    let outcome = &completion.outcomes()[0];
    assert_eq!(outcome.document(), &document);
    assert_eq!(outcome.source_path(), &source_path);
    assert_eq!(outcome.failure_stage(), Some(AutosaveFailureStage::Capture));
    assert_eq!(outcome.retryability(), AutosaveRetryability::Retryable);
    assert!(outcome.usable_snapshot().is_none());
    assert!(
        outcome
            .error_chain()
            .iter()
            .any(|message| message.contains("snapshot failure"))
    );
    assert!(outcome.diagnostic_persisted());

    let report = AutosaveDiagnosticStore::new(&root).load().unwrap();
    assert_eq!(report.records().len(), 1);
    assert_eq!(report.records()[0].outcome(), outcome);
    assert_eq!(
        AutosaveDiagnosticStore::new(&root).document_folder(&document),
        root.join(".zircon/autosave/scene_main")
    );
    remove_temporary_root(&root);
}

#[test]
fn project_switch_persists_cancelled_autosave_outcome_for_the_retired_project() {
    let first_root = temporary_root("service-project-switch-first");
    let second_root = temporary_root("service-project-switch-second");
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Import, 1));
    let service = EditorAutosaveService::new(
        jobs.clone(),
        AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
    );
    let blocked_mutex = MutexGroup::parse("save_retired_autosave").unwrap();
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("retired-autosave-blocker", JobCategory::Import)
                .with_mutex_group(blocked_mutex.clone()),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    assert_eq!(
        service.poll_project(Some(&first_root)).now(),
        Duration::ZERO
    );
    assert!(
        service
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 1,
                |requested| {
                    Some(AutosaveDocumentRequest::new(
                        requested.clone(),
                        AutosaveJobPolicy::for_save_mutex(blocked_mutex.clone()),
                        Arc::new(CountingSnapshotSource::success()),
                    ))
                },
            )
            .unwrap()
    );

    service.poll_project(Some(&second_root));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        service.poll_project(Some(&second_root));
        let report = AutosaveDiagnosticStore::new(&first_root).load().unwrap();
        if let Some(record) = report.records().first() {
            assert_eq!(record.outcome().document(), &document);
            assert_eq!(
                record.outcome().failure_stage(),
                Some(AutosaveFailureStage::Cancelled)
            );
            assert_eq!(
                record.outcome().retryability(),
                AutosaveRetryability::NotRetryable
            );
            assert!(record.outcome().diagnostic_persisted());
            break;
        }
        assert!(
            Instant::now() < deadline,
            "retired project autosave cancellation was not persisted"
        );
        thread::yield_now();
    }

    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
    remove_temporary_root(&first_root);
    remove_temporary_root(&second_root);
}
