//! 作者掩码到运行时权重的导出边界；具体权重属于已编译骨架。
#[path = "asset.rs"]
mod asset;
#[path = "compile.rs"]
mod compile;
#[path = "error.rs"]
mod error;

pub use asset::{AvatarMaskAsset, AvatarMaskRule};
pub use compile::MaskWeights;
pub use error::AvatarMaskError;
