use super::*;

#[test]
fn supplied_scheduler_preserves_foreign_execution_owner_and_domain() {
    for kind in [TaskPoolKind::Io, TaskPoolKind::Compute] {
        let original =
            EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
        let target =
            EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
        let scope = target
            .create_scope(TaskGraphScopeDescriptor::new("foreign-admission"))
            .unwrap();
        let scheduler = original.scheduler(kind);
        let (blockers, releases) = occupy_domain(&scheduler);
        let (ran_tx, ran_rx) = mpsc::sync_channel(1);
        let task = scope
            .submit_on_scheduler(
                &scheduler,
                TaskDescriptor::new(TaskId::new(900), kind, "foreign-worker"),
                move |_| {
                    ran_tx.send(()).unwrap();
                },
            )
            .unwrap();
        let original_owner = original.task_belongs_to_owner(&task);
        let target_owner = target.task_belongs_to_owner(&task);
        let ran_while_original_domain_occupied = ran_rx.try_recv().is_ok();
        let submitted = scope.census().submitted;
        release_domain(blockers, releases);
        task.wait();
        if !ran_while_original_domain_occupied {
            ran_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        }
        let quiescent = scope.wait_until_quiescent(Duration::from_secs(1));
        original.shutdown(Duration::from_secs(1)).unwrap();
        target.shutdown(Duration::from_secs(1)).unwrap();
        assert_eq!(task.descriptor().kind, kind);
        assert!(original_owner);
        assert!(!target_owner);
        assert!(!ran_while_original_domain_occupied, "foreign target must not execute the task while every original domain worker is occupied");
        assert_eq!(submitted, 1);
        assert!(quiescent);
    }
}

#[test]
fn supplied_original_stopped_or_closing_precedes_dead_foreign_scope() {
    for stopped in [false, true] {
        let original =
            EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
        let foreign =
            EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
        let dead_scope = foreign
            .create_scope(TaskGraphScopeDescriptor::new("dead-foreign"))
            .unwrap();
        drop(foreign);
        let scheduler = original.scheduler(TaskPoolKind::Compute);
        if stopped {
            original.shutdown(Duration::from_secs(1)).unwrap();
        } else {
            let admission_scope = original
                .create_scope(TaskGraphScopeDescriptor::new("keep-closing"))
                .unwrap();
            let (start_tx, start_rx) = mpsc::sync_channel(1);
            let (finish_tx, finish_rx) = mpsc::sync_channel(1);
            let blocker = admission_scope
                .submit(
                    descriptor(904, TaskCancellationPolicy::FinishOnShutdown),
                    move |_| {
                        start_tx.send(()).unwrap();
                        finish_rx.recv().unwrap();
                    },
                )
                .unwrap();
            start_rx.recv_timeout(Duration::from_secs(1)).unwrap();
            assert!(original.shutdown(Duration::ZERO).is_err());
            finish_tx.send(()).unwrap();
            blocker.wait();
        }
        let error = dead_scope
            .submit_on_scheduler(
                &scheduler,
                descriptor(901, TaskCancellationPolicy::FinishOnShutdown),
                |_| panic!("refused task must not run"),
            )
            .unwrap_err();
        assert_eq!(
            error,
            if stopped {
                TaskGraphAdmissionError::RuntimeStopped
            } else {
                TaskGraphAdmissionError::RuntimeClosing
            }
        );
        assert_eq!(dead_scope.census().submitted, 0);
        assert_eq!(dead_scope.census().queued, 0);
        if !stopped {
            original.shutdown(Duration::from_secs(1)).unwrap();
        }
    }
}

#[test]
fn scheduler_domain_mismatch_and_closed_scope_do_not_admit_tasks() {
    let original =
        EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
    let target = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
    let scope = target
        .create_scope(TaskGraphScopeDescriptor::new("closed-target"))
        .unwrap();
    let scheduler = original.scheduler(TaskPoolKind::Io);
    assert!(matches!(
        scope.submit_on_scheduler(
            &scheduler,
            descriptor(902, TaskCancellationPolicy::FinishOnShutdown),
            |_| panic!("must not run")
        ),
        Err(TaskGraphAdmissionError::SchedulerKindMismatch { .. })
    ));
    scope.close_admission();
    assert!(matches!(
        scope.submit_on_scheduler(
            &scheduler,
            TaskDescriptor::new(TaskId::new(903), TaskPoolKind::Io, "closed"),
            |_| panic!("must not run")
        ),
        Err(TaskGraphAdmissionError::ScopeClosed { .. })
    ));
    assert_eq!(scope.census().submitted, 0);
    original.shutdown(Duration::from_secs(1)).unwrap();
    target.shutdown(Duration::from_secs(1)).unwrap();
}

fn occupy_domain(
    scheduler: &JobScheduler,
) -> (Vec<crate::core::JobHandle>, Vec<mpsc::SyncSender<()>>) {
    let workers = scheduler.parallelism();
    let (started_tx, started_rx) = mpsc::sync_channel(workers);
    let mut blockers = Vec::with_capacity(workers);
    let mut releases = Vec::with_capacity(workers);
    for _ in 0..workers {
        let started = started_tx.clone();
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        blockers.push(scheduler.schedule(move || {
            started.send(()).unwrap();
            release_rx.recv().unwrap();
        }));
        releases.push(release_tx);
    }
    for _ in 0..workers {
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    }
    (blockers, releases)
}

fn release_domain(blockers: Vec<crate::core::JobHandle>, releases: Vec<mpsc::SyncSender<()>>) {
    for release in releases {
        release.send(()).unwrap();
    }
    for blocker in blockers {
        blocker.wait();
    }
}

#[test]
fn foreign_scope_shutdown_cancels_queued_task_on_original_domain() {
    let original =
        EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
    let target = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
    let scope = target
        .create_scope(TaskGraphScopeDescriptor::new("foreign-cancel"))
        .unwrap();
    let scheduler = original.scheduler(TaskPoolKind::Io);
    let (blockers, releases) = occupy_domain(&scheduler);
    let ran = Arc::new(AtomicBool::new(false));
    let ran_for_task = Arc::clone(&ran);
    let task = scope
        .submit_on_scheduler(
            &scheduler,
            TaskDescriptor::new(TaskId::new(905), TaskPoolKind::Io, "foreign-cancelled")
                .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop),
            move |_| {
                ran_for_task.store(true, Ordering::SeqCst);
            },
        )
        .unwrap();
    let original_owner = original.task_belongs_to_owner(&task);
    let target_close = target.shutdown(Duration::ZERO);
    release_domain(blockers, releases);
    task.wait();
    let quiescent = scope.wait_until_quiescent(Duration::from_secs(1));
    let state = task.status().state;
    let census = scope.census();
    target.shutdown(Duration::from_secs(1)).unwrap();
    original.shutdown(Duration::from_secs(1)).unwrap();
    assert!(original_owner);
    assert!(
        target_close.is_err(),
        "original pool is occupied until its queued cancellation retires"
    );
    assert!(!ran.load(Ordering::SeqCst));
    assert_eq!(state, TaskState::Cancelled);
    assert!(quiescent);
    assert_eq!(census.queued, 0);
    assert_eq!(census.running, 0);
}
