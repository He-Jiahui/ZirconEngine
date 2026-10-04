use serde::{Deserialize, Serialize};

use crate::core::framework::render::{RenderPhaseQueue, RenderPhaseQueueSummary};

use super::RenderSpriteSnapshot;

/// 一帧内的精灵快照和绘制阶段队列；队列索引必须指向同一批 `sprites`。
/// 场景抽取建立该关系，渲染器按队列顺序并结合相机层过滤生成顶点。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteExtract {
    pub sprites: Vec<RenderSpriteSnapshot>,
    pub phase_queue: RenderPhaseQueue,
}

impl SpriteExtract {
    /// Builds a diagnostics summary from the current sorted sprite phase queue.
    pub fn phase_queue_summary(&self) -> RenderPhaseQueueSummary {
        self.phase_queue.summary()
    }
}
