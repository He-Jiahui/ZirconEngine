use zircon_runtime::ui::layout::UiFrame;

use super::PrecisionShape;
use crate::scene::viewport::pointer::overlay_router::frame_from_points;

impl PrecisionShape {
    pub(in crate::scene::viewport::pointer) fn hit_frame(&self) -> Option<UiFrame> {
        match self {
            Self::Line {
                start,
                end,
                radius_px,
                threshold_px,
                ..
            } => {
                let expand = *radius_px + *threshold_px;
                Some(frame_from_points(&[*start, *end], expand))
            }
            Self::Circle {
                center,
                radius_px,
                threshold_px,
                ..
            } => {
                let radius = *radius_px + *threshold_px;
                Some(UiFrame::new(
                    center.x - radius,
                    center.y - radius,
                    radius * 2.0,
                    radius * 2.0,
                ))
            }
            Self::Ring {
                segments,
                radius_px,
                thickness_px,
                threshold_px,
                ..
            } => {
                let expand = *radius_px + *thickness_px + *threshold_px;
                let points: Vec<_> = segments
                    .iter()
                    .flat_map(|(start, end)| [*start, *end])
                    .collect();
                if points.is_empty() {
                    None
                } else {
                    Some(frame_from_points(&points, expand))
                }
            }
        }
    }
}
