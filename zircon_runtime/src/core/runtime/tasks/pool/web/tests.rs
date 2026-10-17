use super::*;
use std::sync::atomic::AtomicUsize;

fn pool() -> TaskPool {
    TaskPool::new(TaskPoolDescriptor::compute().with_thread_name("browser-test"))
}

#[test]
fn bounded_admission_serial_execution_and_owner_shutdown() {
    let pool = pool();
    let handle = pool.clone();
    let completed = Arc::new(AtomicUsize::new(0));
    for _ in 0..TASK_CAPACITY {
        let completed = completed.clone();
        assert!(handle
            .try_spawn(move || {
                completed.fetch_add(1, Ordering::Relaxed);
            })
            .is_ok());
    }
    assert!(handle
        .try_spawn(|| panic!("refused task executed"))
        .is_err());
    assert_eq!(completed.load(Ordering::Relaxed), 0);
    assert!(run_one(&pool.inner.upgrade().unwrap()));
    assert_eq!(completed.load(Ordering::Relaxed), 1);
    drop(pool);
    assert!(handle.try_spawn(|| panic!("closed task executed")).is_err());
    for _ in 1..TASK_CAPACITY {
        assert!(matches!(
            assist_current_thread_once(),
            Some(TaskPoolYield::Executed)
        ));
    }
    assert_eq!(completed.load(Ordering::Relaxed), TASK_CAPACITY);
    assert!(handle.shutdown_census().termination_signalled);
    assert_eq!(handle.shutdown_census().expected_worker_count, 0);
}

#[test]
fn serial_scope_does_not_retain_borrowed_work() {
    let pool = pool();
    let mut value = 0;
    pool.in_place_scope(|scope| scope.spawn(|_| value = 7));
    assert_eq!(value, 7);
    assert_eq!(pool.join(|| 2, || 3), (2, 3));
}
