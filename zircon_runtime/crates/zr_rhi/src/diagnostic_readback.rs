//! Submission-qualified diagnostic readback admission and terminal tracking.
//!
//! Native backends own copy encoding, mapping, and callback execution. This
//! module owns the backend-neutral lifecycle: every admitted request is bound
//! to exactly one device-generation submission, all quotas are checked before
//! native work is recorded, and every terminal path can emit one receipt.

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{DeviceGeneration, DeviceId, SubmissionTicket};

const DEFAULT_MAX_REQUESTS_PER_FRAME: usize = 64;
const DEFAULT_MAX_PENDING_REQUESTS: usize = 192;
const DEFAULT_MAX_REQUEST_BYTES: u64 = 16 * 1024 * 1024;
const DEFAULT_MAX_FRAME_BYTES: u64 = 32 * 1024 * 1024;
const DEFAULT_MAX_PENDING_BYTES: u64 = 64 * 1024 * 1024;
const DEFAULT_MAX_COMPLETED_RECEIPTS: usize = 256;
const DEFAULT_MAX_DIAGNOSTIC_PASSES: usize = 128;
const DEFAULT_MAX_TIMESTAMP_SCOPES: usize = 64;
const DEFAULT_MAX_PIPELINE_STATISTICS_SCOPES: usize = 64;

/// Readback and query request categories sharing the same lifecycle owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticReadbackKind {
    Buffer,
    Texture,
    Timestamp,
    PipelineStatistics,
}

/// Bounded admission policy for diagnostic copies and query resolves.
///
/// A backend may choose lower values for constrained devices, but it cannot
/// bypass these checks after request recording begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticReadbackBudget {
    max_requests_per_frame: usize,
    max_pending_requests: usize,
    max_request_bytes: u64,
    max_frame_bytes: u64,
    max_pending_bytes: u64,
    max_completed_receipts: usize,
    max_diagnostic_passes: usize,
    max_timestamp_scopes: usize,
    max_pipeline_statistics_scopes: usize,
}

impl DiagnosticReadbackBudget {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        max_requests_per_frame: usize,
        max_pending_requests: usize,
        max_request_bytes: u64,
        max_frame_bytes: u64,
        max_pending_bytes: u64,
        max_completed_receipts: usize,
    ) -> Self {
        Self {
            max_requests_per_frame,
            max_pending_requests,
            max_request_bytes,
            max_frame_bytes,
            max_pending_bytes,
            max_completed_receipts,
            max_diagnostic_passes: DEFAULT_MAX_DIAGNOSTIC_PASSES,
            max_timestamp_scopes: DEFAULT_MAX_TIMESTAMP_SCOPES,
            max_pipeline_statistics_scopes: DEFAULT_MAX_PIPELINE_STATISTICS_SCOPES,
        }
    }

    pub const fn with_query_limits(
        mut self,
        max_diagnostic_passes: usize,
        max_timestamp_scopes: usize,
        max_pipeline_statistics_scopes: usize,
    ) -> Self {
        self.max_diagnostic_passes = max_diagnostic_passes;
        self.max_timestamp_scopes = max_timestamp_scopes;
        self.max_pipeline_statistics_scopes = max_pipeline_statistics_scopes;
        self
    }

    pub const fn max_requests_per_frame(self) -> usize {
        self.max_requests_per_frame
    }

    pub const fn max_pending_requests(self) -> usize {
        self.max_pending_requests
    }

    pub const fn max_request_bytes(self) -> u64 {
        self.max_request_bytes
    }

    pub const fn max_frame_bytes(self) -> u64 {
        self.max_frame_bytes
    }

    pub const fn max_pending_bytes(self) -> u64 {
        self.max_pending_bytes
    }

    pub const fn max_completed_receipts(self) -> usize {
        self.max_completed_receipts
    }

    pub const fn max_diagnostic_passes(self) -> usize {
        self.max_diagnostic_passes
    }

    pub const fn max_timestamp_scopes(self) -> usize {
        self.max_timestamp_scopes
    }

    pub const fn max_pipeline_statistics_scopes(self) -> usize {
        self.max_pipeline_statistics_scopes
    }
}

impl Default for DiagnosticReadbackBudget {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_REQUESTS_PER_FRAME,
            DEFAULT_MAX_PENDING_REQUESTS,
            DEFAULT_MAX_REQUEST_BYTES,
            DEFAULT_MAX_FRAME_BYTES,
            DEFAULT_MAX_PENDING_BYTES,
            DEFAULT_MAX_COMPLETED_RECEIPTS,
        )
        .with_query_limits(
            DEFAULT_MAX_DIAGNOSTIC_PASSES,
            DEFAULT_MAX_TIMESTAMP_SCOPES,
            DEFAULT_MAX_PIPELINE_STATISTICS_SCOPES,
        )
    }
}

/// Opaque per-device-generation request identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DiagnosticReadbackRequestId(u64);

impl DiagnosticReadbackRequestId {
    const fn new(sequence: u64) -> Self {
        Self(sequence)
    }

    pub const fn sequence(self) -> u64 {
        self.0
    }
}

/// The exact submission that owns an encoded diagnostic frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagnosticFrameKey {
    device_id: DeviceId,
    generation: DeviceGeneration,
    submission: SubmissionTicket,
}

impl DiagnosticFrameKey {
    const fn new(submission: SubmissionTicket) -> Self {
        Self {
            device_id: submission.device_id(),
            generation: submission.generation(),
            submission,
        }
    }

    pub const fn device_id(self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(self) -> DeviceGeneration {
        self.generation
    }

    pub const fn submission(self) -> SubmissionTicket {
        self.submission
    }
}

/// The only terminal outcomes a diagnostic requester can observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticReadbackTerminal {
    Succeeded,
    /// The selected backend device cannot execute this optional diagnostic.
    Unavailable,
    MapFailed,
    Cancelled,
    DeviceLost,
    Shutdown,
    OverBudget,
}

/// A terminal delivery token for a request. Backends dispatch it on their
/// completion executor and do not manufacture a second one after removal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticReadbackReceipt {
    request: DiagnosticReadbackRequestId,
    kind: DiagnosticReadbackKind,
    byte_len: u64,
    frame_key: Option<DiagnosticFrameKey>,
    terminal: DiagnosticReadbackTerminal,
}

impl DiagnosticReadbackReceipt {
    pub const fn request(self) -> DiagnosticReadbackRequestId {
        self.request
    }

    pub const fn kind(self) -> DiagnosticReadbackKind {
        self.kind
    }

    pub const fn byte_len(self) -> u64 {
        self.byte_len
    }

    pub const fn frame_key(self) -> Option<DiagnosticFrameKey> {
        self.frame_key
    }

    pub const fn terminal(self) -> DiagnosticReadbackTerminal {
        self.terminal
    }
}

/// Admission result for callers that need every quota rejection to become an
/// auditable terminal receipt instead of a silently dropped request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticReadbackAdmission {
    Admitted(DiagnosticReadbackRequestId),
    Rejected(DiagnosticReadbackReceipt),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DiagnosticReadbackError {
    #[error("diagnostic readback frame {requested} cannot start while frame {active} is active")]
    FrameAlreadyActive { active: u64, requested: u64 },
    #[error("diagnostic readback requests require an active frame")]
    NoActiveFrame,
    #[error("diagnostic readback requests must contain at least one byte")]
    EmptyRequest,
    #[error("diagnostic readback request sequence is exhausted")]
    RequestSequenceExhausted,
    #[error("diagnostic readback staging layout overflowed")]
    StagingLayoutOverflow,
    #[error("diagnostic readback request of {requested_bytes} bytes exceeds {limit_bytes} bytes")]
    RequestBytesExceeded {
        requested_bytes: u64,
        limit_bytes: u64,
    },
    #[error("diagnostic readback frame request count {current_requests} exceeds {limit} requests")]
    FrameRequestLimitExceeded {
        current_requests: usize,
        limit: usize,
    },
    #[error(
        "diagnostic readback frame bytes {current_bytes} plus {requested_bytes} exceed {limit_bytes}"
    )]
    FrameBytesExceeded {
        current_bytes: u64,
        requested_bytes: u64,
        limit_bytes: u64,
    },
    #[error(
        "diagnostic readback pending request count {current_requests} exceeds {limit} requests"
    )]
    PendingRequestLimitExceeded {
        current_requests: usize,
        limit: usize,
    },
    #[error(
        "diagnostic readback pending bytes {current_bytes} plus {requested_bytes} exceed {limit_bytes}"
    )]
    PendingBytesExceeded {
        current_bytes: u64,
        requested_bytes: u64,
        limit_bytes: u64,
    },
    #[error(
        "diagnostic readback submission {received:?} does not belong to device {expected_device:?} generation {expected_generation:?}"
    )]
    SubmissionIdentityMismatch {
        expected_device: DeviceId,
        expected_generation: DeviceGeneration,
        received: SubmissionTicket,
    },
}

/// Bounded, submission-qualified lifecycle owner for diagnostic requests.
///
/// This type intentionally contains no callback or native WGPU object. The
/// WGPU layer must do copy/map work and submit every returned receipt through
/// its single completion executor, keeping slow consumers out of render submit.
pub struct DiagnosticReadbackTracker {
    device_id: DeviceId,
    generation: DeviceGeneration,
    budget: DiagnosticReadbackBudget,
    next_request_sequence: u64,
    active_frame: Option<ActiveDiagnosticFrame>,
    pending: HashMap<DiagnosticReadbackRequestId, PendingDiagnosticRequest>,
    requests_by_frame: HashMap<DiagnosticFrameKey, Vec<DiagnosticReadbackRequestId>>,
    pending_bytes: u64,
    completed: VecDeque<DiagnosticReadbackReceipt>,
    dropped_completed_receipt_count: u64,
}

struct ActiveDiagnosticFrame {
    frame_index: u64,
    request_ids: Vec<DiagnosticReadbackRequestId>,
    bytes: u64,
}

struct PendingDiagnosticRequest {
    kind: DiagnosticReadbackKind,
    byte_len: u64,
    frame_key: Option<DiagnosticFrameKey>,
}

impl DiagnosticReadbackTracker {
    pub fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        budget: DiagnosticReadbackBudget,
    ) -> Self {
        Self {
            device_id,
            generation,
            budget,
            next_request_sequence: 1,
            active_frame: None,
            pending: HashMap::new(),
            requests_by_frame: HashMap::new(),
            pending_bytes: 0,
            completed: VecDeque::new(),
            dropped_completed_receipt_count: 0,
        }
    }

    pub const fn budget(&self) -> DiagnosticReadbackBudget {
        self.budget
    }

    pub fn begin_frame(&mut self, frame_index: u64) -> Result<(), DiagnosticReadbackError> {
        if let Some(active) = &self.active_frame {
            return Err(DiagnosticReadbackError::FrameAlreadyActive {
                active: active.frame_index,
                requested: frame_index,
            });
        }
        self.active_frame = Some(ActiveDiagnosticFrame {
            frame_index,
            request_ids: Vec::new(),
            bytes: 0,
        });
        Ok(())
    }

    /// Admits a request before a native copy or query resolve is encoded.
    pub fn admit(
        &mut self,
        kind: DiagnosticReadbackKind,
        byte_len: u64,
    ) -> Result<DiagnosticReadbackRequestId, DiagnosticReadbackError> {
        let active = self
            .active_frame
            .as_ref()
            .ok_or(DiagnosticReadbackError::NoActiveFrame)?;
        if byte_len == 0 {
            return Err(DiagnosticReadbackError::EmptyRequest);
        }
        if byte_len > self.budget.max_request_bytes {
            return Err(DiagnosticReadbackError::RequestBytesExceeded {
                requested_bytes: byte_len,
                limit_bytes: self.budget.max_request_bytes,
            });
        }
        if active.request_ids.len() >= self.budget.max_requests_per_frame {
            return Err(DiagnosticReadbackError::FrameRequestLimitExceeded {
                current_requests: active.request_ids.len(),
                limit: self.budget.max_requests_per_frame,
            });
        }
        if byte_len > self.budget.max_frame_bytes.saturating_sub(active.bytes) {
            return Err(DiagnosticReadbackError::FrameBytesExceeded {
                current_bytes: active.bytes,
                requested_bytes: byte_len,
                limit_bytes: self.budget.max_frame_bytes,
            });
        }
        if self.pending.len() >= self.budget.max_pending_requests {
            return Err(DiagnosticReadbackError::PendingRequestLimitExceeded {
                current_requests: self.pending.len(),
                limit: self.budget.max_pending_requests,
            });
        }
        if byte_len
            > self
                .budget
                .max_pending_bytes
                .saturating_sub(self.pending_bytes)
        {
            return Err(DiagnosticReadbackError::PendingBytesExceeded {
                current_bytes: self.pending_bytes,
                requested_bytes: byte_len,
                limit_bytes: self.budget.max_pending_bytes,
            });
        }

        let request = self.allocate_request_id()?;
        self.pending.insert(
            request,
            PendingDiagnosticRequest {
                kind,
                byte_len,
                frame_key: None,
            },
        );
        self.pending_bytes = self.pending_bytes.saturating_add(byte_len);
        if let Some(active) = self.active_frame.as_mut() {
            active.request_ids.push(request);
            active.bytes = active.bytes.saturating_add(byte_len);
            Ok(request)
        } else {
            self.pending.remove(&request);
            self.pending_bytes = self.pending_bytes.saturating_sub(byte_len);
            Err(DiagnosticReadbackError::NoActiveFrame)
        }
    }

    /// Admits a request or records a terminal `OverBudget` receipt for quota
    /// rejections. Invalid frame state and empty requests remain typed errors
    /// because no request exists to dispatch.
    pub fn admit_or_reject(
        &mut self,
        kind: DiagnosticReadbackKind,
        byte_len: u64,
    ) -> Result<DiagnosticReadbackAdmission, DiagnosticReadbackError> {
        match self.admit(kind, byte_len) {
            Ok(request) => Ok(DiagnosticReadbackAdmission::Admitted(request)),
            Err(
                DiagnosticReadbackError::RequestBytesExceeded { .. }
                | DiagnosticReadbackError::FrameRequestLimitExceeded { .. }
                | DiagnosticReadbackError::FrameBytesExceeded { .. }
                | DiagnosticReadbackError::PendingRequestLimitExceeded { .. }
                | DiagnosticReadbackError::PendingBytesExceeded { .. },
            ) => {
                let request = self.allocate_request_id()?;
                let receipt = DiagnosticReadbackReceipt {
                    request,
                    kind,
                    byte_len,
                    frame_key: None,
                    terminal: DiagnosticReadbackTerminal::OverBudget,
                };
                self.push_completed(receipt);
                Ok(DiagnosticReadbackAdmission::Rejected(receipt))
            }
            Err(error) => Err(error),
        }
    }

    /// Binds all active requests to the exact submission that encoded them.
    pub fn bind_active_frame(
        &mut self,
        submission: SubmissionTicket,
    ) -> Result<DiagnosticFrameKey, DiagnosticReadbackError> {
        self.validate_submission_identity(submission)?;
        let active = self
            .active_frame
            .take()
            .ok_or(DiagnosticReadbackError::NoActiveFrame)?;
        let key = DiagnosticFrameKey::new(submission);
        for request in &active.request_ids {
            if let Some(pending) = self.pending.get_mut(request) {
                pending.frame_key = Some(key);
            }
        }
        if !active.request_ids.is_empty() {
            self.requests_by_frame.insert(key, active.request_ids);
        }
        Ok(key)
    }

    /// Terminates only the unbound active frame. This is used when native
    /// packet encoding fails after admission but before a submission ticket
    /// can safely own the requests.
    pub fn terminalize_active_frame(
        &mut self,
        terminal: DiagnosticReadbackTerminal,
    ) -> Vec<DiagnosticReadbackReceipt> {
        self.active_frame
            .take()
            .map(|active| active.request_ids)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|request| self.terminalize(request, terminal))
            .collect()
    }

    /// Emits the first terminal receipt for a request, or `None` when it was
    /// already terminalized by completion, cancellation, loss, or shutdown.
    pub fn terminalize(
        &mut self,
        request: DiagnosticReadbackRequestId,
        terminal: DiagnosticReadbackTerminal,
    ) -> Option<DiagnosticReadbackReceipt> {
        let pending = self.pending.remove(&request)?;
        self.pending_bytes = self.pending_bytes.saturating_sub(pending.byte_len);
        if let Some(active) = self.active_frame.as_mut() {
            active
                .request_ids
                .retain(|active_request| *active_request != request);
        }
        if let Some(frame_key) = pending.frame_key {
            let remove_frame = if let Some(requests) = self.requests_by_frame.get_mut(&frame_key) {
                requests.retain(|frame_request| *frame_request != request);
                requests.is_empty()
            } else {
                false
            };
            if remove_frame {
                self.requests_by_frame.remove(&frame_key);
            }
        }
        let receipt = DiagnosticReadbackReceipt {
            request,
            kind: pending.kind,
            byte_len: pending.byte_len,
            frame_key: pending.frame_key,
            terminal,
        };
        self.push_completed(receipt);
        Some(receipt)
    }

    pub fn terminalize_frame(
        &mut self,
        frame_key: DiagnosticFrameKey,
        terminal: DiagnosticReadbackTerminal,
    ) -> Vec<DiagnosticReadbackReceipt> {
        self.requests_by_frame
            .remove(&frame_key)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|request| self.terminalize(request, terminal))
            .collect()
    }

    /// Terminalizes both encoded and still-active requests, preserving request
    /// sequence order for deterministic shutdown and device-loss dispatch.
    pub fn terminalize_all(
        &mut self,
        terminal: DiagnosticReadbackTerminal,
    ) -> Vec<DiagnosticReadbackReceipt> {
        self.active_frame = None;
        let mut requests = self.pending.keys().copied().collect::<Vec<_>>();
        requests.sort_unstable();
        requests
            .into_iter()
            .filter_map(|request| self.terminalize(request, terminal))
            .collect()
    }

    pub fn pending_request_count(&self) -> usize {
        self.pending.len()
    }

    pub fn pending_bytes(&self) -> u64 {
        self.pending_bytes
    }

    pub fn completed_receipt_count(&self) -> usize {
        self.completed.len()
    }

    pub fn dropped_completed_receipt_count(&self) -> u64 {
        self.dropped_completed_receipt_count
    }

    pub fn take_completed_receipt(&mut self) -> Option<DiagnosticReadbackReceipt> {
        self.completed.pop_front()
    }

    fn validate_submission_identity(
        &self,
        submission: SubmissionTicket,
    ) -> Result<(), DiagnosticReadbackError> {
        if submission.device_id() != self.device_id || submission.generation() != self.generation {
            return Err(DiagnosticReadbackError::SubmissionIdentityMismatch {
                expected_device: self.device_id,
                expected_generation: self.generation,
                received: submission,
            });
        }
        Ok(())
    }

    fn allocate_request_id(
        &mut self,
    ) -> Result<DiagnosticReadbackRequestId, DiagnosticReadbackError> {
        let sequence = self.next_request_sequence;
        self.next_request_sequence = sequence
            .checked_add(1)
            .ok_or(DiagnosticReadbackError::RequestSequenceExhausted)?;
        Ok(DiagnosticReadbackRequestId::new(sequence))
    }

    fn push_completed(&mut self, receipt: DiagnosticReadbackReceipt) {
        if self.budget.max_completed_receipts == 0 {
            self.dropped_completed_receipt_count =
                self.dropped_completed_receipt_count.saturating_add(1);
            return;
        }
        if self.completed.len() >= self.budget.max_completed_receipts {
            self.completed.pop_front();
            self.dropped_completed_receipt_count =
                self.dropped_completed_receipt_count.saturating_add(1);
        }
        self.completed.push_back(receipt);
    }
}
