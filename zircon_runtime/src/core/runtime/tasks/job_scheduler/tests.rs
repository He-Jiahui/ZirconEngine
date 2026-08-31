use std::panic::{self, AssertUnwindSafe};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::time::Duration;

use super::super::callback_dispatcher::TaskCallbackDispatcher;
use super::{
    run_detached_task, JobExecutionOutcome, JobHandle, JobScheduler, JobSchedulerDiagnosticsState,
    PendingScheduledJob, PendingScheduledWork, TaskPool, TaskPoolDescriptor,
};

#[test]
fn from_pool_preserves_the_explicit_execution_owner_and_lane() {
    let pool = TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1));
    let scheduler = JobScheduler::from_pool(pool.clone());

    assert_eq!(scheduler.pool_kind(), super::TaskPoolKind::Io);
    assert!(scheduler.shares_execution_owner_with(&pool));
}

#[test]
fn schedule_after_propagates_dependency_cancellation_without_running_dependent_work() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ));
    let dependency = JobHandle::pending_with_dependencies(0);
    let (ran_tx, ran_rx) = std::sync::mpsc::sync_channel(1);
    let dependent = scheduler.schedule_after(&[dependency.clone()], move || {
        ran_tx
            .send(())
            .expect("cancelled dependencies must not run dependent work");
    });

    dependency.mark_cancelled();
    dependent.wait();

    assert!(dependent.is_cancelled());
    assert!(ran_rx.try_recv().is_err());
}

#[test]
fn dependency_panic_is_not_misclassified_as_prelaunch_cancellation() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ));
    let source = scheduler.task_diagnostic_source();
    let cursor = source.initial_cursor();
    let dependency = JobHandle::pending_with_dependencies(0);
    let dependent = scheduler.schedule_after(&[dependency.clone()], || {});

    dependency.mark_panicked(Arc::from("dependency panic"));
    let wait_result = panic::catch_unwind(AssertUnwindSafe(|| dependent.wait()));

    assert!(wait_result.is_err());
    let report = scheduler.diagnostic_report();
    assert_eq!(report.panicked, 1);
    assert_eq!(report.cancelled, 0);
    let batch = source.read_after(cursor, 8);
    assert_eq!(batch.observations().len(), 1);
    assert_eq!(
        batch.observations()[0].kind(),
        super::TaskDiagnosticKind::Panicked
    );
    assert_eq!(batch.observations()[0].message(), "dependency panic");
}

#[test]
fn first_dependency_terminal_winner_owns_handle_metrics_and_observation() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ));
    let source = scheduler.task_diagnostic_source();
    let cursor = source.initial_cursor();
    let cancelled_dependency = JobHandle::pending_with_dependencies(0);
    let panicked_dependency = JobHandle::pending_with_dependencies(0);
    let dependent = scheduler.schedule_after(
        &[cancelled_dependency.clone(), panicked_dependency.clone()],
        || {},
    );

    cancelled_dependency.mark_cancelled();
    dependent.wait();
    panicked_dependency.mark_panicked(Arc::from("late dependency panic"));

    assert!(dependent.is_cancelled());
    let report = scheduler.diagnostic_report();
    assert_eq!(report.cancelled, 1);
    assert_eq!(report.panicked, 0);
    let batch = source.read_after(cursor, 8);
    assert_eq!(batch.observations().len(), 1);
    assert_eq!(
        batch.observations()[0].kind(),
        super::TaskDiagnosticKind::Cancelled
    );
}

#[test]
fn scheduled_task_records_one_execution_sample_after_completion() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ))
    .with_diagnostics();

    scheduler.schedule(|| {}).wait();

    let report = scheduler.diagnostic_report();
    assert_eq!(report.scheduled, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(report.execution_samples, 1);
    assert!(report.execution_ms >= 0.0);
}

#[test]
fn detached_spawn_counts_panicked_tasks_as_completed() {
    const CHILD_ENV: &str = "ZIRCON_DETACHED_PANIC_DIAGNOSTICS_CHILD";
    const CHILD_STARTED: &str = "zircon detached panic child started";
    const CHILD_SURVIVED_EXIT_CODE: i32 = 91;

    if std::env::var_os(CHILD_ENV).is_some() {
        eprintln!("{CHILD_STARTED}");
        let scheduler = JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::compute().with_worker_threads(1),
        ));
        scheduler.spawn(|| panic!("detached task failure"));
        std::thread::sleep(Duration::from_secs(2));
        std::process::exit(CHILD_SURVIVED_EXIT_CODE);
    }

    let diagnostics = Arc::new(JobSchedulerDiagnosticsState::default());
    diagnostics.enable();
    let enqueued_at = diagnostics.record_scheduled_and_enqueued();
    let tracked = diagnostics.record_started(enqueued_at);
    let execution_started_at = diagnostics.execution_started_at(tracked);
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        run_detached_task(Arc::clone(&diagnostics), None, execution_started_at, || {
            panic!("detached task failure")
        });
    }));
    assert!(result.is_err());
    let report = diagnostics.report();
    assert_eq!(report.scheduled, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(report.panicked, 1);
    assert_eq!(report.queued, 0);
    assert_eq!(report.active, 0);
    assert_eq!(report.execution_samples, 1);

    let test_executable = std::env::current_exe().expect("current lib-test executable");
    let listed = Command::new(&test_executable)
        .arg("--list")
        .output()
        .expect("list current lib-test names");
    assert!(
        listed.status.success(),
        "current lib-test list must succeed"
    );
    let test_suffix = "::detached_spawn_counts_panicked_tasks_as_completed";
    let listed_stdout = String::from_utf8_lossy(&listed.stdout);
    let test_name = listed_stdout
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .find(|name| name.ends_with(test_suffix))
        .unwrap_or_else(|| panic!("lib-test list should contain `{test_suffix}`"))
        .to_owned();

    let output = Command::new(test_executable)
        .args(["--exact", test_name.as_str(), "--nocapture"])
        .env(CHILD_ENV, "1")
        .output()
        .expect("launch isolated detached-panic diagnostic test");

    assert!(
        !output.status.success(),
        "real Rayon detached panic must retain its process-terminating default"
    );
    assert_ne!(
        output.status.code(),
        Some(CHILD_SURVIVED_EXIT_CODE),
        "detached task did not reach Rayon's panic termination path"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(CHILD_STARTED));
    assert!(
        stderr.contains("detached task failure"),
        "child stderr should prove the selected detached task actually panicked: {stderr}"
    );
}

#[test]
fn pending_scheduled_job_recovers_poisoned_task_lock() {
    let diagnostics = Arc::new(JobSchedulerDiagnosticsState::default());
    let pool = TaskPool::new(TaskPoolDescriptor::compute());
    let handle = JobHandle::pending_with_scheduler_diagnostics(
        0,
        Arc::clone(&diagnostics),
        TaskCallbackDispatcher::new(pool.clone()),
    );
    let task_ran = Arc::new(AtomicBool::new(false));
    let task_ran_for_job = Arc::clone(&task_ran);
    let pending = PendingScheduledJob {
        handle: handle.clone(),
        diagnostics,
        identity: None,
        created_at: None,
        diagnostics_tracked: false,
        dependency_count: 0,
        work: Mutex::new(Some(PendingScheduledWork {
            submission: pool
                .try_acquire_submission()
                .expect("open pool should issue a submission authority"),
            prelaunch_terminal: None,
            task: Box::new(move || {
                task_ran_for_job.store(true, Ordering::SeqCst);
                JobExecutionOutcome::Completed
            }),
        })),
    };

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = pending.work.lock().unwrap();
        panic!("poison pending scheduled job task");
    }));

    pending.try_launch();
    handle.wait();
    assert!(task_ran.load(Ordering::SeqCst));
}

#[test]
fn accepted_dependency_chain_completes_after_external_admission_closes() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let scheduler = JobScheduler::from_pool(pool.clone());
    let dependency = JobHandle::pending_with_dependencies(0);
    let (sender, receiver) = mpsc::sync_channel(1);
    let dependent = scheduler.schedule_after(&[dependency.clone()], move || {
        sender.send(()).expect("dependent task result");
    });

    pool.close_admission();
    dependency.mark_complete();

    dependent.wait();
    receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("accepted continuation must drain after external admission closes");
}
