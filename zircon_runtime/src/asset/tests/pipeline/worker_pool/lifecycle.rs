use super::*;

#[test]
fn completion_entry_budget_rejects_unharvested_payload_without_blocking_worker() {
    let pool = AssetWorkerPool::new(
        single_worker_io_pool(),
        AssetWorkerPoolOptions::new().with_completion_entry_capacity(0),
    );
    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(2)),
        Err(AssetWorkerCompletionError::Rejected)
    ));
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.rejected, 1);
    assert_eq!(diagnostics.completion_rejected, 1);
    assert_eq!(diagnostics.completed, 0);
    assert_eq!(diagnostics.completion_entries, 0);
}

#[test]
fn completion_byte_budget_rejects_the_retained_payload_allocation() {
    let pool = AssetWorkerPool::new(
        single_worker_io_pool(),
        AssetWorkerPoolOptions::new().with_completion_byte_capacity(1),
    );
    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(2)),
        Err(AssetWorkerCompletionError::Rejected)
    ));
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.completion_entries, 0);
    assert_eq!(diagnostics.completion_bytes, 0);
    assert_eq!(diagnostics.completion_rejected, 1);
}

#[test]
fn completion_deadline_transition_reuses_a_full_timer_slot() {
    const TIMER_CAPACITY: usize = 512;

    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let timer = TaskTimer::new(TIMER_CAPACITY).expect("test timer should start");
    let pool = AssetWorkerPool::with_expiry_timer(
        io_pool,
        AssetWorkerPoolOptions::new()
            .with_queue_depth(TIMER_CAPACITY - 1)
            .with_completion_entry_capacity(TIMER_CAPACITY),
        timer,
    );
    let mut tickets = Vec::with_capacity(TIMER_CAPACITY);
    for index in 0..TIMER_CAPACITY {
        tickets.push(
            pool.request(missing_texture_request(&format!("timer-slot-{index}")))
                .expect(
                    "each bounded queued request should occupy one request-deadline timer slot",
                ),
        );
    }

    release.send(()).unwrap();
    assert!(matches!(
        tickets[0].wait_timeout(Duration::from_secs(2)),
        Ok(payload) if matches!(&*payload, CpuAssetPayload::Failure { .. })
    ));
    assert_eq!(pool.diagnostics().completion_rejected, 0);
    drop(pool);
}

#[test]
fn completion_age_expiry_is_observable_and_removes_unharvested_payload() {
    let pool = AssetWorkerPool::new(
        single_worker_io_pool(),
        AssetWorkerPoolOptions::new().with_completion_max_age(Duration::from_millis(10)),
    );
    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    wait_for_completed(&pool);
    let expiry_started = Instant::now();
    wait_for_expiry(&pool, Duration::from_millis(250));
    assert!(
        expiry_started.elapsed() < Duration::from_millis(250),
        "the registered completion deadline should not fall back to a frame-sampler sweep"
    );
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.in_flight, 0);
    assert_eq!(diagnostics.completion_entries, 0);
    assert_eq!(diagnostics.completion_bytes, 0);
    assert_eq!(diagnostics.expired, 1);
    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(2)),
        Err(AssetWorkerCompletionError::Expired)
    ));
}

#[test]
fn completion_deadline_replaces_the_pending_request_deadline() {
    let pool = AssetWorkerPool::new(
        single_worker_io_pool(),
        AssetWorkerPoolOptions::new()
            .with_request_max_age(Duration::from_millis(10))
            .with_completion_max_age(Duration::from_millis(250)),
    );
    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    wait_for_completed(&pool);
    std::thread::sleep(Duration::from_millis(25));
    assert!(ticket
        .try_result()
        .expect("a ready completion must outlive its superseded request deadline")
        .is_some());
}

#[test]
fn request_age_expiry_wakes_a_queued_ticket_as_expired_not_timed_out() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(
        io_pool,
        AssetWorkerPoolOptions::new()
            .with_queue_depth(0)
            .with_request_max_age(Duration::from_millis(10)),
    );
    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    wait_for_expiry(&pool, Duration::from_millis(250));
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.in_flight, 0);
    assert_eq!(diagnostics.expired, 1);
    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Expired)
    ));
    release.send(()).unwrap();
}

#[test]
fn cancelling_a_queued_request_wakes_tickets_without_waiting_for_the_worker() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new());
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let ticket = pool.request(request.clone()).unwrap();

    assert!(pool.cancel(&request));
    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
    assert_eq!(pool.diagnostics().cancelled, 1);
    release.send(()).unwrap();
}

#[test]
fn dropping_worker_pool_cancels_pending_jobs_without_synchronous_wait() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new());
    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();
    let (drop_started_tx, drop_started_rx) = bounded::<()>(1);
    let (dropped_tx, dropped_rx) = bounded::<()>(1);

    let drop_thread = std::thread::spawn(move || {
        drop_started_tx.send(()).unwrap();
        drop(pool);
        dropped_tx.send(()).unwrap();
    });

    drop_started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("drop thread should start");
    dropped_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("drop must not wait for a queued IO task");
    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
    release.send(()).unwrap();
    drop_thread.join().unwrap();
}

#[test]
fn dropping_worker_pool_preserves_cancelled_ticket_after_armed_deadline() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(
        io_pool,
        AssetWorkerPoolOptions::new()
            .with_queue_depth(0)
            .with_request_max_age(Duration::from_millis(10)),
    );
    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    drop(pool);
    std::thread::sleep(Duration::from_millis(25));
    assert!(matches!(
        ticket.try_result(),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
    release.send(()).unwrap();
}

#[test]
fn dropping_worker_pool_on_its_io_worker_cancels_its_queued_ticket() {
    let io_pool = single_worker_io_pool();
    let pool = AssetWorkerPool::new(io_pool.clone(), AssetWorkerPoolOptions::new());
    let (ticket_tx, ticket_rx) = bounded::<AssetWorkerCompletionTicket>(1);
    let (dropped_tx, dropped_rx) = bounded::<()>(1);

    io_pool.spawn(move || {
        let ticket = pool
            .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
            .unwrap();
        ticket_tx.send(ticket).unwrap();
        drop(pool);
        dropped_tx.send(()).unwrap();
    });

    dropped_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("dropping on the only IO worker must return before the queued request runs");
    let ticket = ticket_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
}
