use glyphon::{Color, FontSystem, SwashCache, SwashContent, TextArea, TextBounds};

use crate::core::math::UVec2;
use crate::graphics::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::graphics::text::atlas::{
    glyph_atlas_bitmap_render_submission_plan, GlyphAtlasBitmapRenderSubmissionPlan,
    GlyphAtlasBitmapRenderSubmissionReport, GlyphAtlasBitmapSource,
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasFormat, GlyphAtlasStorageFormat,
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
};

const BITMAP_ATLAS_FRAME_INDEX: u64 = 1;

pub(super) struct NativeBitmapAtlasFrame {
    pub(super) submission: GlyphAtlasBitmapRenderSubmissionPlan,
    source_images: Vec<NativeBitmapAtlasSourceImage>,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
    visible_raster_glyph_count: usize,
    unsupported_glyph_count: usize,
    clipped_glyph_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct NativeBitmapAtlasPrepareReport {
    pub(super) visible_raster_glyph_count: usize,
    pub(super) source_image_count: usize,
    pub(super) unsupported_glyph_count: usize,
    pub(super) clipped_glyph_count: usize,
    pub(super) atlas_storage_format: Option<GlyphAtlasStorageFormat>,
    pub(super) mixed_atlas_storage_format: bool,
    pub(super) storage_submission_count: usize,
    pub(super) storage_submission_visible_glyph_count: usize,
    pub(super) mixed_storage_replacement_ready: bool,
    pub(super) requires_background_composite: bool,
    pub(super) replaces_glyphon: bool,
    pub(super) submission: GlyphAtlasBitmapRenderSubmissionReport,
}

pub(super) struct NativeBitmapAtlasStorageSubmission {
    pub(super) storage_format: GlyphAtlasStorageFormat,
    pub(super) submission: GlyphAtlasBitmapRenderSubmissionPlan,
    source_images: Vec<NativeBitmapAtlasSourceImage>,
}

#[derive(Clone)]
struct NativeBitmapAtlasSourceImage {
    source: GlyphAtlasBitmapSource,
    bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
struct NativeBitmapGlyphImage {
    x: i32,
    y: i32,
    line_y: f32,
    top: i16,
    left: i16,
    width: u16,
    height: u16,
    format: GlyphAtlasFormat,
    scale_factor: f32,
    source_byte_len: usize,
    foreground_color: [f32; 4],
    background_color: [f32; 4],
}

impl NativeBitmapAtlasFrame {
    pub(super) fn replaces_glyphon(&self) -> bool {
        self.source_coverage_supports_replacement()
            && self.submission.run.allocation_failures.is_empty()
            && self.submission.gpu_draw.visible_glyph_count == self.visible_raster_glyph_count
            && self.atlas_storage_format().is_some()
            && !self.submission.gpu_draw.requires_background_composite
    }

    pub(super) fn source_bytes(&self) -> Vec<GlyphAtlasBitmapUploadSourceBytes<'_>> {
        self.source_images
            .iter()
            .enumerate()
            .map(|(source_index, image)| {
                GlyphAtlasBitmapUploadSourceBytes::new(source_index, &image.bytes)
            })
            .collect()
    }

    pub(super) fn storage_submissions(&self) -> Vec<NativeBitmapAtlasStorageSubmission> {
        native_bitmap_atlas_storage_formats_in_frame(
            self.source_images
                .iter()
                .map(|image| image.source.format.storage_format()),
        )
        .into_iter()
        .filter_map(|storage_format| {
            let source_images = self
                .source_images
                .iter()
                .filter(|image| image.source.format.storage_format() == storage_format)
                .cloned()
                .collect::<Vec<_>>();
            (!source_images.is_empty()).then(|| {
                NativeBitmapAtlasStorageSubmission::new(
                    storage_format,
                    source_images,
                    self.viewport_size,
                    self.clip_rect,
                )
            })
        })
        .collect()
    }

    pub(super) fn atlas_layer_count(&self) -> u32 {
        self.submission
            .gpu_draw
            .vertices
            .iter()
            .map(|vertex| vertex.page_index)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(1)
    }

    pub(super) fn atlas_storage_format(&self) -> Option<GlyphAtlasStorageFormat> {
        single_native_bitmap_atlas_storage_format(
            self.submission
                .run
                .glyphs
                .iter()
                .map(|glyph| glyph.page_key.format.storage_format()),
        )
    }

    pub(super) fn prepare_report(&self) -> NativeBitmapAtlasPrepareReport {
        let storage_submissions = self.storage_submissions();
        let storage_submission_visible_glyph_count = storage_submissions
            .iter()
            .map(NativeBitmapAtlasStorageSubmission::visible_glyph_count)
            .sum();
        let storage_submissions_replace_glyphon =
            self.storage_submissions_replace_glyphon(&storage_submissions);
        NativeBitmapAtlasPrepareReport {
            visible_raster_glyph_count: self.visible_raster_glyph_count,
            source_image_count: self.source_images.len(),
            unsupported_glyph_count: self.unsupported_glyph_count,
            clipped_glyph_count: self.clipped_glyph_count,
            atlas_storage_format: self.atlas_storage_format(),
            mixed_atlas_storage_format: native_bitmap_atlas_has_mixed_storage_formats(
                self.submission
                    .run
                    .glyphs
                    .iter()
                    .map(|glyph| glyph.page_key.format.storage_format()),
            ),
            storage_submission_count: storage_submissions.len(),
            storage_submission_visible_glyph_count,
            mixed_storage_replacement_ready: storage_submissions.len() > 1
                && storage_submissions_replace_glyphon,
            requires_background_composite: self.submission.gpu_draw.requires_background_composite,
            replaces_glyphon: self.replaces_glyphon(),
            submission: self.submission.submission_report(),
        }
    }

    fn source_coverage_supports_replacement(&self) -> bool {
        self.visible_raster_glyph_count > 0
            && self.unsupported_glyph_count == 0
            && self.clipped_glyph_count <= self.visible_raster_glyph_count
            && self.source_images.len() == self.visible_raster_glyph_count
    }

    fn storage_submissions_replace_glyphon(
        &self,
        storage_submissions: &[NativeBitmapAtlasStorageSubmission],
    ) -> bool {
        if !self.source_coverage_supports_replacement()
            || self.submission.gpu_draw.requires_background_composite
            || !native_bitmap_atlas_storage_formats_are_contiguous(
                self.source_images
                    .iter()
                    .map(|image| image.source.format.storage_format()),
            )
        {
            return false;
        }

        let source_count = storage_submissions
            .iter()
            .map(NativeBitmapAtlasStorageSubmission::source_image_count)
            .sum::<usize>();
        let visible_count = storage_submissions
            .iter()
            .map(NativeBitmapAtlasStorageSubmission::visible_glyph_count)
            .sum::<usize>();
        let has_failures = storage_submissions
            .iter()
            .any(NativeBitmapAtlasStorageSubmission::has_allocation_failures);

        source_count == self.visible_raster_glyph_count
            && visible_count == self.visible_raster_glyph_count
            && !has_failures
    }
}

impl NativeBitmapAtlasStorageSubmission {
    fn new(
        storage_format: GlyphAtlasStorageFormat,
        source_images: Vec<NativeBitmapAtlasSourceImage>,
        viewport_size: UVec2,
        clip_rect: GlyphAtlasScreenRect,
    ) -> Self {
        let submission = glyph_atlas_bitmap_render_submission_plan(
            source_images.iter().map(|image| image.source),
            bitmap_atlas_page_size(),
            BITMAP_ATLAS_FRAME_INDEX,
            GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
            viewport_size,
            clip_rect,
        );

        Self {
            storage_format,
            submission,
            source_images,
        }
    }

    pub(super) fn source_bytes(&self) -> Vec<GlyphAtlasBitmapUploadSourceBytes<'_>> {
        self.source_images
            .iter()
            .enumerate()
            .map(|(source_index, image)| {
                GlyphAtlasBitmapUploadSourceBytes::new(source_index, &image.bytes)
            })
            .collect()
    }

    pub(super) fn atlas_layer_count(&self) -> u32 {
        self.submission
            .gpu_draw
            .vertices
            .iter()
            .map(|vertex| vertex.page_index)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(1)
    }

    fn source_image_count(&self) -> usize {
        self.source_images.len()
    }

    fn visible_glyph_count(&self) -> usize {
        self.submission.gpu_draw.visible_glyph_count
    }

    fn has_allocation_failures(&self) -> bool {
        !self.submission.run.allocation_failures.is_empty()
    }
}

pub(super) fn native_bitmap_atlas_frame(
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    viewport_size: UVec2,
    text_areas: &[TextArea<'_>],
) -> NativeBitmapAtlasFrame {
    let mut sources = Vec::new();
    let mut source_images = Vec::new();
    let mut visible_raster_glyph_count = 0;
    let mut unsupported_glyph_count = 0;
    let mut clipped_glyph_count = 0;

    for text_area in text_areas {
        let default_color = unpack_color(text_area.default_color);
        for run in text_area.buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical = glyph.physical((text_area.left, text_area.top), text_area.scale);
                let Some(image) = swash_cache.get_image_uncached(font_system, physical.cache_key)
                else {
                    continue;
                };
                if image.placement.width == 0 || image.placement.height == 0 {
                    continue;
                }
                let screen_rect = native_bitmap_atlas_screen_rect(
                    physical.x,
                    physical.y,
                    run.line_y,
                    image.placement.top as i16,
                    image.placement.left as i16,
                    image.placement.width as u16,
                    image.placement.height as u16,
                    text_area.scale,
                );
                let Some(clipped_rect) =
                    text_bounds_clipped_screen_rect(text_area.bounds, screen_rect)
                else {
                    continue;
                };
                visible_raster_glyph_count += 1;

                let Some(format) = native_bitmap_atlas_format(image.content) else {
                    unsupported_glyph_count += 1;
                    continue;
                };

                let glyph_image = NativeBitmapGlyphImage {
                    x: physical.x,
                    y: physical.y,
                    line_y: run.line_y,
                    top: image.placement.top as i16,
                    left: image.placement.left as i16,
                    width: image.placement.width as u16,
                    height: image.placement.height as u16,
                    format,
                    scale_factor: text_area.scale,
                    source_byte_len: image.data.len(),
                    foreground_color: native_bitmap_atlas_foreground_color(
                        format,
                        glyph.color_opt.map(unpack_color).unwrap_or(default_color),
                    ),
                    background_color: [0.0, 0.0, 0.0, 1.0],
                };
                if let Some(clipped_source) =
                    native_bitmap_atlas_source_from_image(glyph_image, clipped_rect, image.data)
                {
                    clipped_glyph_count += usize::from(clipped_source.was_clipped);
                    let source_index = sources.len();
                    source_images.push(NativeBitmapAtlasSourceImage {
                        source: clipped_source.source,
                        bytes: clipped_source.bytes,
                    });
                    sources.push(clipped_source.source);
                }
            }
        }
    }

    let clip_rect = GlyphAtlasScreenRect::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let submission = glyph_atlas_bitmap_render_submission_plan(
        sources,
        bitmap_atlas_page_size(),
        BITMAP_ATLAS_FRAME_INDEX,
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
        viewport_size,
        clip_rect,
    );

    NativeBitmapAtlasFrame {
        submission,
        source_images,
        viewport_size,
        clip_rect,
        visible_raster_glyph_count,
        unsupported_glyph_count,
        clipped_glyph_count,
    }
}

pub(super) fn bitmap_atlas_page_size() -> UVec2 {
    UVec2::new(512, 512)
}

struct NativeBitmapAtlasClippedSource {
    source: GlyphAtlasBitmapSource,
    bytes: Vec<u8>,
    was_clipped: bool,
}

fn native_bitmap_atlas_source_from_image(
    image: NativeBitmapGlyphImage,
    clipped_rect: GlyphAtlasScreenRect,
    source_bytes: Vec<u8>,
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

fn native_bitmap_atlas_format(content: SwashContent) -> Option<GlyphAtlasFormat> {
    match content {
        SwashContent::Mask => Some(GlyphAtlasFormat::AlphaMask),
        SwashContent::Color => Some(GlyphAtlasFormat::Color),
        SwashContent::SubpixelMask => Some(GlyphAtlasFormat::SubpixelMask),
    }
}

fn native_bitmap_atlas_foreground_color(
    format: GlyphAtlasFormat,
    text_color: [f32; 4],
) -> [f32; 4] {
    match format {
        GlyphAtlasFormat::Color => [1.0, 1.0, 1.0, 1.0],
        GlyphAtlasFormat::AlphaMask | GlyphAtlasFormat::SubpixelMask => text_color,
        GlyphAtlasFormat::Sdf | GlyphAtlasFormat::Msdf => text_color,
    }
}

fn single_native_bitmap_atlas_storage_format<I>(formats: I) -> Option<GlyphAtlasStorageFormat>
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut formats = formats.into_iter();
    let first = formats.next()?;
    formats.all(|format| format == first).then_some(first)
}

fn native_bitmap_atlas_has_mixed_storage_formats<I>(formats: I) -> bool
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut formats = formats.into_iter();
    let Some(first) = formats.next() else {
        return false;
    };
    formats.any(|format| format != first)
}

fn native_bitmap_atlas_storage_formats_in_frame<I>(formats: I) -> Vec<GlyphAtlasStorageFormat>
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut unique_formats = Vec::new();
    for format in formats {
        if !unique_formats.contains(&format) {
            unique_formats.push(format);
        }
    }
    unique_formats
}

fn native_bitmap_atlas_storage_formats_are_contiguous<I>(formats: I) -> bool
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut seen_formats = Vec::new();
    let mut previous_format = None;
    for format in formats {
        if previous_format == Some(format) {
            continue;
        }
        if seen_formats.contains(&format) {
            return false;
        }
        seen_formats.push(format);
        previous_format = Some(format);
    }
    true
}

#[derive(Clone, Copy)]
struct NativeBitmapAtlasCrop {
    left: u32,
    top: u32,
    size: UVec2,
    was_clipped: bool,
}

fn native_bitmap_atlas_screen_rect(
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

fn text_bounds_clipped_screen_rect(
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
    source_bytes: Vec<u8>,
    source_size: UVec2,
    format: GlyphAtlasFormat,
    crop: NativeBitmapAtlasCrop,
) -> Option<Vec<u8>> {
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
    Some(cropped)
}

fn unpack_color(color: Color) -> [f32; 4] {
    let [r, g, b, a] = color.as_rgba();
    [
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    ]
}

#[cfg(test)]
mod tests;
