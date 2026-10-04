use super::*;
use std::sync::{Arc, Condvar, Mutex};
use tokio::{
    sync::Notify,
    time::{Duration, Instant},
};

struct ReleaseOnDrop(Arc<(Mutex<bool>, Condvar)>);

impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        let (released, changed) = &*self.0;
        *released.lock().unwrap() = true;
        changed.notify_all();
    }
}

#[tokio::test]
async fn cancelled_waiter_leaves_owned_job_for_shutdown_retry_and_finished_census() {
    let database = Database::memory();
    let entered = Arc::new(Notify::new());
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let release_on_drop = ReleaseOnDrop(release.clone());
    let task_database = database.clone();
    let task_entered = entered.clone();
    let task_release = release.clone();
    let caller = tokio::spawn(async move {
        task_database
            .execute(move |_| {
                task_entered.notify_one();
                let (released, changed) = &*task_release;
                let mut released = released.lock().unwrap();
                while !*released {
                    released = changed.wait(released).unwrap();
                }
                Ok(())
            })
            .await
    });

    tokio::time::timeout(Duration::from_secs(2), entered.notified())
        .await
        .unwrap();
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());

    assert!(matches!(
        database.shutdown(Instant::now()).await,
        Err(ServiceError::OutcomeUnknown)
    ));
    assert_eq!(
        database.job_census(),
        JobCensus {
            active: 1,
            finished: 0,
            failed: 0,
        }
    );
    assert!(matches!(
        database.execute(|_| Ok(())).await,
        Err(ServiceError::Capacity)
    ));

    drop(release_on_drop);
    let census = database
        .shutdown(Instant::now() + Duration::from_secs(2))
        .await
        .unwrap();
    assert_eq!(
        census,
        JobCensus {
            active: 0,
            finished: 1,
            failed: 0,
        }
    );
    assert_eq!(
        database
            .shutdown(Instant::now() + Duration::from_secs(2))
            .await
            .unwrap(),
        census
    );
}
