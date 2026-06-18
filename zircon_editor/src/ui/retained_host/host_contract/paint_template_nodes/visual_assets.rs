use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

use self::candidates::{icon_candidates, image_candidates, template_image_candidates};
use self::loading::{load_pixels_from_candidates, missing_icon_pixels};
use super::super::paint_frame::HostPaintAtlasImage;
use super::super::paint_theme::PALETTE;

mod candidates;
mod loading;
mod mui_icons;
mod svg;

const ICON_TINT: [u8; 4] = PALETTE.text;
const ICON_TINT_ACTIVE: [u8; 4] = PALETTE.focus_ring;
const ICON_TINT_DISABLED: [u8; 4] = PALETTE.text_disabled;
const ICON_TINT_ERROR: [u8; 4] = PALETTE.error;
const ICON_TINT_WARNING: [u8; 4] = PALETTE.warning;
const MAX_VECTOR_RASTER_EDGE: u32 = 4096;
const MUI_ICON_DEFAULT_EDGE: u32 = 24;

#[derive(Clone)]
pub(super) struct HostPaintImagePixels {
    pub(super) resource_key: String,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) rgba: Vec<u8>,
    pub(super) atlas: Option<HostPaintAtlasImage>,
}

impl HostPaintImagePixels {
    fn with_resource_key(mut self, resource_key: impl Into<String>) -> Self {
        self.resource_key = resource_key.into();
        self
    }

    fn with_atlas(mut self, atlas: Option<HostPaintAtlasImage>) -> Self {
        self.atlas = atlas;
        self
    }
}

#[derive(Clone, Copy)]
struct RasterTargetSize {
    width: u32,
    height: u32,
}

pub(super) fn retained_image_pixels(
    image: &crate::ui::retained_host::primitives::Image,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let buffer = image.to_rgba8()?;
    let mut rgba = buffer.as_bytes().to_vec();
    if let Some(tint) = tint {
        tint_non_transparent_pixels(&mut rgba, tint);
    }
    let image = HostPaintImagePixels {
        resource_key: retained_image_resource_key(buffer.width(), buffer.height(), &rgba),
        width: buffer.width(),
        height: buffer.height(),
        rgba,
        atlas: None,
    };
    image.is_valid().then_some(image)
}

pub(super) fn raster_size_from_frame(width: f32, height: f32) -> Option<(u32, u32)> {
    let target = RasterTargetSize::from_frame(width, height)?;
    Some((target.width, target.height))
}

pub(super) fn template_image_tint(
    is_icon_like: bool,
    active: bool,
    disabled: bool,
    text_tone: &str,
    validation_level: &str,
    style_tint: Option<[u8; 4]>,
) -> Option<[u8; 4]> {
    if !is_icon_like {
        return None;
    }
    if disabled {
        return Some(ICON_TINT_DISABLED);
    }
    if validation_level.eq_ignore_ascii_case("error") || text_tone.eq_ignore_ascii_case("error") {
        return Some(ICON_TINT_ERROR);
    }
    if validation_level.eq_ignore_ascii_case("warning") || text_tone.eq_ignore_ascii_case("warning")
    {
        return Some(ICON_TINT_WARNING);
    }
    if let Some(style_tint) = style_tint {
        return Some(style_tint);
    }
    if active {
        return Some(ICON_TINT_ACTIVE);
    }
    Some(ICON_TINT)
}

pub(super) fn template_image_pixels(
    preview_image: &crate::ui::retained_host::primitives::Image,
    media_source: &str,
    icon_name: &str,
    target_width: u32,
    target_height: u32,
    tint: Option<[u8; 4]>,
    prefer_preview_image: bool,
) -> Option<HostPaintImagePixels> {
    let target = RasterTargetSize::new(target_width, target_height);
    let key = template_image_cache_key(media_source, icon_name);
    let source_pixels = || {
        load_pixels_from_candidates(
            template_image_candidates(media_source, icon_name),
            &key,
            target,
            tint,
        )
    };
    let preview_pixels = || retained_image_pixels(preview_image, tint);
    let pixels = if prefer_preview_image {
        preview_pixels().or_else(source_pixels)
    } else {
        source_pixels().or_else(preview_pixels)
    };
    pixels.or_else(|| {
        (!icon_name.trim().is_empty())
            .then_some(())
            .and_then(|_| target)
            .and_then(|target| missing_icon_pixels(&key, target, tint))
    })
}

pub(super) fn load_visual_asset_pixels(asset: &UiVisualAssetRef) -> Option<HostPaintImagePixels> {
    load_visual_asset_pixels_for_target(asset, None)
}

pub(super) fn load_visual_asset_pixels_for_size(
    asset: &UiVisualAssetRef,
    target_width: u32,
    target_height: u32,
) -> Option<HostPaintImagePixels> {
    load_visual_asset_pixels_for_target(asset, RasterTargetSize::new(target_width, target_height))
}

fn load_visual_asset_pixels_for_target(
    asset: &UiVisualAssetRef,
    target: Option<RasterTargetSize>,
) -> Option<HostPaintImagePixels> {
    let key = visual_asset_cache_key(asset);
    match asset {
        UiVisualAssetRef::Icon(icon_name) => {
            let target = target.unwrap_or(RasterTargetSize {
                width: MUI_ICON_DEFAULT_EDGE,
                height: MUI_ICON_DEFAULT_EDGE,
            });
            load_pixels_from_candidates(
                icon_candidates(icon_name),
                &key,
                Some(target),
                Some(ICON_TINT),
            )
            .or_else(|| missing_icon_pixels(&key, target, Some(ICON_TINT)))
        }
        UiVisualAssetRef::Image(source) => {
            load_pixels_from_candidates(image_candidates(source), &key, target, None)
        }
    }
}

fn visual_asset_cache_key(asset: &UiVisualAssetRef) -> String {
    match asset {
        UiVisualAssetRef::Icon(icon_name) => format!("icon:{icon_name}"),
        UiVisualAssetRef::Image(source) => format!("image:{source}"),
    }
}

fn template_image_cache_key(source: &str, icon_name: &str) -> String {
    if !icon_name.is_empty() {
        return format!("template-icon:{icon_name}");
    }
    format!("template-image:{source}")
}

fn retained_image_resource_key(width: u32, height: u32, rgba: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    rgba.hash(&mut hasher);
    format!("retained-image:{width}x{height}:{:016x}", hasher.finish())
}

fn tint_non_transparent_pixels(rgba: &mut [u8], tint: [u8; 4]) {
    for pixel in rgba.chunks_exact_mut(4) {
        if pixel[3] == 0 {
            continue;
        }
        pixel[0] = tint[0];
        pixel[1] = tint[1];
        pixel[2] = tint[2];
    }
}

impl HostPaintImagePixels {
    fn is_valid(&self) -> bool {
        !self.resource_key.is_empty()
            && self.width > 0
            && self.height > 0
            && self.rgba.len() == self.width as usize * self.height as usize * 4
    }
}

impl RasterTargetSize {
    fn new(width: u32, height: u32) -> Option<Self> {
        (width > 0 && height > 0).then_some(Self { width, height })
    }

    fn from_frame(width: f32, height: f32) -> Option<Self> {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return None;
        }
        Self::new(
            width.ceil().clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32,
            height.ceil().clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32,
        )
    }
}

#[cfg(test)]
#[path = "visual_assets_tests.rs"]
mod tests;
