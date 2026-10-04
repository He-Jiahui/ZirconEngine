//! 精确评分把屏幕点与形状边界比较，容差内才参与拾取；非零评分用于候选排序，提前返回不能改变形状间相对接近度。

use zircon_runtime_interface::math::Vec2;

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
                    if *threshold_px >= 0.0 {
                        if best <= *thickness_px {
                            return Some(0.0);
                        }
                        // BUG: [CR-EDITOR-SP-0006] 容差内首段的非零分数还不是整环最小值；此处早退使候选排序依赖环的分段起点。
                        if best <= *thickness_px + *threshold_px {
                            return Some(best - *thickness_px);
                        }
                    }
                }
                let score = best - *thickness_px;
                (score <= *threshold_px).then_some(score.max(0.0))
            }
        }
    }
}

#[cfg(test)]
#[path = "precision_shape_score/tests/early_exit_tests.rs"]
mod early_exit_tests;

#[cfg(test)]
#[path = "precision_shape_score/tests/threshold_exit_tests.rs"]
mod threshold_exit_tests;
