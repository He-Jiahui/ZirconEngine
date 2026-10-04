//! 纹理复制的中立区域与平面选择类型；WGPU 编码器据此执行边界和 aspect 校验。

mod aspect;
mod region;

pub use aspect::TextureCopyAspect;
pub use region::TextureCopyRegion;
