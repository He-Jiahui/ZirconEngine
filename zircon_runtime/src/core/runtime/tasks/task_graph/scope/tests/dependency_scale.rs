use super::*;

#[test]
fn deep_dependency_chain_completes_in_order() {
    const DEPTH: u64 = 64;

    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("deep-dependency-chain"))
        .expect("running runtime should admit a scope");
    let order = Arc::new(std::sync::Mutex::new(Vec::with_capacity(DEPTH as usize)));
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let first_order = Arc::clone(&order);
    let mut tail = scope
        .submit(
            descriptor(10_001, TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                release_rx.recv().expect("chain head should be released");
                first_order.lock().unwrap().push(1);
            },
        )
        .expect("scope should admit the chain head");

    for sequence in 2..=DEPTH {
        let task_order = Arc::clone(&order);
        tail = scope
            .submit_after(
                std::slice::from_ref(&tail),
                descriptor(10_000 + sequence, TaskCancellationPolicy::FinishOnShutdown),
                move |_| task_order.lock().unwrap().push(sequence),
            )
            .expect("scope should admit the dependency chain");
    }

    release_tx.send(()).expect("chain head should be released");
    tail.wait();
    assert_eq!(
        *order.lock().unwrap(),
        (1..=DEPTH).collect::<Vec<_>>(),
        "every dependency must publish before its successor"
    );
    scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(2))
        .expect("deep dependency chain should drain without recursion overflow");
    assert_eq!(scope.census().completed, DEPTH);
}

#[test]
fn wide_fanout_combine_waits_for_all() {
    const FANOUT: usize = 64;

    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(8))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("wide-fanout"))
        .expect("running runtime should admit a scope");
    let release = Arc::new(AtomicBool::new(false));
    let completed = Arc::new(AtomicUsize::new(0));
    let mut prerequisites = Vec::with_capacity(FANOUT);

    for index in 0..FANOUT {
        let task_release = Arc::clone(&release);
        let task_completed = Arc::clone(&completed);
        prerequisites.push(
            scope
                .submit(
                    descriptor(
                        20_000 + index as u64,
                        TaskCancellationPolicy::FinishOnShutdown,
                    ),
                    move |_| {
                        while !task_release.load(Ordering::Acquire) {
                            std::thread::yield_now();
                        }
                        task_completed.fetch_add(1, Ordering::AcqRel);
                    },
                )
                .expect("scope should admit every fanout prerequisite"),
        );
    }

    let completed_for_fence = Arc::clone(&completed);
    let (observed_tx, observed_rx) = mpsc::sync_channel(1);
    let fence = scope
        .submit_after(
            &prerequisites,
            descriptor(21_000, TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                observed_tx
                    .send(completed_for_fence.load(Ordering::Acquire))
                    .expect("fan-in fence should publish its completed prerequisite count");
            },
        )
        .expect("scope should admit the fan-in fence");

    release.store(true, Ordering::Release);
    TaskHandle::wait_all(&prerequisites);
    assert_eq!(completed.load(Ordering::Acquire), FANOUT);
    fence.wait();
    assert_eq!(
        observed_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("fan-in fence should run"),
        FANOUT
    );
    scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(2))
        .expect("wide fanout should drain");
    assert_eq!(scope.census().completed, (FANOUT + 1) as u64);
}

#[test]
fn canonical_wait_all_observes_every_terminal_before_propagating_failure() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(2))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("canonical-wait-all"))
        .expect("running runtime should admit a scope");
    let failing = scope
        .submit(
            descriptor(30_001, TaskCancellationPolicy::FinishOnShutdown),
            |_| panic!("wait-all prerequisite failure"),
        )
        .expect("failing task should be admitted");
    let failing_probe = failing.clone();
    let (delayed_started_tx, delayed_started_rx) = mpsc::sync_channel(1);
    let (delayed_release_tx, delayed_release_rx) = mpsc::sync_channel(1);
    let delayed_completed = Arc::new(AtomicBool::new(false));
    let delayed_completed_for_task = Arc::clone(&delayed_completed);
    let delayed = scope
        .submit(
            descriptor(30_002, TaskCancellationPolicy::FinishOnShutdown),
            move |_| {
                delayed_started_tx
                    .send(())
                    .expect("delayed task should publish its start");
                delayed_release_rx
                    .recv()
                    .expect("test owner should release the delayed task");
                delayed_completed_for_task.store(true, Ordering::Release);
            },
        )
        .expect("delayed task should be admitted");
    delayed_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("delayed task should occupy its worker");
    assert!(catch_unwind(AssertUnwindSafe(|| failing_probe.wait())).is_err());

    let (wait_started_tx, wait_started_rx) = mpsc::sync_channel(1);
    let (wait_result_tx, wait_result_rx) = mpsc::sync_channel(1);
    let waiter = std::thread::spawn(move || {
        wait_started_tx
            .send(())
            .expect("waiter should publish its entry");
        let panicked = catch_unwind(AssertUnwindSafe(|| {
            TaskHandle::wait_all(&[failing, delayed]);
        }))
        .is_err();
        wait_result_tx
            .send(panicked)
            .expect("waiter should publish the combined result");
    });
    wait_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("waiter should enter before the delayed task is released");
    assert!(
        matches!(
            wait_result_rx.recv_timeout(Duration::from_millis(50)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ),
        "wait_all must not propagate failure before every input reaches terminal"
    );

    delayed_release_tx
        .send(())
        .expect("delayed task should be released");
    assert!(wait_result_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("wait_all should finish after the delayed task terminates"));
    waiter.join().expect("waiter thread should join");
    assert!(delayed_completed.load(Ordering::Acquire));

    scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(2))
        .expect("canonical wait-all tasks should drain");
}

#[test]
fn wait_all_empty_and_single_inputs_preserve_terminal_behavior() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("wait-all-small-inputs"))
        .expect("running task graph should admit a scope");
    let single = scope
        .submit(
            descriptor(31_001, TaskCancellationPolicy::FinishOnShutdown),
            |_| {},
        )
        .expect("single task should be admitted");

    TaskHandle::wait_all(&[]);
    TaskHandle::wait_all(std::slice::from_ref(&single));
    assert!(single.is_complete());

    scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(2))
        .expect("small-input wait-all task should drain");
}

#[test]
fn wait_all_small_input_paths_bypass_combined_completion_node() {
    let source = include_str!("../../task_handle.rs");
    let wait_all = source
        .split("pub fn wait_all")
        .nth(1)
        .and_then(|body| body.split("pub fn on_terminal").next())
        .expect("wait-all implementation");
    assert!(wait_all.contains("match handles"));
    assert!(wait_all.contains("[] => return"));
    assert!(wait_all.contains("[handle] => handle.wait()"));
    let small_input = wait_all
        .split("let completions")
        .next()
        .expect("small-input fast paths");
    assert!(!small_input.contains("collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Runtime02 performance evidence"]
fn runtime02_task_handle_wait_all_fast_path_evidence() {
    const SMALL_WAIT_CALLS: usize = 4_096;
    let legacy_combined_nodes = SMALL_WAIT_CALLS;
    let optimized_combined_nodes = 0;

    println!(
        "RUNTIME02_TASK_HANDLE_WAIT_ALL_FAST_PATH_BENCH_V1 empty_and_single_calls={} combined_nodes_before={} combined_nodes_after={}",
        SMALL_WAIT_CALLS, legacy_combined_nodes, optimized_combined_nodes,
    );
    assert!(legacy_combined_nodes > 0);
    assert_eq!(optimized_combined_nodes, 0);
}
