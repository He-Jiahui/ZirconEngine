//! Shared runtime primitives for channels, threads, and frame timing.

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use thiserror::Error;

pub type ChannelSender<T> = Sender<T>;
pub type ChannelReceiver<T> = Receiver<T>;

#[derive(Debug, Error)]
pub enum ZirconError {
    #[error("channel send failed: {0}")]
    ChannelSend(String),
    #[error("thread spawn failed: {0}")]
    ThreadSpawn(String),
}

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

#[derive(Debug, Clone)]
pub struct FrameClock {
    last_tick: Instant,
}

impl Default for FrameClock {
    fn default() -> Self {
        Self {
            last_tick: Instant::now(),
        }
    }
}

impl FrameClock {
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;

    #[test]
    fn recv_latest_keeps_last_message() {
        let (sender, receiver) = unbounded();
        sender.send(1).unwrap();
        sender.send(2).unwrap();

        assert_eq!(recv_latest(&receiver), Some(2));
        assert_eq!(recv_latest::<i32>(&receiver), None);
    }
}
