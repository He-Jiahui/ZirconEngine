use serde::{Deserialize, Serialize};

use super::metrics::UiLayoutMetrics;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiPoint {
    pub x: f32,
    pub y: f32,
}

impl UiPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSize {
    pub width: f32,
    pub height: f32,
}

impl UiSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiFrame {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(self) -> f32 {
        self.y + self.height
    }

    pub fn center(self) -> UiPoint {
        UiPoint::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn contains_point(self, point: UiPoint) -> bool {
        self.width > 0.0
            && self.height > 0.0
            && point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        let width = right - left;
        let height = bottom - top;
        (width > 0.0 && height > 0.0).then_some(Self::new(left, top, width, height))
    }

    pub fn translated(self, translation: UiPoint) -> Self {
        Self::new(
            self.x + translation.x,
            self.y + translation.y,
            self.width,
            self.height,
        )
    }

    pub fn scaled(self, scale: UiPoint) -> Self {
        Self::new(
            self.x * scale.x,
            self.y * scale.y,
            self.width * scale.x,
            self.height * scale.y,
        )
    }

    pub fn apply_layout_transform(self, transform: UiLayoutTransform) -> Self {
        self.scaled(transform.scale)
            .translated(transform.translation)
    }

    pub fn apply_render_transform(self, transform: UiRenderTransform) -> Self {
        self.scaled(transform.scale)
            .translated(transform.translation)
    }

    pub fn pixel_snapped(self, dpi_scale: f32) -> Self {
        if !frame_is_finite(self) {
            return self;
        }
        let scale = sanitized_metric_scale(dpi_scale);
        let left = snap_floor(self.x, scale);
        let top = snap_floor(self.y, scale);
        let right = snap_ceil(self.right(), scale);
        let bottom = snap_ceil(self.bottom(), scale);
        Self::new(left, top, (right - left).max(0.0), (bottom - top).max(0.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiPixelSnapping {
    Disabled,
    #[default]
    Enabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiLayoutTransform {
    #[serde(default)]
    pub translation: UiPoint,
    #[serde(default = "default_transform_scale")]
    pub scale: UiPoint,
}

impl Default for UiLayoutTransform {
    fn default() -> Self {
        Self::identity()
    }
}

impl UiLayoutTransform {
    pub const fn identity() -> Self {
        Self {
            translation: UiPoint::new(0.0, 0.0),
            scale: UiPoint::new(1.0, 1.0),
        }
    }

    pub const fn new(translation: UiPoint, scale: UiPoint) -> Self {
        Self { translation, scale }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderTransform {
    #[serde(default)]
    pub translation: UiPoint,
    #[serde(default = "default_transform_scale")]
    pub scale: UiPoint,
}

impl Default for UiRenderTransform {
    fn default() -> Self {
        Self::identity()
    }
}

impl UiRenderTransform {
    pub const fn identity() -> Self {
        Self {
            translation: UiPoint::new(0.0, 0.0),
            scale: UiPoint::new(1.0, 1.0),
        }
    }

    pub const fn new(translation: UiPoint, scale: UiPoint) -> Self {
        Self { translation, scale }
    }
}

const fn default_transform_scale() -> UiPoint {
    UiPoint::new(1.0, 1.0)
}

/// Retains the unsnapped layout frame separately from render bounds so hit/debug
/// code can keep using layout geometry after render-side pixel snapping.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiGeometry {
    #[serde(default)]
    pub local_size: UiSize,
    #[serde(default)]
    pub local_offset: UiPoint,
    #[serde(default)]
    pub layout_transform: UiLayoutTransform,
    #[serde(default)]
    pub render_transform: UiRenderTransform,
    #[serde(default)]
    pub absolute_frame: UiFrame,
    #[serde(default)]
    pub render_bounds: UiFrame,
    #[serde(default)]
    pub clip_frame: Option<UiFrame>,
    #[serde(default)]
    pub pixel_snapping: UiPixelSnapping,
}

impl Default for UiGeometry {
    fn default() -> Self {
        Self::from_frame(UiFrame::default())
    }
}

impl UiGeometry {
    pub const fn from_frame(frame: UiFrame) -> Self {
        Self {
            local_size: UiSize::new(frame.width, frame.height),
            local_offset: UiPoint::new(0.0, 0.0),
            layout_transform: UiLayoutTransform::identity(),
            render_transform: UiRenderTransform::identity(),
            absolute_frame: frame,
            render_bounds: frame,
            clip_frame: None,
            pixel_snapping: UiPixelSnapping::Enabled,
        }
    }

    pub fn from_frame_with_metrics(frame: UiFrame, metrics: UiLayoutMetrics) -> Self {
        let dpi_scale = sanitized_metric_scale(metrics.dpi_scale);
        let layout_scale = sanitized_metric_scale(metrics.layout_scale);
        let render_bounds = if metrics.pixel_snapping == UiPixelSnapping::Enabled {
            frame.pixel_snapped(dpi_scale)
        } else {
            frame
        };
        Self {
            local_size: UiSize::new(frame.width, frame.height),
            local_offset: UiPoint::new(0.0, 0.0),
            layout_transform: UiLayoutTransform::new(
                UiPoint::new(0.0, 0.0),
                UiPoint::new(layout_scale, layout_scale),
            ),
            render_transform: UiRenderTransform::new(
                UiPoint::new(0.0, 0.0),
                UiPoint::new(dpi_scale, dpi_scale),
            ),
            absolute_frame: frame,
            render_bounds,
            clip_frame: None,
            pixel_snapping: metrics.pixel_snapping,
        }
    }
}

fn frame_is_finite(frame: UiFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
}

fn sanitized_metric_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

fn snap_floor(value: f32, scale: f32) -> f32 {
    (value * scale).floor() / scale
}

fn snap_ceil(value: f32, scale: f32) -> f32 {
    (value * scale).ceil() / scale
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Pivot {
    pub x: f32,
    pub y: f32,
}

impl Pivot {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Anchor {
    pub x: f32,
    pub y: f32,
}

impl Anchor {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
