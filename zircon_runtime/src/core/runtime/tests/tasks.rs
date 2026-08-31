use crate::core::framework::render::{
    build_source_cubemap_from_equirect, source_cubemap_mip_count, SourceCubemapMipChain,
    SourceCubemapPrefilterQuality,
};
use crate::core::runtime::tasks::TaskPool;
use crate::core::runtime::tasks::TaskPoolDescriptor;

#[test]
fn task_pool_in_place_scope_keeps_the_scope_body_on_the_caller() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let caller = std::thread::current().id();
    let (worker_tx, worker_rx) = std::sync::mpsc::channel();

    pool.in_place_scope(|scope| {
        assert_eq!(std::thread::current().id(), caller);
        scope.spawn(move |_| {
            worker_tx
                .send(std::thread::current().id())
                .expect("worker identity should reach the caller");
        });
    });

    assert_ne!(worker_rx.recv().expect("scoped worker should run"), caller);
}

#[test]
fn source_cubemap_explicit_executor_entry_preserves_output_contract() {
    let serial = build_source_cubemap_from_equirect(4, |u, v| [u, v, u + v, 1.0]);
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let pooled = SourceCubemapMipChain::from_equirect_with_parallel_executor(4, &pool, |u, v| {
        [u, v, u + v, 1.0]
    });

    assert_eq!(pooled, serial);
}

#[test]
fn source_cubemap_parallel_executor_preserves_explicit_pmrem_layout() {
    let serial = SourceCubemapMipChain::from_equirect_with_pmrem_layout(
        64,
        32,
        source_cubemap_mip_count(32),
        SourceCubemapPrefilterQuality::Normal,
        |u, v| [u, v, u * v, 1.0],
    );
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let parallel = SourceCubemapMipChain::from_equirect_with_pmrem_layout_and_parallel_executor(
        64,
        32,
        source_cubemap_mip_count(32),
        SourceCubemapPrefilterQuality::Normal,
        &pool,
        |u, v| [u, v, u * v, 1.0],
    );

    assert_eq!(parallel, serial);
}
