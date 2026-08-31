use crate::scene::viewport::pointer::constants::{
    RENDERABLE_PICK_MIN_RADIUS_PX, RENDERABLE_PRIORITY,
};
use crate::scene::viewport::pointer::precision::{PrecisionCandidate, PrecisionShape};
use crate::scene::viewport::pointer::{
    viewport_pointer_route::ViewportPointerRoute,
    viewport_renderable_pick_candidate::ViewportRenderablePickCandidate,
};
use crate::scene::viewport::projection::ViewportProjectionContext;

pub(in crate::scene::viewport::pointer) fn renderable_candidate(
    candidate: &ViewportRenderablePickCandidate,
    projection: &ViewportProjectionContext,
) -> Option<PrecisionCandidate> {
    let projected = projection.projected_point(candidate.position)?;
    let radius_px = (candidate.radius_world / projection.world_units_per_pixel(candidate.position))
        .abs()
        .max(RENDERABLE_PICK_MIN_RADIUS_PX);
    Some(PrecisionCandidate {
        route: ViewportPointerRoute::Renderable {
            owner: candidate.owner,
        },
        priority: RENDERABLE_PRIORITY,
        shape: PrecisionShape::Circle {
            center: projected.position,
            radius_px,
            threshold_px: 0.0,
            depth: projected.depth,
        },
    })
}
