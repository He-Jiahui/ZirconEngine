use super::*;

#[test]
fn net_driver_has_a_bounded_worker_set_even_on_a_large_host() {
    let budget = EngineThreadBudget::for_parallelism(128);
    let state = NetRuntimeState::new(NetRuntimeMode::Client, budget);
    assert_eq!(
        state.runtime.metrics().num_workers(),
        budget.net_async_worker_threads()
    );
    state.shutdown_worker_for_tests();
}
