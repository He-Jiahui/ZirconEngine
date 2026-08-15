use crate::core::framework::scene::EntityId;
use crate::core::math::Real;
use crate::core::resource::ResourceId;

/// Runtime event emitted when an animation clip playback range crosses an event track.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClipEvent {
    pub entity: EntityId,
    pub target_id: Option<String>,
    pub event: String,
    pub payload: Option<String>,
    pub clip_time_seconds: Real,
    pub playback_time_seconds: Real,
}

/// One playback interval awaiting bounded clip-event sampling.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClipEventSamplingRange {
    pub entity: EntityId,
    pub clip_id: ResourceId,
    pub from_time_seconds: Real,
    pub to_time_seconds: Real,
    pub looping: bool,
}

/// Resume point retained by a bounded clip-event queue.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClipEventSamplingCursor {
    pub playback_time_seconds: Real,
    pub last_event: Option<Box<str>>,
    pub last_track_index: usize,
}

impl AnimationClipEventSamplingCursor {
    pub fn at_range_start(playback_time_seconds: Real) -> Self {
        Self {
            playback_time_seconds,
            last_event: None,
            last_track_index: 0,
        }
    }
}

/// Per-frame bounds for draining clip events.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationClipEventSamplingLimits {
    pub max_events: usize,
    pub max_event_bytes: usize,
    pub max_playback_span_seconds: Real,
}

impl Default for AnimationClipEventSamplingLimits {
    fn default() -> Self {
        Self {
            max_events: 64,
            max_event_bytes: 64 * 1024,
            max_playback_span_seconds: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClipEventSamplingRequest {
    pub entity: EntityId,
    pub clip_id: ResourceId,
    pub from_time_seconds: Real,
    pub to_time_seconds: Real,
    pub looping: bool,
    pub cursor: AnimationClipEventSamplingCursor,
    pub limits: AnimationClipEventSamplingLimits,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationClipEventSamplingBatch {
    pub events: Vec<AnimationClipEvent>,
    pub next_cursor: Option<AnimationClipEventSamplingCursor>,
    pub emitted_event_bytes: usize,
    pub playback_span_seconds: Real,
    pub budget_exhausted: bool,
    pub oversized_event_count: usize,
}

/// Admission outcome for one producer batch submitted to the bounded scene queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationClipEventBatchAdmission {
    Admitted,
    Deferred,
    RejectedOversized { range_count: usize, capacity: usize },
}

/// Admission outcome for all producer batches submitted against one replacement epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationClipEventQueueAdmission {
    Current {
        batch_admissions: Vec<AnimationClipEventBatchAdmission>,
        admitted_range_count: usize,
        deferred_range_count: usize,
        rejected_range_count: usize,
    },
    RetiredEpoch,
}

/// Optional animation implementations sample one bounded request through this contract.
///
/// `None` means the referenced clip is currently unavailable. Queue retention and retry policy
/// remain owned by the scene level that submitted the request.
pub trait AnimationClipEventSampler: Send + Sync {
    fn sample_clip_events(
        &self,
        request: AnimationClipEventSamplingRequest,
    ) -> Option<AnimationClipEventSamplingBatch>;
}
