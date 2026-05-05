use serde::{Deserialize, Serialize};

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
    pub translation: UiPoint,
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
    pub translation: UiPoint,
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

/// Retains the unsnapped layout frame separately from render bounds so hit/debug
/// code can keep using layout geometry after render-side pixel snapping.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiGeometry {
    pub local_size: UiSize,
    pub local_offset: UiPoint,
    pub layout_transform: UiLayoutTransform,
    pub render_transform: UiRenderTransform,
    pub absolute_frame: UiFrame,
    pub render_bounds: UiFrame,
    pub clip_frame: Option<UiFrame>,
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
