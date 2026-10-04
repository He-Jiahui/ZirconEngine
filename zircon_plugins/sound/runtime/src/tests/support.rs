//! 本组测试挂接相同运行时边界的夹具和行为用例；具体限制由子模块的入口断言界定。
mod assertions;
mod assets;
mod effects;
mod listener;

pub(super) use assertions::{assert_sample_near, assert_samples_near};
pub(super) use assets::{
    test_clip, test_clip_with_channels, test_clip_with_layout, test_clip_with_rate,
    test_stereo_clip_with_rate,
};
pub(super) use effects::test_effect;
pub(super) use listener::test_listener;
