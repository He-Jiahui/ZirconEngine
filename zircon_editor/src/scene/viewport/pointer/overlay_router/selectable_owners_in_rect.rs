use indexmap::IndexSet;
use zircon_runtime_interface::math::Vec2;

use crate::scene::viewport::pointer::constants::{
    GIZMO_PICK_THRESHOLD_PX, RENDERABLE_PICK_MIN_RADIUS_PX,
};
use crate::scene::viewport::projection::ViewportProjectionContext;
use crate::scene::viewport::OverlayPickShape;

use super::ViewportOverlayPointerRouter;

impl ViewportOverlayPointerRouter {
    pub(crate) fn selectable_owners_in_rect(&self, start: Vec2, end: Vec2) -> Vec<u64> {
        let min = start.min(end);
        let max = start.max(end);
        let projection = ViewportProjectionContext::new(&self.layout.camera, self.layout.viewport);
        let mut owners = IndexSet::new();

        for candidate in self.renderable_candidates.iter() {
            let Some(projected) = projection.projected_point(candidate.position) else {
                continue;
            };
            let radius_px = (candidate.radius_world
                / projection.world_units_per_pixel(candidate.position))
            .abs()
            .max(RENDERABLE_PICK_MIN_RADIUS_PX);
            if circle_intersects_rect(projected.position, radius_px, min, max) {
                owners.insert(candidate.owner);
            }
        }

        for gizmo in self.layout.scene_gizmos.iter() {
            if gizmo
                .pick_shapes
                .iter()
                .any(|shape| pick_shape_intersects_rect(shape, &projection, min, max))
            {
                owners.insert(gizmo.owner);
            }
        }

        owners.into_iter().collect()
    }
}

fn pick_shape_intersects_rect(
    shape: &OverlayPickShape,
    projection: &ViewportProjectionContext,
    min: Vec2,
    max: Vec2,
) -> bool {
    match shape {
        OverlayPickShape::Sphere { center, radius } => {
            let Some(projected) = projection.projected_point(*center) else {
                return false;
            };
            let radius_px = (*radius / projection.world_units_per_pixel(*center))
                .abs()
                .clamp(10.0, 44.0)
                + GIZMO_PICK_THRESHOLD_PX;
            circle_intersects_rect(projected.position, radius_px, min, max)
        }
        OverlayPickShape::Segment {
            start,
            end,
            thickness,
        } => {
            let Some(start_projection) = projection.projected_point(*start) else {
                return false;
            };
            let Some(end_projection) = projection.projected_point(*end) else {
                return false;
            };
            let mid = (*start + *end) * 0.5;
            let radius_px = (*thickness / projection.world_units_per_pixel(mid))
                .abs()
                .clamp(6.0, 20.0)
                + GIZMO_PICK_THRESHOLD_PX;
            segment_intersects_rect(
                start_projection.position,
                end_projection.position,
                min - Vec2::splat(radius_px),
                max + Vec2::splat(radius_px),
            )
        }
        OverlayPickShape::Circle {
            center,
            radius,
            thickness,
            ..
        } => {
            let Some(projected) = projection.projected_point(*center) else {
                return false;
            };
            let radius_px = (*radius / projection.world_units_per_pixel(*center))
                .abs()
                .max(1.0);
            let thickness_px = (*thickness / projection.world_units_per_pixel(*center))
                .abs()
                .clamp(6.0, 20.0);
            circle_intersects_rect(
                projected.position,
                radius_px + thickness_px + GIZMO_PICK_THRESHOLD_PX,
                min,
                max,
            )
        }
    }
}

fn circle_intersects_rect(center: Vec2, radius: f32, min: Vec2, max: Vec2) -> bool {
    let closest = center.clamp(min, max);
    center.distance_squared(closest) <= radius * radius
}

fn segment_intersects_rect(start: Vec2, end: Vec2, min: Vec2, max: Vec2) -> bool {
    let delta = end - start;
    let mut near = 0.0_f32;
    let mut far = 1.0_f32;
    for (direction, distance) in [
        (-delta.x, start.x - min.x),
        (delta.x, max.x - start.x),
        (-delta.y, start.y - min.y),
        (delta.y, max.y - start.y),
    ] {
        if direction.abs() <= f32::EPSILON {
            if distance < 0.0 {
                return false;
            }
            continue;
        }
        let ratio = distance / direction;
        if direction < 0.0 {
            near = near.max(ratio);
        } else {
            far = far.min(ratio);
        }
        if near > far {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_query_includes_intersecting_candidate_bounds() {
        assert!(circle_intersects_rect(
            Vec2::new(12.0, 10.0),
            3.0,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
        ));
        assert!(!circle_intersects_rect(
            Vec2::new(14.0, 10.0),
            3.0,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
        ));
    }

    #[test]
    fn segment_query_detects_crossing_and_rejects_separated_segments() {
        let min = Vec2::new(0.0, 0.0);
        let max = Vec2::new(10.0, 10.0);
        assert!(segment_intersects_rect(
            Vec2::new(-5.0, 5.0),
            Vec2::new(15.0, 5.0),
            min,
            max,
        ));
        assert!(!segment_intersects_rect(
            Vec2::new(-5.0, 15.0),
            Vec2::new(15.0, 15.0),
            min,
            max,
        ));
    }
}
