use zircon_runtime::core::math::Vec2;

use super::PrecisionShape;
use crate::scene::viewport::projection::distance_to_segment;

impl PrecisionShape {
    pub(in crate::scene::viewport::pointer) fn score(&self, point: Vec2) -> Option<f32> {
        match self {
            Self::Line {
                start,
                end,
                radius_px,
                threshold_px,
                ..
            } => {
                let score = distance_to_segment(point, *start, *end) - *radius_px;
                (score <= *threshold_px).then_some(score.max(0.0))
            }
            Self::Circle {
                center,
                radius_px,
                threshold_px,
                ..
            } => {
                let score = point.distance(*center) - *radius_px;
                (score <= *threshold_px).then_some(score.max(0.0))
            }
            Self::Ring {
                segments,
                thickness_px,
                threshold_px,
                ..
            } => {
                let mut best = f32::MAX;
                for (start, end) in segments {
                    best = best.min(distance_to_segment(point, *start, *end));
                }
                let score = best - *thickness_px;
                (score <= *threshold_px).then_some(score.max(0.0))
            }
        }
    }
}
