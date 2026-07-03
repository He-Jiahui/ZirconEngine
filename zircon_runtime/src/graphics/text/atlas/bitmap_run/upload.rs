use super::super::{
    glyph_atlas_upload_command, GlyphAtlasDirtyPage, GlyphAtlasPageSpec, GlyphAtlasRect,
    GlyphAtlasSet, GlyphAtlasUploadCommand, GlyphAtlasUploadMode,
};

pub(super) fn bitmap_upload_commands(
    atlas: &GlyphAtlasSet,
    dirty_pages: &[GlyphAtlasDirtyPage],
) -> Vec<GlyphAtlasUploadCommand> {
    dirty_pages
        .iter()
        .filter_map(|dirty_page| {
            let page_key = dirty_page.page_key();
            let page = atlas.page(page_key.format, page_key.page_index)?;
            let dirty_rect = dirty_page.merged_rect()?;
            glyph_atlas_upload_command(
                page,
                bitmap_upload_mode(page, dirty_rect),
                Some(dirty_rect),
                bitmap_page_source_byte_len(page),
            )
        })
        .collect()
}

fn bitmap_upload_mode(
    page: &GlyphAtlasPageSpec,
    dirty_rect: GlyphAtlasRect,
) -> GlyphAtlasUploadMode {
    if dirty_rect.x == 0
        && dirty_rect.y == 0
        && dirty_rect.width >= page.size.x
        && dirty_rect.height >= page.size.y
    {
        GlyphAtlasUploadMode::FullPage
    } else {
        GlyphAtlasUploadMode::PartialRect
    }
}

fn bitmap_page_source_byte_len(page: &GlyphAtlasPageSpec) -> usize {
    (page.size.x as usize)
        .saturating_mul(page.size.y as usize)
        .saturating_mul(page.storage_format.bytes_per_pixel() as usize)
}
