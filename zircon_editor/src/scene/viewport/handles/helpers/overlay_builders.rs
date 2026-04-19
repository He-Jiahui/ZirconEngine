use zircon_framework::render::{HandleElementExtract, OverlayAxis};
use zircon_math::Vec3;

use crate::scene::viewport::handles::{constants::CENTER_COLOR, handle_basis::HandleBasis};

use super::axis_color;

pub(in crate::scene::viewport::handles) fn center_anchor(
    basis: &HandleBasis,
) -> HandleElementExtract {
    HandleElementExtract::CenterAnchor {
        position: basis.origin.translation,
        size: basis.extent * 0.08,
        color: CENTER_COLOR,
    }
}

pub(in crate::scene::viewport::handles) fn push_axis_line(
    elements: &mut Vec<HandleElementExtract>,
    axis: OverlayAxis,
    origin: Vec3,
    direction: Vec3,
    extent: f32,
) {
    elements.push(HandleElementExtract::AxisLine {
        axis,
        start: origin,
        end: origin + direction.normalize_or_zero() * extent,
        color: axis_color(axis),
        pick_radius: 0.12,
    });
}

pub(in crate::scene::viewport::handles) fn push_axis_ring(
    elements: &mut Vec<HandleElementExtract>,
    axis: OverlayAxis,
    center: Vec3,
    normal: Vec3,
    radius: f32,
) {
    elements.push(HandleElementExtract::AxisRing {
        axis,
        center,
        normal: normal.normalize_or_zero(),
        radius,
        color: axis_color(axis),
        pick_radius: 0.14,
    });
}

pub(in crate::scene::viewport::handles) fn push_axis_scale(
    elements: &mut Vec<HandleElementExtract>,
    axis: OverlayAxis,
    origin: Vec3,
    direction: Vec3,
    extent: f32,
) {
    elements.push(HandleElementExtract::AxisScale {
        axis,
        start: origin,
        end: origin + direction.normalize_or_zero() * extent,
        color: axis_color(axis),
        pick_radius: 0.12,
        handle_size: extent * 0.09,
    });
}
