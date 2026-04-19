//! Helpers for crossbeam channels and named threads.

use crossbeam_channel::{Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::core::error::ZirconError;

pub fn spawn_named_thread<F, T>(
    name: impl Into<String>,
    task: F,
) -> Result<JoinHandle<T>, ZirconError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let name = name.into();
    thread::Builder::new()
        .name(name.clone())
        .spawn(task)
        .map_err(|error| ZirconError::ThreadSpawn(format!("{name}: {error}")))
}

pub fn recv_latest<T>(receiver: &Receiver<T>) -> Option<T> {
    let mut latest = None;
    while let Ok(value) = receiver.try_recv() {
        latest = Some(value);
    }
    latest
}

pub fn wait_for<T>(receiver: &Receiver<T>, timeout: Duration) -> Result<T, RecvTimeoutError> {
    receiver.recv_timeout(timeout)
}
