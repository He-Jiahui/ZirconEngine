use serde::{Deserialize, Serialize};

use super::RenderPhase;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RenderPhaseSortKey(i64);

impl RenderPhaseSortKey {
    pub const fn new(raw: i64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> i64 {
        self.0
    }

    pub fn for_mesh(phase: RenderPhase, depth: f32, tie_breaker: u64) -> Self {
        let finite_depth = if depth.is_finite() { depth } else { 0.0 };
        let depth_key = (finite_depth * 1000.0).round() as i64;
        let tie_key = (tie_breaker & 0x3ff) as i64;
        let ordered_depth = if phase.is_transparent() {
            -depth_key
        } else {
            depth_key
        };

        Self((ordered_depth << 10) | tie_key)
    }

    pub fn for_sprite(phase: RenderPhase, z_order: i32, depth: f32, tie_breaker: u64) -> Self {
        let finite_depth = if depth.is_finite() { depth } else { 0.0 };
        let z_key = i64::from(z_order) + 32_768;
        let depth_key = (finite_depth * 1000.0).round() as i64;
        let ordered_depth = if phase.is_transparent() {
            -depth_key
        } else {
            depth_key
        };
        let tie_key = (tie_breaker & 0x3ff) as i64;

        Self((z_key << 32) | ((ordered_depth + 1_048_576) << 10) | tie_key)
    }
}
