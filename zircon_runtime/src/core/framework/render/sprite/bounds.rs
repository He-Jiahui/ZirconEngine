use serde::{Deserialize, Serialize};

use crate::core::math::{Vec2, Vec3};

// TODO: [CR-RENDER-SPRITE-0001] 确认此公开包围类型的消费者和坐标空间；目前除重导出外没有调用方，顶点路径直接使用快照尺寸。
/// 精灵中心与半尺寸的独立表示，尚未接入当前场景抽取或顶点构建路径。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderSpriteBounds {
    pub center: Vec3,
    pub half_size: Vec2,
}

impl RenderSpriteBounds {
    pub const fn new(center: Vec3, half_size: Vec2) -> Self {
        Self { center, half_size }
    }
}
