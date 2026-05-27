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
