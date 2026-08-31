use thiserror::Error;
use zr_rhi::{DeviceGeneration, DeviceId, SubmissionPollReceipt, SubmissionTicket};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderSceneSubmissionCompletionStatus {
    #[default]
    None,
    Completed,
    Failed,
    Cancelled,
    DeviceLost,
    ObservationFailed,
    TrackingFailed,
}

impl RenderSceneSubmissionCompletionStatus {
    pub const fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Completed => 1,
            Self::Failed => 2,
            Self::Cancelled => 3,
            Self::DeviceLost => 4,
            Self::ObservationFailed => 5,
            Self::TrackingFailed => 6,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::DeviceLost => "device_lost",
            Self::ObservationFailed => "observation_failed",
            Self::TrackingFailed => "tracking_failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderSceneSubmissionCompletionFailure {
    #[default]
    None,
    SubmissionOwnerMismatch,
    SubmissionSequenceDidNotAdvance,
    CapacityExceeded,
    StatusUnavailable,
}

impl RenderSceneSubmissionCompletionFailure {
    pub const fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::SubmissionOwnerMismatch => 1,
            Self::SubmissionSequenceDidNotAdvance => 2,
            Self::CapacityExceeded => 3,
            Self::StatusUnavailable => 4,
        }
    }
}

/// Most recent completion observation or tracking failure for a scene submission.
///
/// This report is intentionally separate from command-recording reports. A
/// frame can be recorded and submitted successfully before a later poll
/// observes its terminal GPU status. The identity fields describe the latest
/// reported event, while the count fields describe the latest accepted poll.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderSceneSubmissionCompletionReport {
    pub status: RenderSceneSubmissionCompletionStatus,
    pub failure: RenderSceneSubmissionCompletionFailure,
    pub frame_generation: u64,
    pub submission: Option<SubmissionTicket>,
    pub observed_after_poll: Option<SubmissionPollReceipt>,
    /// Current number of submissions still awaiting a terminal observation.
    pub pending_submission_count: usize,
    pub tracking_capacity: usize,
    /// Number of pending tickets inspected by the latest accepted poll.
    pub last_poll_observed_submission_count: usize,
    /// Number of terminal statuses found by the latest accepted poll.
    pub last_poll_terminal_submission_count: usize,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum RenderSceneSubmissionCompletionError {
    #[error(
        "scene submission completion poll owner {poll_device:?}/{poll_generation:?} does not match journal owner {journal_device:?}/{journal_generation:?}"
    )]
    PollOwnerMismatch {
        poll_device: DeviceId,
        poll_generation: DeviceGeneration,
        journal_device: DeviceId,
        journal_generation: DeviceGeneration,
    },
    #[error(
        "scene submission completion poll sequence {poll_sequence} must advance beyond prior sequence {previous_sequence}"
    )]
    PollSequenceDidNotAdvance {
        previous_sequence: u64,
        poll_sequence: u64,
    },
    #[error(
        "scene submission completion status batch returned {actual} entries for {expected} tickets"
    )]
    StatusResultCountMismatch { expected: usize, actual: usize },
}
