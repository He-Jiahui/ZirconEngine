use super::*;

#[test]
fn autosave_completion_pump_inspects_only_the_explicit_ticket_budget() {
    let root = temporary_root("adapter-completion-budget");
    let jobs =
        test_job_system_with_limits(EditorJobLimits::resolved(16, []).with_admission_limits(
            EditorJobAdmissionLimits::new(100, 1_024, Duration::from_secs(10)),
        ));
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let dirty = (0..100)
        .map(|index| {
            AutosaveDocumentState::from_dirty_for_test(
                document_id(&format!("scene_{index:03}")),
                true,
            )
        })
        .collect::<Vec<_>>();
    let source = Arc::new(CountingSnapshotSource::success());
    let save_mutex = MutexGroup::parse("save_completion_budget").unwrap();

    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 1,
                |document| {
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        source.clone(),
                    ))
                },
            )
            .unwrap()
    );

    let first = adapter.pump_completed_with_budget(Duration::from_secs(11), 8);
    assert_eq!(first.inspected_tickets(), 8);
    assert!(first.pending() != 0);
    let completion = wait_for_autosave_completion(&mut adapter, Duration::from_secs(11));
    assert_eq!(completion.succeeded(), 100);
    assert_eq!(completion.failed(), 0);
    remove_temporary_root(&root);
}

#[test]
fn autosave_completion_budget_preserves_zero_budget_and_rotates_a_blocked_head() {
    let root = temporary_root("adapter-completion-rotation");
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(4, []));
    let blocked_mutex = MutexGroup::parse("save_blocked_head").unwrap();
    let ready_mutex = MutexGroup::parse("save_ready_tail").unwrap();
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("autosave-completion-blocker", JobCategory::Import)
                .with_mutex_group(blocked_mutex.clone()),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let blocked_source = Arc::new(CountingSnapshotSource::success());
    let ready_source = Arc::new(CountingSnapshotSource::success());
    let dirty = [
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_a"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_b"), true),
    ];
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
                |document| {
                    let (save_mutex, source) = if document.as_str() == "scene_a" {
                        (blocked_mutex.clone(), blocked_source.clone())
                    } else {
                        (ready_mutex.clone(), ready_source.clone())
                    };
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex),
                        source,
                    ))
                },
            )
            .unwrap()
    );
    wait_for_capture_count(&ready_source, 1);

    let zero = adapter.pump_completed_with_budget(Duration::from_secs(11), 0);
    assert_eq!(zero.inspected_tickets(), 0);
    assert_eq!(zero.pending(), 2);
    assert_eq!(zero.succeeded(), 0);

    let blocked = adapter.pump_completed_with_budget(Duration::from_secs(11), 1);
    assert_eq!(blocked.inspected_tickets(), 1);
    assert_eq!(blocked.pending(), 2);
    assert_eq!(blocked.succeeded(), 0);
    let ready = wait_for_autosave_completion_state(
        &mut adapter,
        Duration::from_secs(11),
        1,
        |completion| completion.pending() == 1 && completion.succeeded() == 1,
    );
    assert_eq!(ready.pending(), 1);
    assert_eq!(ready.succeeded(), 1);

    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
    let completion = wait_for_autosave_completion(&mut adapter, Duration::from_secs(11));
    assert_eq!(completion.succeeded(), 2);
    assert_eq!(completion.failed(), 0);
    remove_temporary_root(&root);
}

#[test]
fn autosave_completion_counts_accumulate_then_reset_for_the_next_interval() {
    let root = temporary_root("adapter-completion-reset");
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(4, []));
    let succeeded_source = Arc::new(CountingSnapshotSource::success());
    let failed_source = Arc::new(CountingSnapshotSource::failure());
    let save_mutex = MutexGroup::parse("save_completion_reset").unwrap();
    let dirty = [
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_a"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_b"), true),
    ];
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
                |document| {
                    let source: Arc<dyn AutosaveSnapshotSource> = if document.as_str() == "scene_a"
                    {
                        succeeded_source.clone()
                    } else {
                        failed_source.clone()
                    };
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        source,
                    ))
                },
            )
            .unwrap()
    );
    wait_for_capture_count(&succeeded_source, 1);
    wait_for_capture_count(&failed_source, 1);

    let first = wait_for_autosave_completion_state(
        &mut adapter,
        Duration::from_secs(11),
        1,
        |completion| completion.pending() == 1 && completion.succeeded() + completion.failed() == 1,
    );
    assert_eq!(first.pending(), 1);
    assert_eq!(first.succeeded() + first.failed(), 1);
    let terminal = wait_for_autosave_completion_state(
        &mut adapter,
        Duration::from_secs(11),
        1,
        |completion| {
            completion.pending() == 0 && completion.succeeded() == 1 && completion.failed() == 1
        },
    );
    assert_eq!(terminal.succeeded(), 1);
    assert_eq!(terminal.failed(), 1);
    assert_eq!(terminal.pending(), 0);
    assert!(!adapter.is_in_flight());

    let reset = adapter.pump_completed_with_budget(Duration::from_secs(11), 1);
    assert_eq!(reset.succeeded(), 0);
    assert_eq!(reset.failed(), 0);
    assert_eq!(reset.pending(), 0);
    assert!(
        adapter
            .schedule(
                Duration::from_secs(21),
                &dirty[..1],
                |_| 1,
                |document| {
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        Arc::new(CountingSnapshotSource::success()),
                    ))
                },
            )
            .unwrap()
    );
    assert_eq!(
        wait_for_autosave_completion(&mut adapter, Duration::from_secs(22)).succeeded(),
        1
    );
    remove_temporary_root(&root);
}
