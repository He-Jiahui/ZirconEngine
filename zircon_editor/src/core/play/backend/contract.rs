use std::sync::Arc;

use super::{PlayBackendPoll, PlayBackendStartReport, PlayBackendStopReport};
use crate::core::play::PlayStartRequest;

pub trait PlayBackend: Send + Sync {
    fn start(&self, request: &PlayStartRequest) -> Result<PlayBackendStartReport, String>;

    fn stop(&self) -> Result<PlayBackendStopReport, String>;

    fn poll(&self) -> Result<PlayBackendPoll, String>;
}

pub type SharedPlayBackend = Arc<dyn PlayBackend>;
