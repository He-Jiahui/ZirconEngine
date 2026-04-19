use crate::scene::viewport::{OverlayPickShape, ViewportCameraSnapshot};
use zircon_runtime::core::math::UVec2;

use crate::scene::viewport::pointer::constants::{GIZMO_PICK_THRESHOLD_PX, GIZMO_PRIORITY};
use crate::scene::viewport::pointer::precision::{PrecisionCandidate, PrecisionShape};
use crate::scene::viewport::pointer::viewport_pointer_route::ViewportPointerRoute;
use crate::scene::viewport::projection::{projected_point, world_units_per_pixel};

use super::projected_ring_segments;

pub(in crate::scene::viewport::pointer) fn scene_gizmo_candidate(
    owner: u64,
    shape: &OverlayPickShape,
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
) -> Option<PrecisionCandidate> {
    match shape {
        OverlayPickShape::Sphere { center, radius } => {
            let projection = projected_point(*center, camera, viewport)?;
            let radius_px = (*radius / world_units_per_pixel(camera, *center, viewport))
                .abs()
                .clamp(10.0, 44.0);
            Some(PrecisionCandidate {
                route: ViewportPointerRoute::SceneGizmo { owner },
                priority: GIZMO_PRIORITY,
                shape: PrecisionShape::Circle {
                    center: projection.position,
                    radius_px,
                    threshold_px: GIZMO_PICK_THRESHOLD_PX,
                    depth: projection.depth,
                },
            })
        }
        OverlayPickShape::Segment {
            start,
            end,
            thickness,
        } => {
            let start_projection = projected_point(*start, camera, viewport)?;
            let end_projection = projected_point(*end, camera, viewport)?;
            let mid = (*start + *end) * 0.5;
            let thickness_px = (*thickness / world_units_per_pixel(camera, mid, viewport))
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
            let projection = projected_point(*center, camera, viewport)?;
            let ring_segments =
                projected_ring_segments(*center, *normal, *radius, camera, viewport);
            if ring_segments.is_empty() {
                return None;
            }
            let radius_px = (*radius / world_units_per_pixel(camera, *center, viewport))
                .abs()
                .max(1.0);
            let thickness_px = (*thickness / world_units_per_pixel(camera, *center, viewport))
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
                    depth: projection.depth,
                },
            })
        }
    }
}
