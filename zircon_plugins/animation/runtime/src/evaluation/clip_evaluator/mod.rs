//! 剪辑评估器的缓存、校验、时间和诊断边界；对外只导出带资源修订的采样契约。
#[path = "../animation_asset_revision.rs"]
mod animation_asset_revision;
#[path = "../animation_channel_data_role.rs"]
mod animation_channel_data_role;
#[path = "../animation_clip_evaluator.rs"]
mod animation_clip_evaluator;
#[path = "../../animation_clip_evaluator_stats.rs"]
mod animation_clip_evaluator_stats;
#[path = "../animation_evaluation_error.rs"]
mod animation_evaluation_error;
#[path = "../animation_transform_channel.rs"]
mod animation_transform_channel;
#[path = "cache.rs"]
mod cache;
#[path = "cache_policy.rs"]
mod cache_policy;
#[path = "channel_sample.rs"]
mod channel_sample;
#[path = "../channel_validation.rs"]
mod channel_validation;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "hermite.rs"]
mod hermite;
#[path = "sample.rs"]
mod sample;
#[path = "time.rs"]
mod time;

pub use animation_asset_revision::AnimationAssetRevision;
pub use animation_channel_data_role::AnimationChannelDataRole;
pub use animation_clip_evaluator::AnimationClipEvaluator;
pub use animation_clip_evaluator_stats::AnimationClipEvaluatorStats;
pub use animation_evaluation_error::AnimationEvaluationError;
pub use animation_transform_channel::AnimationTransformChannel;
