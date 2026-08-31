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
    pub(crate) page_generation: u64,
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
    if source_byte_len == 0
        || matches!(mode, GlyphAtlasUploadMode::None)
        || page.size.x == 0
        || page.size.y == 0
    {
        return None;
    }

    let page_size = page.size;
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
    let rect = admit_upload_rect(rect, page_size)?;
    let bytes_per_pixel = page.storage_format.bytes_per_pixel();
    let bytes_per_row = page_size.x.checked_mul(bytes_per_pixel)?;
    let source_offset = u64::from(rect.y)
        .checked_mul(u64::from(page_size.x))?
        .checked_add(u64::from(rect.x))?
        .checked_mul(u64::from(bytes_per_pixel))?;
    let upload_byte_len = rect_byte_len(rect, bytes_per_pixel)?;
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
        page_generation: page.generation,
        sampling_semantics: page.sampling_semantics,
        rect,
        source_offset,
        bytes_per_row,
        rows_per_image: page_size.y,
        upload_byte_len,
    })
}

fn admit_upload_rect(rect: GlyphAtlasRect, page_size: UVec2) -> Option<GlyphAtlasRect> {
    if rect.width == 0 || rect.height == 0 {
        return None;
    }
    let right = rect.x.checked_add(rect.width)?;
    let bottom = rect.y.checked_add(rect.height)?;
    (right <= page_size.x && bottom <= page_size.y).then_some(rect)
}

fn rect_byte_len(rect: GlyphAtlasRect, bytes_per_pixel: u32) -> Option<usize> {
    usize::try_from(rect.width)
        .ok()?
        .checked_mul(usize::try_from(rect.height).ok()?)?
        .checked_mul(usize::try_from(bytes_per_pixel).ok()?)
}

fn source_range_fits(
    source_offset: u64,
    bytes_per_row: u32,
    rect: GlyphAtlasRect,
    bytes_per_pixel: u32,
    source_byte_len: usize,
) -> bool {
    let Some(last_row_index) = rect.height.checked_sub(1).map(u64::from) else {
        return false;
    };
    let Some(row_payload_len) = u64::from(rect.width).checked_mul(u64::from(bytes_per_pixel))
    else {
        return false;
    };
    let Some(last_row_offset) = last_row_index
        .checked_mul(u64::from(bytes_per_row))
        .and_then(|offset| source_offset.checked_add(offset))
    else {
        return false;
    };
    last_row_offset
        .checked_add(row_payload_len)
        .is_some_and(|end| end <= source_byte_len as u64)
}

#[cfg(test)]
mod tests;
