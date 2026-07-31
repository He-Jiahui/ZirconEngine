use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use image::{imageops::FilterType, DynamicImage, ImageBuffer, ImageFormat, Rgba};

use crate::core::asset::ThumbnailPlaceholderPalette;
use zircon_runtime::asset::AssetUuid;

const MAX_PREVIEW_IN_FLIGHT: usize = 64;
static NEXT_PREVIEW_JOB_TOKEN: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PreviewJobToken(u64);

impl PreviewJobToken {
    fn next() -> Self {
        Self(NEXT_PREVIEW_JOB_TOKEN.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreviewArtifactKey {
    pub asset_uuid: AssetUuid,
    pub variant: String,
}

impl PreviewArtifactKey {
    pub fn thumbnail(asset_uuid: AssetUuid, source_hash: &str) -> Self {
        let source_hash = if source_hash.is_empty() {
            "unversioned"
        } else {
            source_hash
        };
        Self {
            asset_uuid,
            variant: format!("thumbnail-{source_hash}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreviewCache {
    root: PathBuf,
}

impl PreviewCache {
    pub fn new(cache_root: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = cache_root.as_ref().join("editor-previews");
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
        colors: ThumbnailPlaceholderPalette,
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

#[derive(Clone, Debug, Default)]
pub struct PreviewScheduler {
    dirty: HashSet<AssetUuid>,
    visible: HashSet<AssetUuid>,
    in_flight: HashMap<AssetUuid, PreviewJobToken>,
}

impl PreviewScheduler {
    pub fn mark_dirty(&mut self, asset_uuid: AssetUuid) {
        self.dirty.insert(asset_uuid);
    }

    pub(crate) fn request_refresh(
        &mut self,
        asset_uuid: AssetUuid,
        visible: bool,
    ) -> Option<PreviewJobToken> {
        if visible {
            self.visible.insert(asset_uuid);
        } else {
            self.visible.remove(&asset_uuid);
        }

        if !visible
            || self.in_flight.contains_key(&asset_uuid)
            || self.in_flight.len() >= MAX_PREVIEW_IN_FLIGHT
            || !self.dirty.remove(&asset_uuid)
        {
            return None;
        }
        let token = PreviewJobToken::next();
        self.in_flight.insert(asset_uuid, token);
        Some(token)
    }

    pub(crate) fn complete_refresh(
        &mut self,
        asset_uuid: AssetUuid,
        token: PreviewJobToken,
    ) -> bool {
        if self.in_flight.get(&asset_uuid) != Some(&token) {
            return false;
        }
        self.in_flight.remove(&asset_uuid);
        true
    }

    pub(crate) fn owns_refresh(&self, asset_uuid: AssetUuid, token: PreviewJobToken) -> bool {
        self.in_flight.get(&asset_uuid) == Some(&token)
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

#[cfg(test)]
mod tests {
    use super::{PreviewScheduler, MAX_PREVIEW_IN_FLIGHT};
    use zircon_runtime::asset::AssetUuid;

    #[test]
    fn preview_scheduler_bounds_in_flight_assets_without_implicitly_retrying_completion() {
        let mut scheduler = PreviewScheduler::default();
        let mut admitted = Vec::new();
        for _ in 0..MAX_PREVIEW_IN_FLIGHT {
            let uuid = AssetUuid::new();
            scheduler.mark_dirty(uuid);
            let token = scheduler.request_refresh(uuid, true).expect("admitted");
            admitted.push((uuid, token));
        }
        let waiting = AssetUuid::new();
        scheduler.mark_dirty(waiting);
        assert!(scheduler.request_refresh(waiting, true).is_none());

        assert!(scheduler.complete_refresh(admitted[0].0, admitted[0].1));
        let waiting_token = scheduler.request_refresh(waiting, true).expect("refill");
        assert!(scheduler.complete_refresh(waiting, waiting_token));
        assert!(scheduler.request_refresh(waiting, true).is_none());
        scheduler.mark_dirty(waiting);
        assert!(scheduler.request_refresh(waiting, true).is_some());
    }

    #[test]
    fn stale_job_token_cannot_release_new_generation_admission() {
        let asset_uuid = AssetUuid::new();
        let mut previous = PreviewScheduler::default();
        previous.mark_dirty(asset_uuid);
        let stale_token = previous
            .request_refresh(asset_uuid, true)
            .expect("old generation admission");

        let mut current = PreviewScheduler::default();
        current.mark_dirty(asset_uuid);
        let current_token = current
            .request_refresh(asset_uuid, true)
            .expect("new generation admission");

        assert!(!current.complete_refresh(asset_uuid, stale_token));
        assert!(current.owns_refresh(asset_uuid, current_token));
    }
}
