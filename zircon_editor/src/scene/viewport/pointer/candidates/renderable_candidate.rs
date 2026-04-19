use crate::scene::viewport::ViewportCameraSnapshot;
use zircon_runtime::core::math::UVec2;

use crate::scene::viewport::pointer::constants::{
    RENDERABLE_PICK_MIN_RADIUS_PX, RENDERABLE_PRIORITY,
};
use crate::scene::viewport::pointer::precision::{PrecisionCandidate, PrecisionShape};
use crate::scene::viewport::pointer::{
    viewport_pointer_route::ViewportPointerRoute,
    viewport_renderable_pick_candidate::ViewportRenderablePickCandidate,
};
use crate::scene::viewport::projection::{projected_point, world_units_per_pixel};

pub(in crate::scene::viewport::pointer) fn renderable_candidate(
    candidate: &ViewportRenderablePickCandidate,
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
) -> Option<PrecisionCandidate> {
    let projection = projected_point(candidate.position, camera, viewport)?;
    let radius_px = (candidate.radius_world
        / world_units_per_pixel(camera, candidate.position, viewport))
    .abs()
    .max(RENDERABLE_PICK_MIN_RADIUS_PX);
    Some(PrecisionCandidate {
        route: ViewportPointerRoute::Renderable {
            owner: candidate.owner,
        },
        priority: RENDERABLE_PRIORITY,
        shape: PrecisionShape::Circle {
            center: projection.position,
            radius_px,
            threshold_px: 0.0,
            depth: projection.depth,
        },
    })
}
