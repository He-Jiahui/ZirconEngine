use super::*;
use crate::service::storage::Database;
use std::sync::{Arc, Condvar, Mutex};
use tokio::time::{sleep, Duration};

struct ReleaseOnDrop(Arc<(Mutex<bool>, Condvar)>);

impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        let (released, changed) = &*self.0;
        *released.lock().unwrap() = true;
        changed.notify_all();
    }
}

#[tokio::test]
async fn startup_completes_normally_without_signal() {
    let (mut supervisor, _trigger) = StartupSupervisor::injected(Duration::from_secs(1));
    let result = supervisor
        .wait(tokio::spawn(async { Ok::<_, ServiceError>(7_u8) }))
        .await
        .unwrap();
    assert_eq!(result, 7);
}

#[tokio::test]
async fn startup_signal_waits_for_blocking_owner_until_it_finishes() {
    let (mut supervisor, trigger) = StartupSupervisor::injected(Duration::from_secs(1));
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let release_on_drop = ReleaseOnDrop(gate.clone());
    let worker_gate = gate.clone();
    let operation = tokio::task::spawn_blocking(move || {
        let (done, changed) = &*worker_gate;
        let mut done = done.lock().unwrap();
        while !*done {
            done = changed.wait(done).unwrap();
        }
        Ok::<_, ServiceError>(())
    });
    let owner = tokio::spawn(async move { supervisor.wait(operation).await });
    trigger.request();
    sleep(Duration::from_millis(10)).await;
    assert!(!owner.is_finished());
    drop(release_on_drop);
    assert!(matches!(
        owner.await.unwrap(),
        Err(ServiceError::OutcomeUnknown)
    ));
}

#[tokio::test]
async fn blocked_startup_returns_unknown_at_shutdown_deadline() {
    let (mut supervisor, trigger) = StartupSupervisor::injected(Duration::from_millis(10));
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let release_on_drop = ReleaseOnDrop(gate.clone());
    let worker_gate = gate.clone();
    let operation = tokio::task::spawn_blocking(move || {
        let (done, changed) = &*worker_gate;
        let mut done = done.lock().unwrap();
        while !*done {
            done = changed.wait(done).unwrap();
        }
        Ok::<_, ServiceError>(())
    });
    trigger.request();
    assert!(matches!(
        supervisor.wait(operation).await,
        Err(ServiceError::OutcomeUnknown)
    ));
    drop(release_on_drop);
}

#[tokio::test]
async fn shutdown_before_storage_registration_closes_later_database_admission() {
    let (supervisor, trigger) = StartupSupervisor::injected(Duration::from_secs(1));
    trigger.request();
    let database = Database::memory();
    supervisor.register_storage(database.clone());
    assert!(matches!(
        database.execute(|_| Ok(())).await,
        Err(ServiceError::Capacity)
    ));
}
