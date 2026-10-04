//! 候选把屏幕形状命中转换为评分与深度对，供 Runtime 统一排序；未命中不产生可参与排序的记录。

use zircon_runtime_interface::math::Vec2;

use super::{CandidateScore, PrecisionCandidate};

impl PrecisionCandidate {
    pub(in crate::scene::viewport::pointer) fn score(&self, point: Vec2) -> Option<CandidateScore> {
        self.shape.score(point).map(|score| CandidateScore {
            score,
            depth: self.shape.depth(),
        })
    }
}
