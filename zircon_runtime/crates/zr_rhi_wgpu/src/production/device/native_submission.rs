use zr_rhi::{RenderQueueClass, RhiError, SubmissionStatus, SubmissionTicket};

use super::super::{WgpuBufferUploadBatch, WgpuResourceUploadBatch, WgpuTextureUploadBatch};
use super::WgpuRenderDevice;

impl WgpuRenderDevice {
    /// Enqueues a transitional native buffer upload on this device generation's sole timeline.
    ///
    /// Product upload producers retain their existing native resource payload during the hard
    /// cut, but receive no queue, poll, or flush authority from this bridge.
    pub fn enqueue_native_buffer_upload_batch(
        &self,
        batch: WgpuBufferUploadBatch,
    ) -> Result<SubmissionTicket, RhiError> {
        self.ensure_admission()?;
        if batch.is_empty() {
            return Err(RhiError::EmptySubmissionPacket);
        }
        let ticket = self.submissions.begin_packet(RenderQueueClass::Copy)?;
        if let Err(error) = self.submissions.commit_buffer_upload_batch(ticket, batch) {
            self.cancel_accepted_packet(ticket);
            return Err(error);
        }
        Ok(ticket)
    }

    /// Enqueues a transitional native texture upload on this device generation's sole timeline.
    pub fn enqueue_native_texture_upload_batch(
        &self,
        batch: WgpuTextureUploadBatch,
    ) -> Result<SubmissionTicket, RhiError> {
        self.ensure_admission()?;
        if batch.is_empty() {
            return Err(RhiError::EmptySubmissionPacket);
        }
        let ticket = self.submissions.begin_packet(RenderQueueClass::Copy)?;
        if let Err(error) = self.submissions.commit_texture_upload_batch(ticket, batch) {
            self.cancel_accepted_packet(ticket);
            return Err(error);
        }
        Ok(ticket)
    }

    /// Enqueues buffer and texture setup writes as one device-generation-qualified Copy packet.
    pub fn enqueue_native_resource_upload_batch(
        &self,
        batch: WgpuResourceUploadBatch,
    ) -> Result<SubmissionTicket, RhiError> {
        self.ensure_admission()?;
        if batch.is_empty() {
            return Err(RhiError::EmptySubmissionPacket);
        }
        let ticket = self.submissions.begin_packet(RenderQueueClass::Copy)?;
        if let Err(error) = self.submissions.commit_resource_upload_batch(ticket, batch) {
            self.cancel_accepted_packet(ticket);
            return Err(error);
        }
        Ok(ticket)
    }

    /// Settles abandoned transitional work under the same bounded submission-state lock.
    pub fn settle_abandoned_native_submissions(
        &self,
        tickets: &[SubmissionTicket],
    ) -> Result<Vec<SubmissionStatus>, RhiError> {
        if let Err(error) = self.fault_gate.ensure_admission() {
            self.submissions
                .terminalize_unresolved(super::submission_terminal_status(error));
            self.lock_diagnostics()
                .terminalize_all(super::diagnostic_terminal_status(error));
            self.terminalize_surface_frames();
        }
        let statuses = self.submissions.settle_abandoned_submissions(tickets)?;
        {
            let mut diagnostics = self.lock_diagnostics();
            for (&ticket, &status) in tickets.iter().zip(&statuses) {
                if status == SubmissionStatus::Cancelled {
                    diagnostics.terminalize_submission(
                        ticket,
                        zr_rhi::DiagnosticReadbackTerminal::Cancelled,
                    );
                }
            }
        }
        self.prune_terminal_resources();
        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_submission_bridge_reuses_the_device_owner_without_queue_escape_hatches() {
        let source = include_str!("native_submission.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production native submission source");

        assert!(!production.contains("wgpu::Queue"));
        assert!(!production.contains(".poll("));
        assert!(!production.contains(".flush("));
        assert!(!production.contains("queue.submit"));
        assert!(production.contains("self.submissions.begin_packet(RenderQueueClass::Copy)?"));
        assert!(production.contains("commit_buffer_upload_batch(ticket, batch)"));
        assert!(production.contains("commit_texture_upload_batch(ticket, batch)"));
        assert!(production.contains("commit_resource_upload_batch(ticket, batch)"));
        assert!(production.contains("settle_abandoned_submissions(tickets)?"));
    }
}
