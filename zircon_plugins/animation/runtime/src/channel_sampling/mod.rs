//! Private channel-sampling primitives shared by the plugin pose evaluator.

mod channel_sample;
mod interpolation;

pub(crate) use channel_sample::AnimationChannelSampleExt;
