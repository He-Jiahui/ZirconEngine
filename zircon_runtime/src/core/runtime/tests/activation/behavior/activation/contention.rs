use super::*;

// 同一模块的七个 public Activate 调用在 owner 的 build 闸门内汇合；计时在确认七名等待者登记后才开始。
// 断言共同成功、只构建一次且最终状态为 Running。
#[test]
fn concurrent_activation_joiners_share_one_build_within_contention_budget() {
    let elapsed = activation_join_sample("ConcurrentActivationBudgetModule");

    assert!(
        elapsed <= Duration::from_millis(750),
        "seven already-waiting activation joiners must finish within 750ms, took {elapsed:?}"
    );
}

// 对二十一个独立模块重复相同竞争样本并输出 P50/P95；保留 release-only 忽略，
// 避免普通测试运行把性能样本当作正确性门槛。
#[test]
#[ignore = "release-only 21-sample activation contention evidence"]
fn concurrent_activation_joiners_release_benchmark_evidence() {
    const SAMPLE_COUNT: usize = 21;
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        samples.push(activation_join_sample(&format!(
            "ConcurrentActivationBenchmarkModule{sample_index}"
        )));
    }

    samples.sort_unstable();
    let p50 = samples[(SAMPLE_COUNT * 50).div_ceil(100) - 1];
    let p95 = samples[(SAMPLE_COUNT * 95).div_ceil(100) - 1];
    println!(
        "PERF_RESULT runtime01_activation_join sample_count={SAMPLE_COUNT} joiners=7 builds=1 p50_ms={:.3} p95_ms={:.3}",
        p50.as_secs_f64() * 1_000.0,
        p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        p95 <= Duration::from_millis(750),
        "activation join P95 must remain within 750ms, observed {p95:?}"
    );
}

// 先让所有等待者进入生命周期协调器，再释放唯一的构建者；计时覆盖释放后至全部等待者结束的协调成本。
fn activation_join_sample(module_name: &str) -> Duration {
    let runtime = CoreRuntime::new();
    let build_calls = Arc::new(AtomicUsize::new(0));
    let cleanup_calls = Arc::new(AtomicUsize::new(0));
    let (lifecycle, first_build_started, second_build_started, _cleanup_started, release_build) =
        activation_transition_gate(Arc::clone(&build_calls), cleanup_calls);
    runtime
        .register_module(
            ModuleDescriptor::new(module_name, "M2 activation join budget")
                .with_lifecycle(lifecycle),
        )
        .unwrap();

    let owner_module_name = module_name.to_owned();
    let first_runtime = runtime.clone();
    let first_activation = thread::spawn(move || first_runtime.activate_module(&owner_module_name));
    first_build_started
        .recv_timeout(Duration::from_secs(1))
        .expect("owner activation should enter build before joiners start");

    let joiner_count = 7;
    let (completion_sender, completion_receiver) = mpsc::channel();
    let joiners: Vec<_> = (0..joiner_count)
        .map(|_| {
            let joiner_runtime = runtime.clone();
            let joiner_module_name = module_name.to_owned();
            let completion_sender = completion_sender.clone();
            thread::spawn(move || {
                completion_sender
                    .send(joiner_runtime.activate_module(&joiner_module_name))
                    .unwrap();
            })
        })
        .collect();
    drop(completion_sender);

    wait_for_activation_joiners(&runtime, module_name, joiner_count, Duration::from_secs(1));
    let started = Instant::now();
    release_build.send(()).unwrap();

    let deadline = started + Duration::from_millis(750);
    for _ in 0..joiner_count {
        let remaining = deadline.saturating_duration_since(Instant::now());
        completion_receiver
            .recv_timeout(remaining)
            .expect("all activation joiners must report before the contention deadline")
            .unwrap();
    }
    let elapsed = started.elapsed();

    first_activation.join().unwrap().unwrap();
    for joiner in joiners {
        joiner.join().unwrap();
    }

    assert!(
        second_build_started.try_recv().is_err(),
        "joiners must attach to the in-flight activate instead of starting a second build"
    );
    assert_eq!(build_calls.load(Ordering::SeqCst), 1);

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get(module_name)
        .expect("activated module should remain registered");
    assert_eq!(module.lifecycle, LifecycleState::Running);
    elapsed
}

// 只轮询测试专用 waiter 计数，确认调用已登记入协调器，再释放回调闸门。
// 这避免把线程尚未被调度的时间混入竞争样本。
fn wait_for_activation_joiners(
    runtime: &CoreRuntime,
    module_name: &str,
    expected: usize,
    timeout: Duration,
) {
    let handle = runtime.handle();
    let deadline = Instant::now() + timeout;
    loop {
        let observed = handle
            .inner
            .lifecycle_coordinator
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .waiter_count(module_name, ModuleLifecycleCommand::Activate);
        if observed == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "expected {expected} activation joiners to enter the coordinator, observed {observed}"
        );
        thread::yield_now();
    }
}

// 直接通过 CoreHandle 协调入口固定 owner 与同命令等待者；在结果未发布时制造 Condvar 唤醒并完成另一模块。
// 随后验证原等待者仍消费 A 的失败回执且不执行自己的回调，完成后新重试可以另行获执行权。
#[test]
fn lifecycle_joiner_keeps_its_result_after_unrelated_and_spurious_wakes() {
    let runtime = CoreRuntime::new();
    let handle = runtime.handle();
    let (owner_started_sender, owner_started) = mpsc::sync_channel(1);
    let (release_owner, release_owner_receiver) = mpsc::sync_channel(1);
    let (owner_finished_sender, owner_finished) = mpsc::sync_channel(1);
    let owner_handle = handle.clone();
    let owner = thread::spawn(move || {
        let result = owner_handle.run_module_lifecycle_transition(
            "ReceiptIdentityA",
            ModuleLifecycleCommand::Activate,
            || {
                owner_started_sender.send(()).unwrap();
                release_owner_receiver.recv().unwrap();
                Err(CoreError::ChannelSend("original failure".to_owned()))
            },
        );
        owner_finished_sender.send(result).unwrap();
    });
    owner_started.recv_timeout(Duration::from_secs(1)).unwrap();

    let unexpected_operation_calls = Arc::new(AtomicUsize::new(0));
    let (joiner_finished_sender, joiner_finished) = mpsc::sync_channel(1);
    let joiner_calls = Arc::clone(&unexpected_operation_calls);
    let joiner_handle = handle.clone();
    let joiner = thread::spawn(move || {
        let result = joiner_handle.run_module_lifecycle_transition(
            "ReceiptIdentityA",
            ModuleLifecycleCommand::Activate,
            || {
                joiner_calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        );
        joiner_finished_sender.send(result).unwrap();
    });
    wait_for_activation_joiners(&runtime, "ReceiptIdentityA", 1, Duration::from_secs(1));

    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        let parked = handle
            .inner
            .lifecycle_coordinator
            .lock()
            .unwrap()
            .parked_waiter_count_for_test("ReceiptIdentityA");
        if parked == 1 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "joiner did not park on its receipt"
        );
        thread::yield_now();
    }

    {
        let coordinator = handle.inner.lifecycle_coordinator.lock().unwrap();
        coordinator.notify_transition_waiters_for_test("ReceiptIdentityA");
    }
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        let coordinator = handle.inner.lifecycle_coordinator.lock().unwrap();
        if coordinator.spurious_wake_count_for_test("ReceiptIdentityA") > 0 {
            assert_eq!(
                coordinator.waiter_count("ReceiptIdentityA", ModuleLifecycleCommand::Activate),
                1,
                "one caller must own one receipt despite a spurious wake"
            );
            break;
        }
        drop(coordinator);
        assert!(Instant::now() < deadline, "joiner did not observe the wake");
        thread::yield_now();
    }

    handle
        .run_module_lifecycle_transition(
            "ReceiptIdentityB",
            ModuleLifecycleCommand::Activate,
            || Ok(()),
        )
        .unwrap();
    assert_eq!(
        handle
            .inner
            .lifecycle_coordinator
            .lock()
            .unwrap()
            .waiter_count("ReceiptIdentityA", ModuleLifecycleCommand::Activate),
        1,
        "unrelated module completion must not register the caller again"
    );

    release_owner.send(()).unwrap();
    let expected = Err(CoreError::ChannelSend("original failure".to_owned()));
    assert_eq!(
        owner_finished.recv_timeout(Duration::from_secs(1)).unwrap(),
        expected
    );
    assert_eq!(
        joiner_finished
            .recv_timeout(Duration::from_secs(1))
            .unwrap(),
        expected
    );
    owner.join().unwrap();
    joiner.join().unwrap();
    assert_eq!(unexpected_operation_calls.load(Ordering::SeqCst), 0);

    let retry_calls = Arc::new(AtomicUsize::new(0));
    let retry_counter = Arc::clone(&retry_calls);
    assert_eq!(
        handle.run_module_lifecycle_transition(
            "ReceiptIdentityA",
            ModuleLifecycleCommand::Activate,
            || {
                retry_counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        ),
        Ok(())
    );
    assert_eq!(retry_calls.load(Ordering::SeqCst), 1);
}

// 先阻塞 Activate owner，再提交相反的 Deactivate 命令；它必须等 owner 终结后重新竞争执行权。
// owner 返回失败后，测试确认 Deactivate 回调才运行并成功。
#[test]
fn opposite_lifecycle_command_waits_for_owner_then_runs_its_own_operation() {
    let handle = CoreRuntime::new().handle();
    let (owner_started_sender, owner_started) = mpsc::sync_channel(1);
    let (release_owner, release_owner_receiver) = mpsc::sync_channel(1);
    let (owner_finished_sender, owner_finished) = mpsc::sync_channel(1);
    let owner_handle = handle.clone();
    let owner = thread::spawn(move || {
        let result = owner_handle.run_module_lifecycle_transition(
            "OppositeCommand",
            ModuleLifecycleCommand::Activate,
            || {
                owner_started_sender.send(()).unwrap();
                release_owner_receiver.recv().unwrap();
                Err(CoreError::ChannelSend("activate failed".to_owned()))
            },
        );
        owner_finished_sender.send(result).unwrap();
    });
    owner_started.recv_timeout(Duration::from_secs(1)).unwrap();

    let (opposite_started_sender, opposite_started) = mpsc::sync_channel(1);
    let (opposite_ran_sender, opposite_ran) = mpsc::sync_channel(1);
    let (opposite_finished_sender, opposite_finished) = mpsc::sync_channel(1);
    let opposite_handle = handle.clone();
    let opposite = thread::spawn(move || {
        opposite_started_sender.send(()).unwrap();
        let result = opposite_handle.run_module_lifecycle_transition(
            "OppositeCommand",
            ModuleLifecycleCommand::Deactivate,
            || {
                opposite_ran_sender.send(()).unwrap();
                Ok(())
            },
        );
        opposite_finished_sender.send(result).unwrap();
    });
    opposite_started
        .recv_timeout(Duration::from_secs(1))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        let observed = handle
            .inner
            .lifecycle_coordinator
            .lock()
            .unwrap()
            .opposite_waiter_count_for_test("OppositeCommand", ModuleLifecycleCommand::Activate);
        if observed == 1 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "opposite command did not join the turn wait"
        );
        thread::yield_now();
    }
    assert!(opposite_ran.try_recv().is_err());
    release_owner.send(()).unwrap();
    assert_eq!(
        owner_finished.recv_timeout(Duration::from_secs(1)).unwrap(),
        Err(CoreError::ChannelSend("activate failed".to_owned()))
    );
    assert_eq!(
        opposite_finished
            .recv_timeout(Duration::from_secs(1))
            .unwrap(),
        Ok(())
    );
    opposite_ran.recv_timeout(Duration::from_secs(1)).unwrap();
    owner.join().unwrap();
    opposite.join().unwrap();
}

// 直接调用 CoreHandle 协调器以观察 panic unwind：owner 守卫 Drop 必须给 joiner 发布类型化终态。
// joiner 回调若被错误执行会再次 panic；owner 退出后同命令重试应能获新执行权。
#[test]
fn panicked_lifecycle_owner_releases_joiner_and_later_retry() {
    let runtime = CoreRuntime::new();
    let handle = runtime.handle();
    let (owner_started_sender, owner_started) = mpsc::sync_channel(1);
    let (release_owner, release_owner_receiver) = mpsc::sync_channel(1);
    let (owner_finished_sender, owner_finished) = mpsc::sync_channel(1);
    let owner_handle = handle.clone();
    let owner = thread::spawn(move || {
        let panicked = panic::catch_unwind(AssertUnwindSafe(|| {
            owner_handle.run_module_lifecycle_transition(
                "PanickingOwner",
                ModuleLifecycleCommand::Activate,
                || {
                    owner_started_sender.send(()).unwrap();
                    release_owner_receiver.recv().unwrap();
                    panic!("owner callback panicked")
                },
            )
        }))
        .is_err();
        owner_finished_sender.send(panicked).unwrap();
    });
    owner_started.recv_timeout(Duration::from_secs(1)).unwrap();
    let (joiner_finished_sender, joiner_finished) = mpsc::sync_channel(1);
    let joiner_handle = handle.clone();
    let joiner = thread::spawn(move || {
        let result = joiner_handle.run_module_lifecycle_transition(
            "PanickingOwner",
            ModuleLifecycleCommand::Activate,
            || panic!("joiner must not own the in-flight transition"),
        );
        joiner_finished_sender.send(result).unwrap();
    });
    wait_for_activation_joiners(&runtime, "PanickingOwner", 1, Duration::from_secs(1));
    release_owner.send(()).unwrap();
    assert!(owner_finished.recv_timeout(Duration::from_secs(1)).unwrap());
    assert_eq!(
        joiner_finished
            .recv_timeout(Duration::from_secs(1))
            .unwrap(),
        Err(CoreError::ModuleLifecycleCallbackPanicked {
            module: "PanickingOwner".to_owned(),
            command: "activate",
        })
    );
    owner.join().unwrap();
    joiner.join().unwrap();
    assert_eq!(
        handle.run_module_lifecycle_transition(
            "PanickingOwner",
            ModuleLifecycleCommand::Activate,
            || Ok(()),
        ),
        Ok(())
    );
}

// 按 token 顺序直接驱动 LifecycleCoordinator：首轮完成后 owner 立刻开始新 epoch，旧 waiter 尚未 resolve。
// 陈旧 token 不得完成新轮；两个 waiter 各自读取创建时捕获的 completion 回执。
#[test]
fn late_lifecycle_retry_cannot_consume_a_previous_joiners_receipt() {
    use crate::core::runtime::state::ModuleLifecycleTransitionPermit;

    let handle = CoreRuntime::new().handle();
    let owner_id = thread::current().id();
    let joiner_id = thread::spawn(|| thread::current().id()).join().unwrap();
    let mut coordinator = handle.inner.lifecycle_coordinator.lock().unwrap();
    let ModuleLifecycleTransitionPermit::Owner(original_token) = coordinator
        .begin("LateRetry", ModuleLifecycleCommand::Activate, owner_id)
        .unwrap()
        .resolve()
    else {
        panic!("first caller must own the operation");
    };
    let joined = coordinator
        .begin("LateRetry", ModuleLifecycleCommand::Activate, joiner_id)
        .unwrap();
    let original_failure = Err(CoreError::ChannelSend("first operation failed".to_owned()));
    coordinator.complete(&original_token, original_failure.clone());

    let ModuleLifecycleTransitionPermit::Owner(retry_token) = coordinator
        .begin("LateRetry", ModuleLifecycleCommand::Activate, owner_id)
        .unwrap()
        .resolve()
    else {
        panic!("a late retry must own a fresh operation before the old joiner receives its result");
    };
    let retry_joined = coordinator
        .begin("LateRetry", ModuleLifecycleCommand::Activate, joiner_id)
        .unwrap();
    coordinator.complete(
        &original_token,
        Err(CoreError::ChannelSend("stale token".to_owned())),
    );
    coordinator.complete(&retry_token, Ok(()));
    drop(coordinator);

    assert!(matches!(
        joined.resolve(),
        ModuleLifecycleTransitionPermit::Completed(result) if result == original_failure
    ));
    assert!(matches!(
        retry_joined.resolve(),
        ModuleLifecycleTransitionPermit::Completed(Ok(()))
    ));
}
