use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use image::{imageops::FilterType, DynamicImage, ImageBuffer, ImageFormat, Rgba};

use zircon_runtime::asset::AssetUuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreviewArtifactKey {
    pub asset_uuid: AssetUuid,
    pub variant: String,
}

impl PreviewArtifactKey {
    pub fn thumbnail(asset_uuid: AssetUuid) -> Self {
        Self {
            asset_uuid,
            variant: "thumbnail".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreviewCache {
    root: PathBuf,
}

impl PreviewCache {
    pub fn new(library_root: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = library_root.as_ref().join("editor-previews");
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn path_for(&self, key: &PreviewArtifactKey) -> PathBuf {
        self.root
            .join(format!("{}-{}.png", key.asset_uuid, key.variant))
    }

    pub fn write_thumbnail(
        &self,
        key: &PreviewArtifactKey,
        image: &DynamicImage,
    ) -> Result<PathBuf, std::io::Error> {
        let path = self.path_for(key);
        image
            .thumbnail_exact(192, 192)
            .save_with_format(&path, ImageFormat::Png)
            .map_err(invalid_data)?;
        Ok(path)
    }

    pub fn write_kind_placeholder(
        &self,
        key: &PreviewArtifactKey,
        colors: PreviewPalette,
    ) -> Result<PathBuf, std::io::Error> {
        let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(256, 160);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let fx = x as f32 / 255.0;
            let fy = y as f32 / 159.0;
            let t = ((fx * 0.7) + (fy * 0.3)).clamp(0.0, 1.0);
            *pixel = blend(colors.primary, colors.secondary, t);
        }

        for y in 0..160_u32 {
            let stripe = ((y / 12) % 2) == 0;
            if stripe {
                for x in 0..256_u32 {
                    let pixel = image.get_pixel_mut(x, y);
                    pixel[0] = pixel[0].saturating_add(colors.accent[0] / 5);
                    pixel[1] = pixel[1].saturating_add(colors.accent[1] / 5);
                    pixel[2] = pixel[2].saturating_add(colors.accent[2] / 5);
                }
            }
        }

        for x in 18..238_u32 {
            for y in 112..134_u32 {
                image.put_pixel(x, y, Rgba(colors.banner));
            }
        }

        let path = self.path_for(key);
        DynamicImage::ImageRgba8(image)
            .resize_exact(192, 192, FilterType::Triangle)
            .save_with_format(&path, ImageFormat::Png)
            .map_err(invalid_data)?;
        Ok(path)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PreviewPalette {
    pub primary: [u8; 4],
    pub secondary: [u8; 4],
    pub accent: [u8; 4],
    pub banner: [u8; 4],
}

#[derive(Clone, Debug, Default)]
pub struct PreviewScheduler {
    dirty: HashSet<AssetUuid>,
    visible: HashSet<AssetUuid>,
}

impl PreviewScheduler {
    pub fn mark_dirty(&mut self, asset_uuid: AssetUuid) {
        self.dirty.insert(asset_uuid);
    }

    pub fn request_refresh(&mut self, asset_uuid: AssetUuid, visible: bool) -> bool {
        if visible {
            self.visible.insert(asset_uuid);
        } else {
            self.visible.remove(&asset_uuid);
        }

        visible && self.dirty.remove(&asset_uuid)
    }
}

fn blend(left: [u8; 4], right: [u8; 4], t: f32) -> Rgba<u8> {
    let lerp = |a: u8, b: u8| -> u8 { (((a as f32) * (1.0 - t)) + ((b as f32) * t)).round() as u8 };
    Rgba([
        lerp(left[0], right[0]),
        lerp(left[1], right[1]),
        lerp(left[2], right[2]),
        lerp(left[3], right[3]),
    ])
}

fn invalid_data(error: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}
