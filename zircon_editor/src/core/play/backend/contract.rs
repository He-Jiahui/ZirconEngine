use std::sync::Arc;

use super::{
    PlayBackendPoll, PlayBackendRetireReport, PlayBackendStartFailure, PlayBackendStartReport,
    PlayBackendStopReport,
};
use crate::core::play::PlayStartRequest;

pub trait PlayBackend: Send + Sync {
    fn start(
        &self,
        request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure>;

    fn stop(&self) -> Result<PlayBackendStopReport, String>;

    fn retire(&self) -> Result<PlayBackendRetireReport, String>;

    fn poll(&self) -> Result<PlayBackendPoll, String>;
}

pub type SharedPlayBackend = Arc<dyn PlayBackend>;
