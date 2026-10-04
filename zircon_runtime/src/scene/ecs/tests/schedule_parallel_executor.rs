use std::panic::{catch_unwind, AssertUnwindSafe};

use super::*;

#[test]
fn cloned_task_registry_shares_frozen_task_map_until_mutated() {
    let mut registry = ScheduleParallelTaskRegistry::<()>::new();
    registry.register("system.alpha", || Ok(()));

    let snapshot = registry.clone();
    assert!(Arc::ptr_eq(&registry.tasks, &snapshot.tasks));

    registry.register("system.beta", || Ok(()));

    assert!(!Arc::ptr_eq(&registry.tasks, &snapshot.tasks));
    assert!(snapshot.contains("system.alpha"));
    assert!(!snapshot.contains("system.beta"));
    assert!(registry.contains("system.beta"));
}

#[test]
fn schedule_parallel_executor_batch_result_slot_recovers_poisoned_lock() {
    let slot: Mutex<Option<ScheduleParallelBatchResult<&'static str>>> = Mutex::new(Some(Ok(())));

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = slot.lock().unwrap();
        panic!("poison schedule parallel executor batch result slot");
    }));

    let recovered = lock_batch_result::<&'static str>(&slot)
        .take()
        .expect("batch result should remain available after poison recovery");
    assert_eq!(recovered, Ok(()));

    *lock_batch_result(&slot) = Some(Err(ScheduleParallelExecutorError::MissingTask {
        system_id: "missing.task".to_string(),
    }));
    let recovered = lock_batch_result(&slot)
        .take()
        .expect("missing-task result should remain available");
    assert_eq!(
        recovered,
        Err(ScheduleParallelExecutorError::MissingTask {
            system_id: "missing.task".to_string()
        })
    );
}
