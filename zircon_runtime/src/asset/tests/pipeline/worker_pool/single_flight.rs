use super::*;

#[test]
fn worker_pool_bounded_queue_rejects_overflow_with_explicit_error() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new().with_queue_depth(0));

    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();
    let error = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinGrid))
        .expect_err("a second unique request must exceed one IO worker with zero queue depth");

    match error {
        CoreError::ChannelSend(message) => assert!(
            message.contains("asset request queue full"),
            "unexpected channel error: {message}"
        ),
        other => panic!("unexpected error variant: {other}"),
    }
    assert_eq!(pool.diagnostics().in_flight, 1);
    assert_eq!(pool.diagnostics().queue_peak, 1);
    assert_eq!(pool.diagnostics().rejected, 1);
    assert_eq!(pool.diagnostics().queue_rejected, 1);

    release.send(()).unwrap();
    receive_completion(&ticket);
}

#[test]
fn cancelled_queued_work_keeps_admission_charged_until_its_closure_exits() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new().with_queue_depth(0));
    let cancelled_request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let replacement_request = AssetRequest::Texture(TextureSource::BuiltinGrid);
    let cancelled = pool.request(cancelled_request.clone()).unwrap();

    assert!(pool.cancel(&cancelled_request));
    assert!(matches!(
        cancelled.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
    let error = pool
        .request(replacement_request.clone())
        .expect_err("a cancelled closure must still occupy bounded executor admission");
    assert!(error.to_string().contains("asset request queue full"));

    release.send(()).unwrap();
    let replacement = wait_for_admission(&pool, replacement_request);
    assert!(matches!(
        receive_completion(&replacement).as_ref(),
        CpuAssetPayload::Texture(texture) if texture.source == TextureSource::BuiltinGrid
    ));
}

#[test]
fn cancelled_request_does_not_orphan_a_same_key_replacement_ticket() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new().with_queue_depth(1));
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let cancelled = pool.request(request.clone()).unwrap();

    assert!(pool.cancel(&request));
    let replacement = pool
        .request(request)
        .expect("the second admission is within the distinct bounded closure capacity");
    release.send(()).unwrap();

    assert!(matches!(
        cancelled.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
    assert!(matches!(
        receive_completion(&replacement).as_ref(),
        CpuAssetPayload::Texture(texture) if texture.source == TextureSource::BuiltinChecker
    ));
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.cancelled, 1);
    assert_eq!(diagnostics.completed, 1);
}

#[test]
fn concurrent_requests_for_same_asset_share_one_immutable_payload_owner() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new().with_queue_depth(0));
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);

    let first = pool.request(request.clone()).unwrap();
    let second = pool.request(request).unwrap();
    assert_eq!(pool.diagnostics().in_flight, 1);
    assert_eq!(pool.diagnostics().in_flight_waiters, 2);
    assert_eq!(pool.diagnostics().merged, 1);
    let overflow = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinGrid))
        .expect_err("a different request must still see the full bounded queue");
    assert!(
        overflow.to_string().contains("asset request queue full"),
        "unexpected error: {overflow}"
    );

    release.send(()).unwrap();
    let first_payload = receive_completion(&first);
    let second_payload = receive_completion(&second);
    assert!(Arc::ptr_eq(&first_payload, &second_payload));
    assert!(
        matches!(first_payload.as_ref(), CpuAssetPayload::Texture(texture) if texture.source == TextureSource::BuiltinChecker)
    );
    assert_eq!(pool.diagnostics().completed, 1);
    assert_eq!(pool.diagnostics().completion_entries, 1);
}

#[test]
fn completed_cache_reads_do_not_consume_live_waiter_budget() {
    let pool = AssetWorkerPool::new(
        single_worker_io_pool(),
        AssetWorkerPoolOptions::new()
            .with_queue_depth(0)
            .with_waiter_capacity(1),
    );
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let first = pool.request(request.clone()).unwrap();
    let first_payload = receive_completion(&first);

    for _ in 0..3 {
        let cached = pool
            .request(request.clone())
            .expect("a retained completion must not consume a live waiter slot");
        let cached_payload = receive_completion(&cached);
        assert!(Arc::ptr_eq(&first_payload, &cached_payload));
    }

    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.waiter_rejected, 0);
    assert_eq!(diagnostics.completion_entries, 1);
}

#[test]
fn duplicate_waiters_are_rejected_at_the_shared_observer_budget() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(
        io_pool,
        AssetWorkerPoolOptions::new()
            .with_queue_depth(0)
            .with_waiter_capacity(1),
    );
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let ticket = pool.request(request.clone()).unwrap();

    let error = pool
        .request(request)
        .expect_err("a duplicate request must not bypass the observer budget");
    assert!(error.to_string().contains("observer budget full"));
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.merged, 0);
    assert_eq!(diagnostics.waiter_rejected, 1);
    assert_eq!(diagnostics.rejected, 1);

    release.send(()).unwrap();
    receive_completion(&ticket);
}

#[test]
fn dropped_ticket_releases_its_live_observer_slot() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(
        io_pool,
        AssetWorkerPoolOptions::new()
            .with_queue_depth(0)
            .with_waiter_capacity(2),
    );
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let first = pool.request(request.clone()).unwrap();
    let dropped = pool.request(request.clone()).unwrap();

    drop(dropped);
    let replacement = pool
        .request(request)
        .expect("dropping a ticket must release its live observer slot");
    assert_eq!(pool.diagnostics().in_flight_waiters, 2);

    release.send(()).unwrap();
    receive_completion(&first);
    receive_completion(&replacement);
}

#[test]
fn duplicate_waiter_budget_remains_hard_at_one_one_thousand_and_one_hundred_thousand() {
    for capacity in [1, 1_000, 100_000] {
        let io_pool = single_worker_io_pool();
        let release = occupy_io_pool(&io_pool);
        let pool = AssetWorkerPool::new(
            io_pool,
            AssetWorkerPoolOptions::new()
                .with_queue_depth(0)
                .with_waiter_capacity(capacity),
        );
        let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
        let first = pool.request(request.clone()).unwrap();
        let mut live_tickets = Vec::with_capacity(capacity.saturating_sub(1));

        for _ in 1..capacity {
            live_tickets.push(
                pool.request(request.clone())
                    .expect("duplicate ticket must remain below the shared observer hard limit"),
            );
        }
        let overflow = pool
            .request(request)
            .expect_err("the observer hard limit must reject one more ticket");
        assert!(overflow.to_string().contains("observer budget full"));
        assert_eq!(pool.diagnostics().in_flight_waiters, capacity);

        release.send(()).unwrap();
        receive_completion(&first);
        drop(live_tickets);
    }
}

#[test]
fn worker_pool_source_uses_shared_ticket_results_not_a_completion_channel() {
    let worker_pool_sources = [
        include_str!("../../../pipeline/worker_pool.rs"),
        include_str!("../../../pipeline/worker_pool/completion.rs"),
        include_str!("../../../pipeline/worker_pool/diagnostics.rs"),
        include_str!("../../../pipeline/worker_pool/options.rs"),
        include_str!("../../../pipeline/worker_pool/payload.rs"),
    ];
    let manager_source =
        include_str!("../../../pipeline/manager/project_asset_manager/construction.rs");

    assert!(worker_pool_sources[0].contains("AssetWorkerCompletionTicket"));
    assert!(worker_pool_sources[0].contains("Arc<CpuAssetPayload>"));
    assert!(worker_pool_sources[0].contains("completion_entry_capacity"));
    assert!(worker_pool_sources[0].contains("completion_byte_capacity"));
    for source in worker_pool_sources {
        assert!(!source.contains("completion_receiver"));
        assert!(!source.contains("unbounded"));
        assert!(!source.contains("payload.clone()"));
    }
    assert!(manager_source.contains("spawn_worker_pool_with_frame_sampler"));
    assert!(!manager_source.contains("pub fn spawn_worker_pool(&self)"));
}
