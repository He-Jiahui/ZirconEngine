use std::time::Instant;

use super::super::{RenderArtifactIoPriority, RenderArtifactLoadBatch};
use super::contract::{RenderArtifactBlockAdmissionError, RenderArtifactBlockRequest};
use super::loader::{RenderArtifactBlockLoader, RenderArtifactBlockTicketBatch};

impl RenderArtifactBlockLoader {
    pub fn request_load_batch(
        &self,
        batch: &RenderArtifactLoadBatch,
        priority: RenderArtifactIoPriority,
        deadline: Option<Instant>,
    ) -> Result<RenderArtifactBlockTicketBatch, RenderArtifactBlockAdmissionError> {
        let requests = batch
            .blocks()
            .iter()
            .cloned()
            .map(|descriptor| {
                let request = RenderArtifactBlockRequest::new(descriptor, priority);
                match deadline {
                    Some(deadline) => request.with_deadline(deadline),
                    None => request,
                }
            })
            .collect::<Vec<_>>();
        self.request_batch(&requests)
    }
}
