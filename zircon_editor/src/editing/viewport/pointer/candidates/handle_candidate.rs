use zircon_math::UVec2;
use zircon_scene::{HandleElementExtract, ViewportCameraSnapshot};

use crate::editing::viewport::pointer::constants::{HANDLE_PICK_THRESHOLD_PX, HANDLE_PRIORITY};
use crate::editing::viewport::pointer::precision::{PrecisionCandidate, PrecisionShape};
use crate::editing::viewport::pointer::viewport_pointer_route::ViewportPointerRoute;
use crate::editing::viewport::projection::{projected_point, world_units_per_pixel};

use super::{gizmo_axis, projected_ring_segments};

pub(in crate::editing::viewport::pointer) fn handle_candidate(
    owner: u64,
    element: &HandleElementExtract,
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
) -> Option<PrecisionCandidate> {
    match element {
        HandleElementExtract::AxisLine {
            axis, start, end, ..
        }
        | HandleElementExtract::AxisScale {
            axis, start, end, ..
        } => {
            let start_projection = projected_point(*start, camera, viewport)?;
            let end_projection = projected_point(*end, camera, viewport)?;
            Some(PrecisionCandidate {
                route: ViewportPointerRoute::HandleAxis {
                    owner,
                    axis: gizmo_axis(*axis),
                },
                priority: HANDLE_PRIORITY,
                shape: PrecisionShape::Line {
                    start: start_projection.position,
                    end: end_projection.position,
                    radius_px: 0.0,
                    threshold_px: HANDLE_PICK_THRESHOLD_PX,
                    depth: start_projection.depth.min(end_projection.depth),
                },
            })
        }
        HandleElementExtract::AxisRing {
            axis,
            center,
            normal,
            radius,
            ..
        } => {
            let center_projection = projected_point(*center, camera, viewport)?;
            let ring_segments =
                projected_ring_segments(*center, *normal, *radius, camera, viewport);
            if ring_segments.is_empty() {
                return None;
            }
            let radius_px = (*radius / world_units_per_pixel(camera, *center, viewport))
                .abs()
                .max(1.0);
            Some(PrecisionCandidate {
                route: ViewportPointerRoute::HandleAxis {
                    owner,
                    axis: gizmo_axis(*axis),
                },
                priority: HANDLE_PRIORITY,
                shape: PrecisionShape::Ring {
                    segments: ring_segments,
                    radius_px,
                    thickness_px: 0.0,
                    threshold_px: HANDLE_PICK_THRESHOLD_PX,
                    depth: center_projection.depth,
                },
            })
        }
        HandleElementExtract::CenterAnchor { .. } => None,
    }
}
