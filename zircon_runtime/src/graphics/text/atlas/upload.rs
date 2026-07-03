use crate::core::math::UVec2;

use super::{GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasRect, GlyphAtlasSamplingSemantics};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GlyphAtlasUploadMode {
    #[default]
    None,
    FullPage,
    PartialRect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasUploadCommand {
    pub(crate) mode: GlyphAtlasUploadMode,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) sampling_semantics: GlyphAtlasSamplingSemantics,
    pub(crate) rect: GlyphAtlasRect,
    pub(crate) source_offset: u64,
    pub(crate) bytes_per_row: u32,
    pub(crate) rows_per_image: u32,
    pub(crate) upload_byte_len: usize,
}

pub(crate) fn glyph_atlas_upload_command(
    page: &GlyphAtlasPageSpec,
    mode: GlyphAtlasUploadMode,
    dirty_rect: Option<GlyphAtlasRect>,
    source_byte_len: usize,
) -> Option<GlyphAtlasUploadCommand> {
    if source_byte_len == 0 || matches!(mode, GlyphAtlasUploadMode::None) {
        return None;
    }

    let page_size = UVec2::new(page.size.x.max(1), page.size.y.max(1));
    let rect = match mode {
        GlyphAtlasUploadMode::None => return None,
        GlyphAtlasUploadMode::FullPage => GlyphAtlasRect {
            x: 0,
            y: 0,
            width: page_size.x,
            height: page_size.y,
        },
        GlyphAtlasUploadMode::PartialRect => dirty_rect?,
    };
    let rect = clamp_upload_rect(rect, page_size)?;
    let bytes_per_pixel = page.storage_format.bytes_per_pixel();
    let bytes_per_row = page_size.x.saturating_mul(bytes_per_pixel);
    let source_offset =
        (rect.y as u64 * page_size.x as u64 + rect.x as u64) * bytes_per_pixel as u64;
    let upload_byte_len = rect_byte_len(rect, bytes_per_pixel);
    if !source_range_fits(
        source_offset,
        bytes_per_row,
        rect,
        bytes_per_pixel,
        source_byte_len,
    ) {
        return None;
    }

    Some(GlyphAtlasUploadCommand {
        mode,
        page_key: page.key,
        sampling_semantics: page.sampling_semantics,
        rect,
        source_offset,
        bytes_per_row,
        rows_per_image: page_size.y,
        upload_byte_len,
    })
}

fn clamp_upload_rect(rect: GlyphAtlasRect, page_size: UVec2) -> Option<GlyphAtlasRect> {
    let x = rect.x.min(page_size.x);
    let y = rect.y.min(page_size.y);
    let width = rect.width.min(page_size.x.saturating_sub(x));
    let height = rect.height.min(page_size.y.saturating_sub(y));
    (width > 0 && height > 0).then_some(GlyphAtlasRect {
        x,
        y,
        width,
        height,
    })
}

fn rect_byte_len(rect: GlyphAtlasRect, bytes_per_pixel: u32) -> usize {
    rect.width as usize * rect.height as usize * bytes_per_pixel as usize
}

fn source_range_fits(
    source_offset: u64,
    bytes_per_row: u32,
    rect: GlyphAtlasRect,
    bytes_per_pixel: u32,
    source_byte_len: usize,
) -> bool {
    let row_payload_len = rect.width as u64 * bytes_per_pixel as u64;
    let last_row_offset =
        source_offset + rect.height.saturating_sub(1) as u64 * bytes_per_row as u64;
    last_row_offset.saturating_add(row_payload_len) <= source_byte_len as u64
}

#[cfg(test)]
mod tests;
