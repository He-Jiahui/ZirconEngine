//! 姿态存储和混合的公共导出层；状态机层与剪辑评估器共享该行序契约。
#[path = "blend.rs"]
mod blend;
#[path = "pose_buffer.rs"]
mod pose_buffer;
#[path = "storage.rs"]
mod storage;

pub use blend::{PoseLayer, PoseLayerBlendMode};
pub use pose_buffer::PoseBuffer;
