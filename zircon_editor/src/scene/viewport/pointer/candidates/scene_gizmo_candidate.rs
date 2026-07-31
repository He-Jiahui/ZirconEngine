use crate::scene::viewport::OverlayPickShape;

use crate::scene::viewport::pointer::constants::{GIZMO_PICK_THRESHOLD_PX, GIZMO_PRIORITY};
use crate::scene::viewport::pointer::precision::{PrecisionCandidate, PrecisionShape};
use crate::scene::viewport::pointer::viewport_pointer_route::ViewportPointerRoute;
use crate::scene::viewport::projection::ViewportProjectionContext;

use super::projected_ring_segments;

pub(in crate::scene::viewport::pointer) fn scene_gizmo_candidate(
    owner: u64,
    shape: &OverlayPickShape,
    projection: &ViewportProjectionContext<'_>,
) -> Option<PrecisionCandidate> {
    match shape {
        OverlayPickShape::Sphere { center, radius } => {
            let projected = projection.projected_point(*center)?;
            let radius_px = (*radius / projection.world_units_per_pixel(*center))
                .abs()
                .clamp(10.0, 44.0);
            Some(PrecisionCandidate {
                route: ViewportPointerRoute::SceneGizmo { owner },
                priority: GIZMO_PRIORITY,
                shape: PrecisionShape::Circle {
                    center: projected.position,
                    radius_px,
                    threshold_px: GIZMO_PICK_THRESHOLD_PX,
                    depth: projected.depth,
                },
            })
        }
        OverlayPickShape::Segment {
            start,
            end,
            thickness,
        } => {
            let start_projection = projection.projected_point(*start)?;
            let end_projection = projection.projected_point(*end)?;
            let mid = (*start + *end) * 0.5;
            let thickness_px = (*thickness / projection.world_units_per_pixel(mid))
                .abs()
                .clamp(6.0, 20.0);
            Some(PrecisionCandidate {
                route: ViewportPointerRoute::SceneGizmo { owner },
                priority: GIZMO_PRIORITY,
                shape: PrecisionShape::Line {
                    start: start_projection.position,
                    end: end_projection.position,
                    radius_px: thickness_px,
                    threshold_px: GIZMO_PICK_THRESHOLD_PX,
                    depth: start_projection.depth.min(end_projection.depth),
                },
            })
        }
        OverlayPickShape::Circle {
            center,
            normal,
            radius,
            thickness,
        } => {
            let projected = projection.projected_point(*center)?;
            let ring_segments = projected_ring_segments(*center, *normal, *radius, projection);
            if ring_segments.is_empty() {
                return None;
            }
            let radius_px = (*radius / projection.world_units_per_pixel(*center))
                .abs()
                .max(1.0);
            let thickness_px = (*thickness / projection.world_units_per_pixel(*center))
                .abs()
                .clamp(6.0, 20.0);
            Some(PrecisionCandidate {
                route: ViewportPointerRoute::SceneGizmo { owner },
                priority: GIZMO_PRIORITY,
                shape: PrecisionShape::Ring {
                    segments: ring_segments,
                    radius_px,
                    thickness_px,
                    threshold_px: GIZMO_PICK_THRESHOLD_PX,
                    depth: projected.depth,
                },
            })
        }
    }
}
