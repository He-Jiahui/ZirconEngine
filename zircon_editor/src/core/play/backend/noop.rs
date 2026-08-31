use super::{
    PlayBackend, PlayBackendPoll, PlayBackendRetireReport, PlayBackendStartFailure,
    PlayBackendStartReport, PlayBackendStopReport,
};
use crate::core::play::PlayStartRequest;

#[cfg(test)]
use crate::core::gateway::SharedEditorRuntimeGateway;

#[derive(Debug, Default)]
pub struct NoopPlayBackend;

#[cfg(test)]
pub(crate) struct TestAttachablePlayBackend {
    gateway: SharedEditorRuntimeGateway,
}

#[cfg(test)]
impl TestAttachablePlayBackend {
    pub(crate) fn new(gateway: SharedEditorRuntimeGateway) -> Self {
        Self { gateway }
    }
}

impl PlayBackend for NoopPlayBackend {
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        Ok(PlayBackendStartReport::default())
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        Ok(PlayBackendStopReport::default())
    }

    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        Ok(PlayBackendRetireReport::default())
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        Ok(PlayBackendPoll::Running {
            diagnostics: Vec::new(),
        })
    }
}

#[cfg(test)]
impl PlayBackend for TestAttachablePlayBackend {
    fn start(
        &self,
        _request: &PlayStartRequest,
    ) -> Result<PlayBackendStartReport, PlayBackendStartFailure> {
        Ok(PlayBackendStartReport::with_gateway(
            Vec::new(),
            self.gateway.clone(),
        ))
    }

    fn stop(&self) -> Result<PlayBackendStopReport, String> {
        Ok(PlayBackendStopReport::default())
    }

    fn retire(&self) -> Result<PlayBackendRetireReport, String> {
        Ok(PlayBackendRetireReport::default())
    }

    fn poll(&self) -> Result<PlayBackendPoll, String> {
        Ok(PlayBackendPoll::Running {
            diagnostics: Vec::new(),
        })
    }
}
