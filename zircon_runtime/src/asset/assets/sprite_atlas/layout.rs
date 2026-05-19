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

        let atlas_width = atlas_width as f32;
        let atlas_height = atlas_height as f32;
        Ok(Self {
            min: [rect.x as f32 / atlas_width, rect.y as f32 / atlas_height],
            max: [
                rect_max_x as f32 / atlas_width,
                rect_max_y as f32 / atlas_height,
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_atlas_uv_rect_derives_from_pixel_rect() {
        let uv = SpriteAtlasUvRect::from_pixel_rect(
            SpriteAtlasRect {
                x: 16,
                y: 8,
                width: 32,
                height: 16,
            },
            128,
            64,
        )
        .expect("uv rect should derive from atlas extent");

        assert_eq!(uv.min, [0.125, 0.125]);
        assert_eq!(uv.max, [0.375, 0.375]);
    }

    #[test]
    fn sprite_atlas_uv_rect_rejects_zero_atlas_extent() {
        let error = SpriteAtlasUvRect::from_pixel_rect(
            SpriteAtlasRect {
                x: 0,
                y: 0,
                width: 16,
                height: 16,
            },
            0,
            64,
        )
        .expect_err("zero atlas width must be rejected");

        assert_eq!(
            error,
            SpriteAtlasValidationError::ZeroAtlasDimensions {
                width: 0,
                height: 64
            }
        );
    }

    #[test]
    fn sprite_atlas_uv_rect_rejects_zero_pixel_extent() {
        let error = SpriteAtlasUvRect::from_pixel_rect(
            SpriteAtlasRect {
                x: 0,
                y: 0,
                width: 0,
                height: 16,
            },
            64,
            64,
        )
        .expect_err("zero pixel width must be rejected");

        assert_eq!(
            error,
            SpriteAtlasValidationError::ZeroEntryDimensions {
                name: None,
                width: 0,
                height: 16,
            }
        );
    }
}
