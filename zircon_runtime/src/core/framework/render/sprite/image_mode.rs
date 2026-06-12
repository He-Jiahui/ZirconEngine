use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum RenderSpriteImageMode {
    #[default]
    Stretch,
    Scale(RenderSpriteScalingMode),
    Tiled {
        tile_x: bool,
        tile_y: bool,
        stretch_value: f32,
    },
    Sliced(RenderSpriteSlicer),
}

impl RenderSpriteImageMode {
    pub const fn scale(scaling_mode: RenderSpriteScalingMode) -> Self {
        Self::Scale(scaling_mode)
    }

    pub const fn tiled(tile_x: bool, tile_y: bool, stretch_value: f32) -> Self {
        Self::Tiled {
            tile_x,
            tile_y,
            stretch_value,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum RenderSpriteScalingMode {
    #[default]
    FillCenter,
    FillStart,
    FillEnd,
    FitCenter,
    FitStart,
    FitEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderSpriteSlicer {
    pub border: RenderSpriteSliceBorder,
    pub center_scale_mode: RenderSpriteSliceScaleMode,
    pub sides_scale_mode: RenderSpriteSliceScaleMode,
    pub max_corner_scale: f32,
}

impl RenderSpriteSlicer {
    pub const fn new(border: RenderSpriteSliceBorder) -> Self {
        Self {
            border,
            center_scale_mode: RenderSpriteSliceScaleMode::Stretch,
            sides_scale_mode: RenderSpriteSliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }
    }
}

impl Default for RenderSpriteSlicer {
    fn default() -> Self {
        Self::new(RenderSpriteSliceBorder::default())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderSpriteSliceBorder {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl RenderSpriteSliceBorder {
    pub const ZERO: Self = Self {
        left: 0.0,
        right: 0.0,
        top: 0.0,
        bottom: 0.0,
    };

    pub const fn all(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    pub const fn horizontal_vertical(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }
}

impl Default for RenderSpriteSliceBorder {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum RenderSpriteSliceScaleMode {
    #[default]
    Stretch,
    Tile {
        stretch_value: f32,
    },
}
