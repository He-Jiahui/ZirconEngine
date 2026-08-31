use super::*;

#[test]
fn final_autosave_bypasses_the_periodic_deadline_and_fences_regular_admission() {
    let root = temporary_root("adapter-final-autosave");
    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    let source = Arc::new(CountingSnapshotSource::success());
    let mut adapter = AutosaveJobAdapter::new(
        test_job_system_with_limits(EditorJobLimits::resolved(1, [])),
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );

    assert!(
        adapter
            .schedule_final(
                Duration::ZERO,
                &dirty,
                |_| 1,
                |requested| {
                    assert_eq!(requested, &document);
                    Some(AutosaveDocumentRequest::new(
                        requested.clone(),
                        AutosaveJobPolicy::for_save_mutex(
                            MutexGroup::parse("save_final_autosave").unwrap(),
                        ),
                        source.clone(),
                    ))
                }
            )
            .unwrap()
    );
    assert!(!adapter.is_accepting());
    assert!(matches!(
        adapter.schedule(Duration::from_secs(10), &dirty, |_| 1, |_| None),
        Err(AutosaveAdmissionError::ShuttingDown)
    ));

    let completion = wait_for_autosave_completion(&mut adapter, Duration::ZERO);
    assert_eq!(completion.succeeded(), 1);
    assert_eq!(completion.failed(), 0);
    assert_eq!(source.capture_count(), 1);
    remove_temporary_root(&root);
}

#[test]
fn service_drains_a_final_snapshot_before_shutting_down_the_shared_job_system() {
    let root = temporary_root("service-final-autosave");
    let document = document_id("scene_main");
    let source = Arc::new(CountingSnapshotSource::success());
    let service = EditorAutosaveService::new(
        test_job_system_with_limits(EditorJobLimits::resolved(1, [])),
        AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
    );
    service.poll_project(Some(&root));

    let shutdown = service.shutdown_with_final_autosave(
        vec![AutosaveDocumentRequest::new(
            document.clone(),
            AutosaveJobPolicy::for_save_mutex(
                MutexGroup::parse("save_service_final_autosave").unwrap(),
            ),
            source.clone(),
        )],
        Instant::now() + Duration::from_secs(5),
    );

    assert!(shutdown.unfinished_jobs().is_empty());
    assert_eq!(shutdown.outcomes().len(), 1);
    assert_eq!(shutdown.outcomes()[0].document(), &document);
    assert!(matches!(
        shutdown.outcomes()[0].kind(),
        AutosaveDocumentOutcomeKind::Saved { .. }
    ));
    assert!(shutdown.diagnostic_persistence_issues().is_empty());
    assert_eq!(source.capture_count(), 1);
    assert!(root.join(".zircon/autosave/scene_main/1.zscene").is_file());
    remove_temporary_root(&root);
}

#[test]
fn service_reports_unbound_final_requests_without_releasing_them_silently() {
    let document = document_id("scene_main");
    let source = Arc::new(CountingSnapshotSource::success());
    let service = EditorAutosaveService::new(
        test_job_system_with_limits(EditorJobLimits::resolved(1, [])),
        AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
    );

    let shutdown = service.shutdown_with_final_autosave(
        vec![AutosaveDocumentRequest::new(
            document.clone(),
            AutosaveJobPolicy::for_save_mutex(
                MutexGroup::parse("save_unbound_final_autosave").unwrap(),
            ),
            source.clone(),
        )],
        Instant::now() + Duration::from_secs(5),
    );

    assert!(shutdown.unfinished_jobs().is_empty());
    assert_eq!(shutdown.outcomes().len(), 1);
    assert_eq!(shutdown.outcomes()[0].document(), &document);
    assert_eq!(
        shutdown.outcomes()[0].failure_stage(),
        Some(AutosaveFailureStage::JobLifecycle)
    );
    assert_eq!(source.capture_count(), 0);
}

#[test]
fn service_drains_all_final_windows_when_admission_allows_one_document() {
    let root = temporary_root("service-final-autosave-windows");
    let first_document = document_id("scene_alpha");
    let second_document = document_id("scene_bravo");
    let source = Arc::new(CountingSnapshotSource::success());
    let jobs = test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
        EditorJobAdmissionLimits::new(1, 1024, Duration::from_secs(5)),
    ));
    let service =
        EditorAutosaveService::new(jobs, AutosavePolicy::new(Duration::from_secs(10)).unwrap());
    service.poll_project(Some(&root));

    let shutdown = service.shutdown_with_final_autosave(
        vec![
            AutosaveDocumentRequest::new(
                first_document.clone(),
                AutosaveJobPolicy::for_save_mutex(
                    MutexGroup::parse("save_service_final_alpha").unwrap(),
                ),
                source.clone(),
            ),
            AutosaveDocumentRequest::new(
                second_document.clone(),
                AutosaveJobPolicy::for_save_mutex(
                    MutexGroup::parse("save_service_final_bravo").unwrap(),
                ),
                source.clone(),
            ),
        ],
        Instant::now() + Duration::from_secs(5),
    );

    assert!(shutdown.unfinished_jobs().is_empty());
    assert!(shutdown.diagnostic_persistence_issues().is_empty());
    assert_eq!(shutdown.outcomes().len(), 2);
    assert!(
        shutdown
            .outcomes()
            .iter()
            .all(|outcome| matches!(outcome.kind(), AutosaveDocumentOutcomeKind::Saved { .. }))
    );
    assert_eq!(source.capture_count(), 2);
    assert!(root.join(".zircon/autosave/scene_alpha/1.zscene").is_file());
    assert!(root.join(".zircon/autosave/scene_bravo/1.zscene").is_file());
    remove_temporary_root(&root);
}
