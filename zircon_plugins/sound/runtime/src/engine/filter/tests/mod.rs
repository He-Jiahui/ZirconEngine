//! 滤波系数与每声道状态只在测试配置中编译，用来固定旧块级滤波契约；Kira 图编译会拒绝当前不支持的高级效果。
#[path = "apply.rs"]
mod apply;
#[path = "coefficients.rs"]
mod coefficients;
#[path = "constants.rs"]
mod constants;
#[path = "shelf.rs"]
mod shelf;
#[path = "state.rs"]
mod state;

pub(crate) use apply::apply_biquad_filter_block;
pub(crate) use state::SoundBiquadFilterState;

#[cfg(test)]
#[path = "cases.rs"]
mod tests;
