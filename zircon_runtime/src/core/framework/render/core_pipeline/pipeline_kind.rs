use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 相机选择的 2D/3D 调度路线；它控制阶段归属，不替代投影模式。
pub enum CorePipelineKind {
    Core2d,
    #[default]
    Core3d,
}
