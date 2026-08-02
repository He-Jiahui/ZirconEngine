use std::sync::Arc;

use glyphon::{Color, SwashContent, TextBounds};

use crate::core::math::UVec2;
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{
    GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapSource, GlyphAtlasFormat, GlyphRasterKey,
};

#[derive(Clone, Copy)]
pub(super) struct NativeBitmapGlyphImage {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) line_y: f32,
    pub(super) top: i16,
    pub(super) left: i16,
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) format: GlyphAtlasFormat,
    pub(super) scale_factor: f32,
    pub(super) source_byte_len: usize,
    pub(super) foreground_color: [f32; 4],
    pub(super) background_color: [f32; 4],
}

pub(super) struct NativeBitmapAtlasClippedSource {
    pub(super) source: GlyphAtlasBitmapSource,
    pub(super) bytes: Arc<[u8]>,
    pub(super) was_clipped: bool,
}

pub(super) fn native_bitmap_atlas_source_from_image(
    image: NativeBitmapGlyphImage,
    clipped_rect: GlyphAtlasScreenRect,
    source_bytes: Arc<[u8]>,
    raster_key: Option<GlyphRasterKey>,
) -> Option<NativeBitmapAtlasClippedSource> {
    let content_size = UVec2::new(u32::from(image.width), u32::from(image.height));
    if content_size.x == 0 || content_size.y == 0 {
        return None;
    }
    if image.source_byte_len != source_bytes.len() {
        return None;
    }

    let screen_rect = native_bitmap_atlas_screen_rect(
        image.x,
        image.y,
        image.line_y,
        image.top,
        image.left,
        image.width,
        image.height,
        image.scale_factor,
    );
    let crop = native_bitmap_atlas_crop_from_clip(screen_rect, clipped_rect, content_size)?;
    let bytes = crop_native_bitmap_source_bytes(source_bytes, content_size, image.format, crop)?;
    let source = GlyphAtlasBitmapSource {
        raster_key: raster_key.filter(|_| !crop.was_clipped),
        format: image.format,
        content_size: crop.size,
        screen_rect: clipped_rect,
        foreground_color: image.foreground_color,
        background_color: image.background_color,
        source_byte_len: bytes.len(),
    };

    Some(NativeBitmapAtlasClippedSource {
        source,
        bytes,
        was_clipped: crop.was_clipped,
    })
}

pub(super) fn glyph_atlas_bitmap_face_validity_for_epoch(
    source_face_epochs: impl IntoIterator<Item = u64>,
    current_face_epoch: u64,
) -> GlyphAtlasBitmapFaceValidity {
    if source_face_epochs
        .into_iter()
        .all(|face_epoch| face_epoch == current_face_epoch)
    {
        GlyphAtlasBitmapFaceValidity::Valid
    } else {
        GlyphAtlasBitmapFaceValidity::Invalidated
    }
}

pub(super) fn native_bitmap_atlas_format(content: SwashContent) -> Option<GlyphAtlasFormat> {
    match content {
        SwashContent::Mask => Some(GlyphAtlasFormat::AlphaMask),
        SwashContent::Color => Some(GlyphAtlasFormat::Color),
        SwashContent::SubpixelMask => Some(GlyphAtlasFormat::SubpixelMask),
    }
}

pub(super) fn native_bitmap_atlas_foreground_color(
    format: GlyphAtlasFormat,
    text_color: [f32; 4],
) -> [f32; 4] {
    match format {
        GlyphAtlasFormat::Color => [1.0, 1.0, 1.0, 1.0],
        GlyphAtlasFormat::AlphaMask | GlyphAtlasFormat::SubpixelMask => text_color,
        GlyphAtlasFormat::Sdf | GlyphAtlasFormat::Msdf => text_color,
    }
}

pub(super) fn native_bitmap_atlas_background_color(
    format: GlyphAtlasFormat,
    text_background_color: Option<[f32; 4]>,
) -> [f32; 4] {
    match format {
        GlyphAtlasFormat::SubpixelMask => text_background_color.unwrap_or([0.0, 0.0, 0.0, 1.0]),
        GlyphAtlasFormat::AlphaMask
        | GlyphAtlasFormat::Color
        | GlyphAtlasFormat::Sdf
        | GlyphAtlasFormat::Msdf => [0.0, 0.0, 0.0, 1.0],
    }
}

pub(super) fn native_bitmap_atlas_format_requires_background_composite(
    format: GlyphAtlasFormat,
) -> bool {
    matches!(format, GlyphAtlasFormat::SubpixelMask)
}

#[derive(Clone, Copy)]
struct NativeBitmapAtlasCrop {
    left: u32,
    top: u32,
    size: UVec2,
    was_clipped: bool,
}

pub(super) fn native_bitmap_atlas_screen_rect(
    x: i32,
    y: i32,
    line_y: f32,
    top: i16,
    left: i16,
    width: u16,
    height: u16,
    scale_factor: f32,
) -> GlyphAtlasScreenRect {
    GlyphAtlasScreenRect::new(
        (x + i32::from(left)) as f32,
        ((line_y * scale_factor).round() as i32 + y - i32::from(top)) as f32,
        f32::from(width),
        f32::from(height),
    )
}

pub(super) fn text_bounds_clipped_screen_rect(
    bounds: TextBounds,
    rect: GlyphAtlasScreenRect,
) -> Option<GlyphAtlasScreenRect> {
    rect.clipped_to(GlyphAtlasScreenRect::new(
        bounds.left as f32,
        bounds.top as f32,
        (bounds.right - bounds.left).max(0) as f32,
        (bounds.bottom - bounds.top).max(0) as f32,
    ))
}

fn native_bitmap_atlas_crop_from_clip(
    screen_rect: GlyphAtlasScreenRect,
    clipped_rect: GlyphAtlasScreenRect,
    source_size: UVec2,
) -> Option<NativeBitmapAtlasCrop> {
    let left = rounded_non_negative_u32(clipped_rect.x - screen_rect.x)?;
    let top = rounded_non_negative_u32(clipped_rect.y - screen_rect.y)?;
    let width = rounded_non_negative_u32(clipped_rect.width)?;
    let height = rounded_non_negative_u32(clipped_rect.height)?;
    if width == 0 || height == 0 {
        return None;
    }
    if left.checked_add(width)? > source_size.x || top.checked_add(height)? > source_size.y {
        return None;
    }

    Some(NativeBitmapAtlasCrop {
        left,
        top,
        size: UVec2::new(width, height),
        was_clipped: left != 0 || top != 0 || width != source_size.x || height != source_size.y,
    })
}

fn rounded_non_negative_u32(value: f32) -> Option<u32> {
    if !value.is_finite() || value < 0.0 {
        return None;
    }

    u32::try_from(value.round() as i64).ok()
}

fn crop_native_bitmap_source_bytes(
    source_bytes: Arc<[u8]>,
    source_size: UVec2,
    format: GlyphAtlasFormat,
    crop: NativeBitmapAtlasCrop,
) -> Option<Arc<[u8]>> {
    if !crop.was_clipped {
        return Some(source_bytes);
    }

    let bytes_per_pixel = format.storage_format().bytes_per_pixel() as usize;
    let source_stride = source_size.x as usize * bytes_per_pixel;
    let crop_stride = crop.size.x as usize * bytes_per_pixel;
    let expected_byte_len = source_stride.checked_mul(source_size.y as usize)?;
    if source_bytes.len() != expected_byte_len {
        return None;
    }

    let mut cropped = Vec::with_capacity(crop_stride.saturating_mul(crop.size.y as usize));
    for row in 0..crop.size.y as usize {
        let source_row = crop.top as usize + row;
        let source_start = source_row
            .saturating_mul(source_stride)
            .saturating_add(crop.left as usize * bytes_per_pixel);
        let source_end = source_start.saturating_add(crop_stride);
        cropped.extend_from_slice(source_bytes.get(source_start..source_end)?);
    }
    Some(Arc::from(cropped))
}

pub(super) fn unpack_color(color: Color) -> [f32; 4] {
    let [r, g, b, a] = color.as_rgba();
    [
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    ]
}
