use super::*;

#[test]
fn worker_pool_completes_builtin_texture_requests_on_the_runtime_io_pool() {
    let io_pool = single_worker_io_pool();
    let pool = AssetWorkerPool::new(io_pool.clone(), AssetWorkerPoolOptions::new());

    assert_eq!(pool.task_pool().kind(), TaskPoolKind::Io);
    assert!(pool.task_pool().shares_execution_owner_with(&io_pool));

    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .unwrap();

    match receive_completion(&ticket).as_ref() {
        CpuAssetPayload::Texture(texture) => {
            assert_eq!(texture.source, TextureSource::BuiltinChecker);
            assert_eq!(
                texture.rgba.len(),
                texture.width as usize * texture.height as usize * 4
            );
        }
        other => panic!("unexpected payload: {other:?}"),
    }
}

#[test]
fn worker_pool_default_budgets_are_hard_limits() {
    let options = AssetWorkerPoolOptions::new();
    let pool = AssetWorkerPool::new(single_worker_io_pool(), options.clone());

    assert_eq!(pool.options(), &options);
    assert_eq!(pool.options().queue_depth, Some(2));
    assert!(pool.options().waiter_capacity > 0);
    assert!(pool.options().completion_entry_capacity > 0);
    assert!(pool.options().completion_byte_capacity > 0);
    assert!(pool.options().request_max_age > Duration::ZERO);
    assert!(pool.options().completion_max_age > Duration::ZERO);
    assert_eq!(
        pool.diagnostics().thread_budget_source,
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
}

#[test]
fn worker_pool_budget_source_tracks_one_eight_and_sixty_four_io_workers() {
    for worker_count in [1, 8, 64] {
        let pool = AssetWorkerPool::new(
            TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(worker_count)),
            AssetWorkerPoolOptions::new(),
        );

        let diagnostics = pool.diagnostics();
        assert_eq!(
            diagnostics.thread_budget_source,
            AssetWorkerThreadBudgetSource::TaskPoolIo
        );
        assert_eq!(diagnostics.budgeted_threads, worker_count);
    }
}

#[test]
fn project_asset_manager_uses_the_injected_runtime_io_pool() {
    let io_pool = single_worker_io_pool();
    let manager = ProjectAssetManager::new(io_pool.clone());
    let (pool, mut sampler) = manager.spawn_worker_pool_with_frame_sampler();

    assert!(
        manager
            .worker_task_pool()
            .shares_execution_owner_with(&io_pool)
    );
    assert!(pool.task_pool().shares_execution_owner_with(&io_pool));
    assert_eq!(pool.options().queue_depth, Some(2));
    assert_eq!(manager.default_worker_count(), io_pool.parallelism());
    assert_eq!(
        manager.default_worker_budget_source(),
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );

    let frame = sampler.sample(&pool);
    assert_eq!(
        frame.thread_budget_source,
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
    assert_eq!(frame.budgeted_threads, 1);
    assert_eq!(frame.in_flight, 0);
    assert_eq!(frame.completed_delta, 0);
    assert_eq!(frame.failed_delta, 0);
}

#[test]
fn project_asset_manager_defaults_share_the_process_io_pool() {
    let first = ProjectAssetManager::default();
    let second = ProjectAssetManager::default();

    assert!(
        first
            .worker_task_pool()
            .shares_execution_owner_with(second.worker_task_pool())
    );
    assert_eq!(
        first.default_worker_budget_source(),
        AssetWorkerThreadBudgetSource::TaskPoolIo
    );
}
