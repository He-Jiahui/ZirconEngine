use core::ops::Range;

use zr_rhi::TextureCopyRegion;

use crate::text::atlas::GlyphAtlasUploadCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasTextureUploadWrite {
    pub(in crate::graphics::scene::scene_renderer::ui) origin_x: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) origin_y: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) origin_layer: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) source_offset: u64,
    pub(in crate::graphics::scene::scene_renderer::ui) bytes_per_row: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) rows_per_image: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) extent_width: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) extent_height: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) extent_layers: u32,
}

pub(in crate::graphics::scene::scene_renderer::ui) fn glyph_atlas_texture_upload_write(
    command: GlyphAtlasUploadCommand,
) -> GlyphAtlasTextureUploadWrite {
    GlyphAtlasTextureUploadWrite {
        origin_x: command.rect.x,
        origin_y: command.rect.y,
        origin_layer: command.page_key.page_index,
        source_offset: command.source_offset,
        bytes_per_row: command.bytes_per_row,
        rows_per_image: command.rows_per_image,
        extent_width: command.rect.width,
        extent_height: command.rect.height,
        extent_layers: 1,
    }
}

pub(in crate::graphics::scene::scene_renderer::ui) fn glyph_atlas_texture_upload_region(
    write: GlyphAtlasTextureUploadWrite,
) -> TextureCopyRegion {
    TextureCopyRegion::new(write.extent_width, write.extent_height).with_origin(
        write.origin_x,
        write.origin_y,
        write.origin_layer,
    )
}

pub(in crate::graphics::scene::scene_renderer::ui) fn glyph_atlas_texture_upload_source_range(
    write: GlyphAtlasTextureUploadWrite,
    upload_byte_len: usize,
) -> Option<Range<usize>> {
    let row_count = usize::try_from(write.extent_height).ok()?;
    if write.extent_width == 0
        || write.extent_layers == 0
        || row_count == 0
        || write.rows_per_image < write.extent_height
        || upload_byte_len % row_count != 0
    {
        return None;
    }
    let row_payload_byte_len = upload_byte_len.checked_div(row_count)?;
    if row_payload_byte_len == 0 || row_payload_byte_len > write.bytes_per_row as usize {
        return None;
    }
    let source_start = usize::try_from(write.source_offset).ok()?;
    let last_row_offset = row_count
        .checked_sub(1)?
        .checked_mul(write.bytes_per_row as usize)?
        .checked_add(source_start)?;
    let source_end = last_row_offset.checked_add(row_payload_byte_len)?;
    Some(source_start..source_end)
}
