//! 图集资产把源纹理、像素矩形和归一化 UV 绑定到同一条目；UI/渲染消费 UV 前需通过整体验证，防止读到越界或不一致的区域。

use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

use super::validation::SpriteAtlasValidationError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteAtlasAsset {
    pub atlas_texture: AssetUri,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub padding: SpriteAtlasPadding,
    #[serde(default)]
    pub entries: Vec<SpriteAtlasEntry>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAtlasPadding {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteAtlasEntry {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<AssetUri>,
    pub pixel_rect: SpriteAtlasRect,
    pub uv_rect: SpriteAtlasUvRect,
    pub source_width: u32,
    pub source_height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAtlasRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteAtlasUvRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl SpriteAtlasUvRect {
    pub fn from_pixel_rect(
        rect: SpriteAtlasRect,
        atlas_width: u32,
        atlas_height: u32,
    ) -> Result<Self, SpriteAtlasValidationError> {
        if atlas_width == 0 || atlas_height == 0 {
            return Err(SpriteAtlasValidationError::ZeroAtlasDimensions {
                width: atlas_width,
                height: atlas_height,
            });
        }
        if rect.width == 0 || rect.height == 0 {
            return Err(SpriteAtlasValidationError::ZeroEntryDimensions {
                name: None,
                width: rect.width,
                height: rect.height,
            });
        }

        let rect_max_x = rect.x.checked_add(rect.width).ok_or(
            SpriteAtlasValidationError::PixelRectOutOfBounds {
                name: None,
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                atlas_width,
                atlas_height,
            },
        )?;
        let rect_max_y = rect.y.checked_add(rect.height).ok_or(
            SpriteAtlasValidationError::PixelRectOutOfBounds {
                name: None,
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                atlas_width,
                atlas_height,
            },
        )?;
        if rect_max_x > atlas_width || rect_max_y > atlas_height {
            return Err(SpriteAtlasValidationError::PixelRectOutOfBounds {
                name: None,
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                atlas_width,
                atlas_height,
            });
        }

        let inverse_atlas_width = 1.0 / atlas_width as f32;
        let inverse_atlas_height = 1.0 / atlas_height as f32;
        Ok(Self {
            min: [
                rect.x as f32 * inverse_atlas_width,
                rect.y as f32 * inverse_atlas_height,
            ],
            max: [
                rect_max_x as f32 * inverse_atlas_width,
                rect_max_y as f32 * inverse_atlas_height,
            ],
        })
    }
}

#[cfg(test)]
#[path = "layout/tests/reciprocal_uv_tests.rs"]
mod reciprocal_uv_tests;

#[cfg(test)]
#[path = "tests/layout.rs"]
mod tests;
