//! Neutral channel aliases and receive helpers shared by runtime framework contracts.

use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};

pub type ChannelSender<T> = Sender<T>;
pub type ChannelReceiver<T> = Receiver<T>;

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
