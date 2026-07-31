use super::{PlayBackend, PlayBackendPoll, PlayBackendStartReport, PlayBackendStopReport};
use crate::core::play::PlayStartRequest;

#[derive(Debug, Default)]
pub struct NoopPlayBackend;

impl PlayBackend for NoopPlayBackend {
    fn start(&self, _request: &PlayStartRequest) -> Result<PlayBackendStartReport, String> {
        Ok(PlayBackendStartReport::default())
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        Ok(PlayBackendStopReport::default())
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        Ok(PlayBackendPoll::Running {
            diagnostics: Vec::new(),
        })
    }
}
