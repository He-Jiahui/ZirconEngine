use std::sync::mpsc;
use std::time::Duration;

use super::{TaskPool, TaskPoolDescriptor};

#[test]
fn acquired_submission_survives_external_admission_close() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let submission = pool
        .try_acquire_submission()
        .expect("open pool should issue a submission authority");
    pool.close_admission();
    let (sender, receiver) = mpsc::sync_channel(1);

    submission.spawn(move || sender.send(()).expect("submission result"));

    receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("work accepted before close must still reach the worker");
}
