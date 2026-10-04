//! 公开独立两骨链和注视求解 API；调用方自行决定何时把求解结果并入姿态管线。
mod error;
mod look_at;
mod two_bone;

pub use error::AnimationIkError;
pub use look_at::LookAtJob;
pub use two_bone::{TwoBoneIkJob, TwoBoneIkSolution};
