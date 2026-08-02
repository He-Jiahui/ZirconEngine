use super::super::{
    glyph_atlas_upload_command, GlyphAtlasAllocation, GlyphAtlasDirtyPage, GlyphAtlasPageSpec,
    GlyphAtlasRect, GlyphAtlasSet, GlyphAtlasUploadCommand, GlyphAtlasUploadMode,
};
use super::types::{GlyphAtlasBitmapSource, GlyphAtlasBitmapUploadCopy};

pub(super) fn bitmap_upload_copy(
    source_index: usize,
    source: GlyphAtlasBitmapSource,
    allocation: GlyphAtlasAllocation,
) -> GlyphAtlasBitmapUploadCopy {
    let bytes_per_pixel = source.format.storage_format().bytes_per_pixel();
    GlyphAtlasBitmapUploadCopy {
        source_index,
        page_key: allocation.page_key,
        atlas_rect: allocation.rect,
        content_size: source.content_size,
        source_bytes_per_row: source.content_size.x.saturating_mul(bytes_per_pixel),
        source_byte_len: source.source_byte_len,
    }
}

pub(super) fn bitmap_upload_commands(
    atlas: &GlyphAtlasSet,
    dirty_pages: &[GlyphAtlasDirtyPage],
) -> Vec<GlyphAtlasUploadCommand> {
    let mut commands = Vec::new();
    for dirty_page in dirty_pages {
        let page_key = dirty_page.page_key();
        let Some(page) = atlas.page(page_key.format, page_key.page_index) else {
            continue;
        };
        let page_rect = GlyphAtlasRect {
            x: 0,
            y: 0,
            width: page.size.x.max(1),
            height: page.size.y.max(1),
        };
        for dirty_rect in dirty_page.regions_for_page(page_rect) {
            if let Some(command) = glyph_atlas_upload_command(
                page,
                bitmap_upload_mode(page, dirty_rect),
                Some(dirty_rect),
                bitmap_page_source_byte_len(page),
            ) {
                commands.push(command);
            }
        }
    }
    commands
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
