use std::sync::mpsc::Receiver;

use zr_rhi::{DiagnosticFrameKey, SubmissionTicket};

use super::DiagnosticReadbackSource;

pub(crate) struct DiagnosticReadbackBatchRequest {
    pub(super) source: DiagnosticReadbackSource,
    pub(super) staging_offset: u64,
}

impl DiagnosticReadbackBatchRequest {
    pub(crate) const fn source(&self) -> &DiagnosticReadbackSource {
        &self.source
    }

    pub(crate) const fn staging_offset(&self) -> u64 {
        self.staging_offset
    }
}

pub(crate) struct DiagnosticReadbackBatch {
    pub(super) requests: Vec<DiagnosticReadbackBatchRequest>,
    pub(super) byte_len: u64,
}

impl DiagnosticReadbackBatch {
    pub(crate) fn requests(&self) -> &[DiagnosticReadbackBatchRequest] {
        &self.requests
    }

    pub(crate) const fn byte_len(&self) -> u64 {
        self.byte_len
    }
}

pub(super) struct ActiveDiagnosticReadbackBatch {
    pub(super) requests: Vec<DiagnosticReadbackBatchRequest>,
    pub(super) byte_len: u64,
}

pub(super) struct InFlightDiagnosticReadbackBatch {
    pub(super) frame_key: DiagnosticFrameKey,
    pub(super) staging: wgpu::Buffer,
    pub(super) byte_len: u64,
    pub(super) requests: Vec<DiagnosticReadbackBatchRequest>,
    pub(super) map_receiver: Option<Receiver<Result<(), wgpu::BufferAsyncError>>>,
}
