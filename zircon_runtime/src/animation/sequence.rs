//! 序列桥接在资产或世界绑定变化时编译属性写入器，再由动画插件按帧复用；缓存须与资产版本和世界绑定保持一致。
mod channel_sample;
mod compiled;
mod conversion;
mod interpolation;
mod target;
#[cfg(test)]
#[path = "sequence/tests/cases.rs"]
mod tests;
mod time;

pub(crate) use channel_sample::AnimationChannelSampleExt;
pub use compiled::{
    apply_compiled_sequence_to_world, compile_sequence_for_world, CompiledAnimationSequence,
    CompiledAnimationSequenceApplyStats,
};
