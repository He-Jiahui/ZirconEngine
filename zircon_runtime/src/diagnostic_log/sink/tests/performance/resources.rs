use std::sync::Arc;
use std::time::Duration;

use super::super::super::DiagnosticLogState;
use super::rss::{RssSampler, RssSnapshot};
use crate::diagnostic_log::DiagnosticLogSinkSnapshot;

pub(super) struct CaseResources {
    state: Arc<DiagnosticLogState>,
    rss: Option<RssSampler>,
    finished: bool,
}

impl CaseResources {
    pub(super) fn new(state: DiagnosticLogState) -> Self {
        Self {
            state: Arc::new(state),
            rss: Some(RssSampler::start()),
            finished: false,
        }
    }

    pub(super) fn state(&self) -> Arc<DiagnosticLogState> {
        Arc::clone(&self.state)
    }

    pub(super) fn finish(mut self) -> (DiagnosticLogSinkSnapshot, RssSnapshot) {
        let sink = self.state.sink.as_ref().expect("test sink runtime");
        assert!(sink.shutdown(Duration::from_secs(30)));
        let snapshot = self.state.snapshot().expect("sink snapshot");
        let rss = self.rss.take().expect("RSS sampler").finish();
        self.finished = true;
        (snapshot, rss)
    }
}

impl Drop for CaseResources {
    fn drop(&mut self) {
        if !self.finished {
            if let Some(sink) = self.state.sink.as_ref() {
                let _ = sink.shutdown(Duration::from_secs(2));
            }
        }
        self.rss.take();
    }
}
