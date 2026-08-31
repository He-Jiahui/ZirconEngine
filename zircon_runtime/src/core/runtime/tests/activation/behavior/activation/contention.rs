use super::*;

#[test]
fn concurrent_activation_joiners_share_one_build_within_contention_budget() {
    let elapsed = activation_join_sample("ConcurrentActivationBudgetModule");

    assert!(
        elapsed <= Duration::from_millis(750),
        "seven already-waiting activation joiners must finish within 750ms, took {elapsed:?}"
    );
}

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
