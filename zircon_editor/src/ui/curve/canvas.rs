use super::{CurveBounds, CurvePoint};

const MIN_AXIS_SPAN: f32 = 1.0e-6;
const MIN_VIEWPORT_EXTENT: f32 = 1.0;

/// Converts between domain-owned curve coordinates and a renderer's local pixel coordinates.
///
/// Values use an upward-positive curve axis while screen pixels remain downward-positive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveCanvasTransform {
    bounds: CurveBounds,
    viewport: CurvePoint,
}

impl CurveCanvasTransform {
    pub fn new(bounds: CurveBounds, viewport: CurvePoint) -> Self {
        Self {
            bounds,
            viewport: CurvePoint::new(finite_extent(viewport.time), finite_extent(viewport.value)),
        }
    }

    pub fn bounds(self) -> CurveBounds {
        self.bounds
    }

    pub fn viewport(self) -> CurvePoint {
        self.viewport
    }

    pub fn curve_to_screen(self, point: CurvePoint) -> CurvePoint {
        CurvePoint::new(
            (point.time - self.bounds.time_start) / self.time_span() * self.viewport.time,
            (self.bounds.value_max - point.value) / self.value_span() * self.viewport.value,
        )
    }

    pub fn screen_to_curve(self, point: CurvePoint) -> CurvePoint {
        CurvePoint::new(
            point.time / self.viewport.time * self.time_span() + self.bounds.time_start,
            self.bounds.value_max - point.value / self.viewport.value * self.value_span(),
        )
    }

    fn time_span(self) -> f32 {
        self.bounds.time_duration().max(MIN_AXIS_SPAN)
    }

    fn value_span(self) -> f32 {
        self.bounds.value_span().max(MIN_AXIS_SPAN)
    }
}

fn finite_extent(value: f32) -> f32 {
    value
        .is_finite()
        .then_some(value.max(MIN_VIEWPORT_EXTENT))
        .unwrap_or(MIN_VIEWPORT_EXTENT)
}
