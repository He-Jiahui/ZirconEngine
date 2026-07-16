//! Deterministic atlas packing for offline font distance-field artifacts.

use crate::core::math::UVec2;
use crate::text::atlas::{GlyphAtlasPageKey, GlyphAtlasShelfAllocator};
use crate::text::sdf::{
    SdfGlyphData, SdfOfflineGlyph, SdfOfflineGlyphMetrics, SdfOfflinePage, SdfOfflineRect,
};

use super::FontSdfBakeError;

pub(super) struct GeneratedGlyph {
    pub(super) codepoint: u32,
    pub(super) glyph_id: u32,
    pub(super) data: SdfGlyphData,
}

pub(super) fn pack_generated_glyphs(
    generated: Vec<GeneratedGlyph>,
    page_size_px: u32,
) -> Result<(Vec<SdfOfflinePage>, Vec<SdfOfflineGlyph>), FontSdfBakeError> {
    let page_size = UVec2::splat(page_size_px);
    let mode = generated
        .first()
        .map(|glyph| glyph.data.mode)
        .ok_or(FontSdfBakeError::NoGeneratedGlyphs { skipped_count: 0 })?;
    let channels = usize::from(mode.channel_count());
    let page_byte_len = usize::try_from(page_size_px)
        .ok()
        .and_then(|side| side.checked_mul(side))
        .and_then(|pixels| pixels.checked_mul(channels))
        .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;

    let mut allocators = Vec::<GlyphAtlasShelfAllocator>::new();
    let mut pages = Vec::<SdfOfflinePage>::new();
    let mut glyphs = Vec::with_capacity(generated.len());
    for glyph in generated {
        if glyph.data.size.x > page_size_px || glyph.data.size.y > page_size_px {
            return Err(FontSdfBakeError::GlyphExceedsPage {
                glyph_id: glyph.glyph_id,
                width: glyph.data.size.x,
                height: glyph.data.size.y,
                page_size: page_size_px,
            });
        }
        let allocation = match allocators
            .last_mut()
            .and_then(|allocator| allocator.allocate(glyph.data.size))
        {
            Some(allocation) => allocation,
            None => {
                let page_index = u32::try_from(allocators.len())
                    .map_err(|_| FontSdfBakeError::AtlasSizeOverflow)?;
                let page_key = GlyphAtlasPageKey::new(mode.atlas_format(), page_index);
                let mut allocator = GlyphAtlasShelfAllocator::new(page_key, page_size, 1);
                let allocation = allocator.allocate(glyph.data.size).ok_or(
                    FontSdfBakeError::GlyphExceedsPage {
                        glyph_id: glyph.glyph_id,
                        width: glyph.data.size.x,
                        height: glyph.data.size.y,
                        page_size: page_size_px,
                    },
                )?;
                allocators.push(allocator);
                pages.push(SdfOfflinePage {
                    page_index,
                    pixels: vec![0; page_byte_len],
                });
                allocation
            }
        };
        let page = pages
            .get_mut(allocation.page_key.page_index as usize)
            .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;
        copy_glyph_pixels(
            &mut page.pixels,
            page_size_px,
            allocation.rect.x,
            allocation.rect.y,
            &glyph.data,
        )?;
        glyphs.push(SdfOfflineGlyph {
            glyph_id: glyph.glyph_id,
            codepoint: glyph.codepoint,
            page_index: allocation.page_key.page_index,
            rect: SdfOfflineRect::new(
                allocation.rect.x,
                allocation.rect.y,
                glyph.data.size.x,
                glyph.data.size.y,
            ),
            metrics: SdfOfflineGlyphMetrics {
                bitmap_left: glyph.data.bitmap_left,
                bitmap_bottom: glyph.data.bitmap_bottom,
                advance: glyph.data.advance,
                ascent: glyph.data.ascent,
            },
        });
    }
    Ok((pages, glyphs))
}

fn copy_glyph_pixels(
    page: &mut [u8],
    page_size_px: u32,
    target_x: u32,
    target_y: u32,
    glyph: &SdfGlyphData,
) -> Result<(), FontSdfBakeError> {
    let channels = usize::from(glyph.channels);
    let source_row_len = usize::try_from(glyph.size.x)
        .ok()
        .and_then(|width| width.checked_mul(channels))
        .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;
    let page_width =
        usize::try_from(page_size_px).map_err(|_| FontSdfBakeError::AtlasSizeOverflow)?;
    for row in 0..glyph.size.y {
        let source_start = usize::try_from(row)
            .ok()
            .and_then(|row| row.checked_mul(source_row_len))
            .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;
        let source_end = source_start
            .checked_add(source_row_len)
            .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;
        let target_pixel = usize::try_from(target_y + row)
            .ok()
            .and_then(|row| row.checked_mul(page_width))
            .and_then(|offset| offset.checked_add(target_x as usize))
            .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;
        let target_start = target_pixel
            .checked_mul(channels)
            .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;
        let target_end = target_start
            .checked_add(source_row_len)
            .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;
        let source = glyph
            .pixels
            .get(source_start..source_end)
            .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;
        let target = page
            .get_mut(target_start..target_end)
            .ok_or(FontSdfBakeError::AtlasSizeOverflow)?;
        target.copy_from_slice(source);
    }
    Ok(())
}
