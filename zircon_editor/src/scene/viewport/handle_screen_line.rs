use zircon_runtime_interface::math::{Vec2, Vec4};

use super::GizmoAxis;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HandleScreenLine {
    start: Vec2,
    end: Vec2,
    color: Vec4,
    width: f32,
    axis: Option<GizmoAxis>,
}

impl HandleScreenLine {
    pub(crate) const fn new(
        start: Vec2,
        end: Vec2,
        color: Vec4,
        width: f32,
        axis: Option<GizmoAxis>,
    ) -> Self {
        Self {
            start,
            end,
            color,
            width,
            axis,
        }
    }

    pub(crate) const fn start(self) -> Vec2 {
        self.start
    }

    pub(crate) const fn end(self) -> Vec2 {
        self.end
    }

    pub(crate) const fn color(self) -> Vec4 {
        self.color
    }

    pub(crate) const fn width(self) -> f32 {
        self.width
    }

    pub(crate) const fn axis(self) -> Option<GizmoAxis> {
        self.axis
    }

    pub(crate) fn is_finite(self) -> bool {
        self.start.is_finite()
            && self.end.is_finite()
            && self.color.is_finite()
            && self.width.is_finite()
            && self.width > 0.0
    }
}
