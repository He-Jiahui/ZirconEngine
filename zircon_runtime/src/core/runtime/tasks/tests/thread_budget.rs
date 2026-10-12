use super::*;

#[test]
fn engine_thread_budget_conserves_the_sum_including_plugin_reservations() {
    for hardware in 0..=256 {
        let budget = EngineThreadBudget::for_parallelism(hardware);
        let pools = budget.task_pool_options().resolve_thread_counts(hardware);
        assert_eq!(pools.total_threads, budget.task_worker_threads());
        assert_eq!(budget.planned_threads(), hardware.max(7));
        assert!(budget.physics_worker_threads() <= 4);
        assert_eq!(budget.net_async_worker_threads(), 1);
        assert_eq!(budget.net_blocking_threads(), 1);
    }
}

#[test]
fn low_parallelism_does_not_ask_jolt_to_auto_create_workers() {
    for hardware in 0..=7 {
        assert_eq!(
            EngineThreadBudget::for_parallelism(hardware).physics_worker_threads(),
            0
        );
    }
}
