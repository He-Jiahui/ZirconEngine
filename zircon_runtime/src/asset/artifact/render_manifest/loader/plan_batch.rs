use std::time::Instant;

use super::super::{RenderArtifactIoPriority, RenderArtifactLoadBatch};
use super::contract::{RenderArtifactBlockAdmissionError, RenderArtifactBlockRequest};
use super::loader::{RenderArtifactBlockLoader, RenderArtifactBlockTicketBatch};

impl RenderArtifactBlockLoader {
    /// 把清单规划好的依赖批次一次登记为票据；调用方仍需显式派发并保留票据至完成。
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
