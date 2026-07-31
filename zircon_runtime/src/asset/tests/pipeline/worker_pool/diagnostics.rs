use super::*;

#[test]
fn worker_pool_diagnostics_track_in_flight_and_failure_counts() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new());
    let request = missing_texture_request("diagnostic");

    let ticket = pool.request(request).unwrap();
    assert_eq!(pool.diagnostics().in_flight, 1);
    assert_eq!(pool.diagnostics().queue_peak, 1);
    release.send(()).unwrap();
    assert!(matches!(
        receive_completion(&ticket).as_ref(),
        CpuAssetPayload::Failure { .. }
    ));

    let diagnostics = pool.diagnostics();
    assert_eq!(
        diagnostics.thread_budget_source,
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
    assert_eq!(diagnostics.budgeted_threads, 1);
    assert_eq!(diagnostics.in_flight, 0);
    assert_eq!(diagnostics.completed, 1);
    assert_eq!(diagnostics.failed, 1);
    assert_eq!(diagnostics.queue_peak, 1);
    assert!(diagnostics.completion_bytes > 0);

    let mut store = DiagnosticStore::default();
    pool.record_diagnostics(&mut store, 7);
    let snapshot = store.snapshot();

    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_IN_FLIGHT_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_COMPLETED_DIAGNOSTIC),
        Some(1.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_FAILED_DIAGNOSTIC),
        Some(1.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_QUEUE_PEAK_DIAGNOSTIC),
        Some(1.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC),
        Some(1.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_COMPLETION_BYTES_DIAGNOSTIC),
        Some(diagnostics.completion_bytes as f64)
    );
}

#[test]
fn worker_pool_diagnostics_record_queue_age_clone_and_cancel_wall() {
    let io_pool = single_worker_io_pool();
    let release = occupy_io_pool(&io_pool);
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new().with_queue_depth(1));
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);

    let queued = pool.request(request.clone()).unwrap();
    std::thread::sleep(Duration::from_millis(10));
    release.send(()).unwrap();
    receive_completion(&queued);

    let cancellation_request = AssetRequest::Texture(TextureSource::BuiltinGrid);
    let cancellation = pool.request(cancellation_request.clone()).unwrap();
    assert!(pool.cancel(&cancellation_request));
    assert!(matches!(
        cancellation.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Cancelled)
    ));

    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.payload_clone_bytes, 0);
    assert!(diagnostics.queue_age_samples >= 1);
    assert!(diagnostics.queue_age_total >= Duration::from_millis(10));
    assert!(diagnostics.queue_age_max >= Duration::from_millis(10));
    assert_eq!(diagnostics.cancel_wall_samples, 1);
    assert!(diagnostics.cancel_wall_total > Duration::ZERO);

    let mut store = DiagnosticStore::default();
    pool.record_diagnostics(&mut store, 8);
    let snapshot = store.snapshot();
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_PAYLOAD_CLONE_BYTES_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_QUEUE_AGE_SAMPLES_DIAGNOSTIC),
        Some(diagnostics.queue_age_samples as f64)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_QUEUE_AGE_TOTAL_MS_DIAGNOSTIC),
        Some(diagnostics.queue_age_total.as_secs_f64() * 1_000.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_CANCEL_WALL_SAMPLES_DIAGNOSTIC),
        Some(1.0)
    );
    assert!(
        diagnostic_current(&snapshot, ASSET_WORKER_CANCEL_WALL_TOTAL_MS_DIAGNOSTIC)
            .is_some_and(|milliseconds| milliseconds > 0.0)
    );
}

#[test]
fn worker_pool_frame_sampler_records_per_job_completion_deltas() {
    let pool = AssetWorkerPool::new(single_worker_io_pool(), AssetWorkerPoolOptions::new());
    let mut sampler = AssetWorkerPoolFrameSampler::from_pool(&pool);
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);

    let first = pool.request(request.clone()).unwrap();
    let second = pool.request(request).unwrap();
    let failure = pool
        .request(missing_texture_request("frame-sampler"))
        .unwrap();
    receive_completion(&first);
    receive_completion(&second);
    receive_completion(&failure);

    let first_frame = sampler.sample(&pool);
    assert_eq!(
        first_frame.thread_budget_source,
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
    assert_eq!(first_frame.budgeted_threads, 1);
    assert_eq!(first_frame.in_flight, 0);
    assert_eq!(first_frame.completed_delta, 2);
    assert_eq!(first_frame.failed_delta, 1);

    let second_frame = sampler.sample(&pool);
    assert_eq!(second_frame.completed_delta, 0);
    assert_eq!(second_frame.failed_delta, 0);

    let mut store = DiagnosticStore::default();
    first_frame.record_diagnostics(&mut store, 11);
    sampler.record_diagnostics(&pool, &mut store, 12);
    let snapshot = store.snapshot();

    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_history(&snapshot, ASSET_WORKER_FRAME_COMPLETED_DIAGNOSTIC),
        vec![2.0, 0.0]
    );
    assert_eq!(
        diagnostic_history(&snapshot, ASSET_WORKER_FRAME_FAILED_DIAGNOSTIC),
        vec![1.0, 0.0]
    );
    assert_eq!(
        diagnostic_current(&snapshot, ASSET_WORKER_BUDGETED_THREADS_DIAGNOSTIC),
        Some(1.0)
    );
}
