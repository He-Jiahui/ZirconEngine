use crate::core::framework::scene::EntityId;
use crate::core::math::Real;
use crate::core::resource::ResourceId;

/// Runtime event emitted when an animation clip playback range crosses an event track.
/// 采样器只返回已跨越的片段事件，动画流水线再写入 World；LevelSystem 队列维持范围顺序。
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
/// LevelSystem 以该范围作为队列项，外部资源不可用或预算耗尽时保留它等待下一帧。
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClipEventSamplingRange {
    pub entity: EntityId,
    pub clip_id: ResourceId,
    pub from_time_seconds: Real,
    pub to_time_seconds: Real,
    pub looping: bool,
}

/// Resume point retained by a bounded clip-event queue for one fixed sampling range.
///
/// Playback direction comes from that range's `from_time_seconds` and `to_time_seconds`.
/// A cursor must not be reused with a different range; a direction change starts a new range.
/// 游标只属于固定范围和方向，重试时由队列原样传回，避免预算切片造成重复或漏发。
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
/// 这些限制由场景层提供给可选采样器，用来同时控制事件数、文本字节数和播放跨度。
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

/// 场景队列交给采样器的一次有界工作；`cursor` 必须随延期范围重试，
/// 以免跨帧预算耗尽时重复或跳过相同播放时间上的事件。
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

/// 一次采样的结果与续读凭据；有 `next_cursor` 时场景队列会保留原范围，
/// `None` 表示这一范围已经处理完毕，可从待处理队列移除。
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
/// `Deferred` 表示剩余队列容量不足，生产者须保留播放状态后重试；`RejectedOversized` 表示批次超过队列硬上限。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationClipEventBatchAdmission {
    Admitted,
    Deferred,
    RejectedOversized { range_count: usize, capacity: usize },
}

/// Admission outcome for all producer batches submitted against one replacement epoch.
/// 旧 epoch 的生产结果会丢弃，当前 epoch 才能把批次和游标交给动画运行时继续处理。
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
/// 实现只负责一次有界采样；资源不可用时返回 `None`，不得在 trait 内部偷偷消费或重排队列。
pub trait AnimationClipEventSampler: Send + Sync {
    fn sample_clip_events(
        &self,
        request: AnimationClipEventSamplingRequest,
    ) -> Option<AnimationClipEventSamplingBatch>;
}
