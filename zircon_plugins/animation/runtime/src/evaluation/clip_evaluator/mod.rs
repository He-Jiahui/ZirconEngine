//! 剪辑评估器的缓存、校验、时间和诊断边界；对外只导出带资源修订的采样契约。
mod animation_asset_revision;
mod animation_channel_data_role;
mod animation_clip_evaluator;
mod animation_clip_evaluator_stats;
mod animation_evaluation_error;
mod animation_transform_channel;
mod cache;
mod cache_policy;
mod channel_sample;
mod channel_validation;
mod diagnostics;
mod hermite;
mod sample;
mod time;

pub use animation_asset_revision::AnimationAssetRevision;
pub use animation_channel_data_role::AnimationChannelDataRole;
pub use animation_clip_evaluator::AnimationClipEvaluator;
pub use animation_clip_evaluator_stats::AnimationClipEvaluatorStats;
pub use animation_evaluation_error::AnimationEvaluationError;
pub use animation_transform_channel::AnimationTransformChannel;
