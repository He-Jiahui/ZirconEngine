//! 仅向管理器同步采样暴露原始通道扩展；帧管线使用带骨架目标表的编译评估器。
//! Private channel-sampling primitives shared by the plugin pose evaluator.

#[path = "channel_sample.rs"]
mod channel_sample;
#[path = "interpolation.rs"]
mod interpolation;

pub(crate) use channel_sample::AnimationChannelSampleExt;
