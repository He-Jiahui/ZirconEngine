use std::fmt;
use std::num::NonZeroU32;

use super::DisplayTopologyError;

/// Physical desktop-space rectangle in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayPhysicalRect {
    x: i32,
    y: i32,
    width: NonZeroU32,
    height: NonZeroU32,
}

impl DisplayPhysicalRect {
    pub const fn new(x: i32, y: i32, width: NonZeroU32, height: NonZeroU32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn x(self) -> i32 {
        self.x
    }

    pub const fn y(self) -> i32 {
        self.y
    }

    pub const fn width(self) -> u32 {
        self.width.get()
    }

    pub const fn height(self) -> u32 {
        self.height.get()
    }
}

/// Logical desktop-space rectangle. A backend must provide finite, positive
/// extents rather than allowing a default or clamped value to masquerade as an
/// observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayLogicalRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl DisplayLogicalRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, DisplayTopologyError> {
        if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
            return Err(DisplayTopologyError::NonFiniteLogicalGeometry);
        }
        if width <= 0.0 || height <= 0.0 {
            return Err(DisplayTopologyError::NonPositiveLogicalExtent { width, height });
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    pub const fn x(self) -> f64 {
        self.x
    }

    pub const fn y(self) -> f64 {
        self.y
    }

    pub const fn width(self) -> f64 {
        self.width
    }

    pub const fn height(self) -> f64 {
        self.height
    }
}

/// Logical safe-area insets when the backend can observe them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayLogicalInsets {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl DisplayLogicalInsets {
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Result<Self, DisplayTopologyError> {
        if !left.is_finite() || !top.is_finite() || !right.is_finite() || !bottom.is_finite() {
            return Err(DisplayTopologyError::NonFiniteSafeAreaInsets);
        }
        if left < 0.0 || top < 0.0 || right < 0.0 || bottom < 0.0 {
            return Err(DisplayTopologyError::NegativeSafeAreaInsets);
        }
        Ok(Self {
            left,
            top,
            right,
            bottom,
        })
    }

    pub const fn left(self) -> f64 {
        self.left
    }

    pub const fn top(self) -> f64 {
        self.top
    }

    pub const fn right(self) -> f64 {
        self.right
    }

    pub const fn bottom(self) -> f64 {
        self.bottom
    }
}

/// Physical panel orientation as reported by the host backend.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayOrientation {
    #[default]
    Unknown,
    Landscape,
    Portrait,
    LandscapeFlipped,
    PortraitFlipped,
}

impl fmt::Display for DisplayOrientation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Unknown => "unknown",
            Self::Landscape => "landscape",
            Self::Portrait => "portrait",
            Self::LandscapeFlipped => "landscape_flipped",
            Self::PortraitFlipped => "portrait_flipped",
        };
        formatter.write_str(value)
    }
}
