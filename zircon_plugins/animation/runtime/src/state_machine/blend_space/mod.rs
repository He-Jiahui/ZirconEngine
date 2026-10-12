//! 混合空间编译和权重类型的内部边界，供编译状态类型保存采样布局。
#[path = "blend_space_1d.rs"]
mod blend_space_1d;
#[path = "blend_space_2d.rs"]
mod blend_space_2d;
#[path = "compile_error.rs"]
mod blend_space_compile_error;
#[path = "../blend_space_weights.rs"]
mod blend_space_weights;
#[path = "geometry.rs"]
mod geometry;

pub(super) use blend_space_1d::BlendSpace1D;
pub(super) use blend_space_2d::BlendSpace2D;
pub use blend_space_compile_error::BlendSpaceCompileError;
pub(super) use blend_space_weights::{BlendSpaceWeights2, BlendSpaceWeights3};
