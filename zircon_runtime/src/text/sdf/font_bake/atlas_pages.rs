use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::core::math::UVec2;
use crate::text::atlas::GlyphAtlasPageKey;

use super::{
    RawBakedGlyph, SdfAtlasBakeDirtyPage, SdfAtlasBakePage, SdfAtlasGlyphKey, SdfAtlasRect,
    SdfAtlasSlot,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfPersistentAtlasUpdateReport {
    pub(super) resident_page_count: usize,
    pub(super) page_alloc_count: usize,
    pub(super) page_zero_byte_count: usize,
    pub(super) page_clear_count: usize,
    pub(super) page_clear_byte_count: usize,
    pub(super) page_write_count: usize,
    pub(super) page_write_byte_count: usize,
    pub(super) reused_slot_count: usize,
    pub(super) atlas_byte_len: usize,
    pub(super) nonzero_pixel_count: usize,
    pub(super) full_page_scan_byte_count: usize,
}

#[derive(Default)]
pub(super) struct SdfPersistentAtlasCache {
    pages: HashMap<GlyphAtlasPageKey, PersistentAtlasPage>,
    placements: HashMap<SdfAtlasGlyphKey, PersistentAtlasPlacement>,
}

struct PersistentAtlasPage {
    size: UVec2,
    pixels: Arc<[u8]>,
    nonzero_pixel_count: usize,
}

struct PersistentAtlasPlacement {
    page_key: GlyphAtlasPageKey,
    rect: SdfAtlasRect,
    glyph_width: u32,
    glyph_height: u32,
    bitmap: Arc<[u8]>,
}

impl SdfPersistentAtlasCache {
    pub(super) fn update(
        &mut self,
        atlas_size: UVec2,
        slots: &[SdfAtlasSlot],
        glyphs: &[RawBakedGlyph],
    ) -> (
        Vec<SdfAtlasBakePage>,
        Vec<SdfAtlasBakeDirtyPage>,
        SdfPersistentAtlasUpdateReport,
    ) {
        let page_size = UVec2::new(atlas_size.x.max(1), atlas_size.y.max(1));
        let required_pages = slots
            .iter()
            .map(|slot| slot.page_key)
            .collect::<HashSet<_>>();
        self.pages
            .retain(|page_key, _| required_pages.contains(page_key));

        let mut report = SdfPersistentAtlasUpdateReport::default();
        let mut reset_pages = HashSet::new();
        let mut dirty_pages = BTreeMap::<GlyphAtlasPageKey, SdfAtlasRect>::new();
        for page_key in required_pages.iter().copied() {
            let byte_len = page_byte_len(page_size, page_key);
            let reset = self
                .pages
                .get(&page_key)
                .is_none_or(|page| page.size != page_size || page.pixels.len() != byte_len);
            if reset {
                self.pages.insert(
                    page_key,
                    PersistentAtlasPage {
                        size: page_size,
                        pixels: vec![0; byte_len].into(),
                        nonzero_pixel_count: 0,
                    },
                );
                reset_pages.insert(page_key);
                report.page_alloc_count = report.page_alloc_count.saturating_add(1);
                report.page_zero_byte_count = report.page_zero_byte_count.saturating_add(byte_len);
                mark_dirty_rect(
                    &mut dirty_pages,
                    page_key,
                    SdfAtlasRect {
                        x: 0,
                        y: 0,
                        width: page_size.x,
                        height: page_size.y,
                    },
                );
            }
        }

        let current_slots = slots
            .iter()
            .enumerate()
            .map(|(index, slot)| (&slot.key, index))
            .collect::<HashMap<_, _>>();
        let previous_placements = std::mem::take(&mut self.placements);

        for (key, previous) in &previous_placements {
            let current_location_unchanged = current_slots
                .get(key)
                .and_then(|index| slots.get(*index))
                .is_some_and(|slot| {
                    previous.page_key == slot.page_key && previous.rect == slot.rect
                });
            if current_location_unchanged || reset_pages.contains(&previous.page_key) {
                continue;
            }
            if let Some(page) = self.pages.get_mut(&previous.page_key) {
                let mutation = clear_rect(page, previous.rect);
                mark_dirty_rect(&mut dirty_pages, previous.page_key, previous.rect);
                report.page_clear_count = report.page_clear_count.saturating_add(1);
                report.page_clear_byte_count = report
                    .page_clear_byte_count
                    .saturating_add(mutation.touched_byte_count);
            }
        }

        for (slot, glyph) in slots.iter().zip(glyphs) {
            let unchanged = previous_placements.get(&slot.key).is_some_and(|previous| {
                !reset_pages.contains(&slot.page_key) && previous.matches(slot, glyph)
            });
            if unchanged {
                report.reused_slot_count = report.reused_slot_count.saturating_add(1);
            } else if let Some(page) = self.pages.get_mut(&slot.page_key) {
                let clear_before_write = !reset_pages.contains(&slot.page_key);
                let mutation = replace_rect(page, slot.rect, glyph, clear_before_write);
                if clear_before_write {
                    report.page_clear_count = report.page_clear_count.saturating_add(1);
                    report.page_clear_byte_count = report
                        .page_clear_byte_count
                        .saturating_add(mutation.cleared_byte_count);
                }
                report.page_write_count = report.page_write_count.saturating_add(1);
                report.page_write_byte_count = report
                    .page_write_byte_count
                    .saturating_add(mutation.written_byte_count);
                mark_dirty_rect(&mut dirty_pages, slot.page_key, slot.rect);
            }
            self.placements
                .insert(slot.key.clone(), PersistentAtlasPlacement::new(slot, glyph));
        }

        let mut source_offset = 0_usize;
        let pages = ordered_persistent_pages(&self.pages)
            .into_iter()
            .map(|(page_key, page)| {
                let byte_len = page.pixels.len();
                let bake_page = SdfAtlasBakePage {
                    page_key: *page_key,
                    source_offset,
                    byte_len,
                    pixels: Arc::clone(&page.pixels),
                };
                source_offset = source_offset.saturating_add(byte_len);
                bake_page
            })
            .collect::<Vec<_>>();
        report.resident_page_count = pages.len();
        report.atlas_byte_len = source_offset;
        report.nonzero_pixel_count = self
            .pages
            .values()
            .map(|page| page.nonzero_pixel_count)
            .sum();
        let dirty_pages = dirty_pages
            .into_iter()
            .map(|(page_key, dirty_rect)| SdfAtlasBakeDirtyPage {
                page_key,
                dirty_rect,
            })
            .collect();
        (pages, dirty_pages, report)
    }
}

fn ordered_persistent_pages(
    pages: &HashMap<GlyphAtlasPageKey, PersistentAtlasPage>,
) -> Vec<(&GlyphAtlasPageKey, &PersistentAtlasPage)> {
    let mut pages = pages.iter().collect::<Vec<_>>();
    pages.sort_unstable_by_key(|(page_key, _)| **page_key);
    pages
}

impl PersistentAtlasPlacement {
    fn new(slot: &SdfAtlasSlot, glyph: &RawBakedGlyph) -> Self {
        Self {
            page_key: slot.page_key,
            rect: slot.rect,
            glyph_width: glyph.metrics.bitmap_width,
            glyph_height: glyph.metrics.bitmap_height,
            bitmap: Arc::clone(&glyph.bitmap),
        }
    }

    fn matches(&self, slot: &SdfAtlasSlot, glyph: &RawBakedGlyph) -> bool {
        self.page_key == slot.page_key
            && self.rect == slot.rect
            && self.glyph_width == glyph.metrics.bitmap_width
            && self.glyph_height == glyph.metrics.bitmap_height
            && (Arc::ptr_eq(&self.bitmap, &glyph.bitmap)
                || (self.bitmap.is_empty() && glyph.bitmap.is_empty()))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RectMutation {
    touched_byte_count: usize,
    cleared_byte_count: usize,
    written_byte_count: usize,
}

fn replace_rect(
    page: &mut PersistentAtlasPage,
    rect: SdfAtlasRect,
    glyph: &RawBakedGlyph,
    clear_before_write: bool,
) -> RectMutation {
    let mut mutation = if clear_before_write {
        let cleared = clear_rect(page, rect);
        RectMutation {
            touched_byte_count: cleared.touched_byte_count,
            cleared_byte_count: cleared.touched_byte_count,
            written_byte_count: 0,
        }
    } else {
        RectMutation::default()
    };
    let bytes_per_pixel = rect_bytes_per_pixel(page, rect);
    let copy_width = glyph.metrics.bitmap_width.min(rect.width);
    let copy_height = glyph.metrics.bitmap_height.min(rect.height);
    let right = rect.x.saturating_add(copy_width).min(page.size.x);
    let bottom = rect.y.saturating_add(copy_height).min(page.size.y);
    let page_width = page.size.x as usize;
    let pixels = Arc::make_mut(&mut page.pixels);
    let mut added_nonzero = 0_usize;
    for y in rect.y..bottom {
        for x in rect.x..right {
            let local_x = x - rect.x;
            let local_y = y - rect.y;
            let src = (local_y as usize * glyph.metrics.bitmap_width as usize + local_x as usize)
                * bytes_per_pixel;
            let dst = (y as usize * page_width + x as usize) * bytes_per_pixel;
            let src_end = src.saturating_add(bytes_per_pixel);
            let dst_end = dst.saturating_add(bytes_per_pixel);
            if let (Some(source), Some(destination)) =
                (glyph.bitmap.get(src..src_end), pixels.get_mut(dst..dst_end))
            {
                added_nonzero = added_nonzero
                    .saturating_add(source.iter().filter(|sample| **sample != 0).count());
                destination.copy_from_slice(source);
                mutation.written_byte_count =
                    mutation.written_byte_count.saturating_add(source.len());
            }
        }
    }
    page.nonzero_pixel_count = page.nonzero_pixel_count.saturating_add(added_nonzero);
    mutation.touched_byte_count = mutation
        .touched_byte_count
        .saturating_add(mutation.written_byte_count);
    mutation
}

fn clear_rect(page: &mut PersistentAtlasPage, rect: SdfAtlasRect) -> RectMutation {
    let bytes_per_pixel = rect_bytes_per_pixel(page, rect);
    let right = rect.x.saturating_add(rect.width).min(page.size.x);
    let bottom = rect.y.saturating_add(rect.height).min(page.size.y);
    let page_width = page.size.x as usize;
    let pixels = Arc::make_mut(&mut page.pixels);
    let mut removed_nonzero = 0_usize;
    let mut touched_byte_count = 0_usize;
    for y in rect.y.min(page.size.y)..bottom {
        let start = (y as usize * page_width + rect.x.min(page.size.x) as usize)
            .saturating_mul(bytes_per_pixel);
        let end = (y as usize * page_width + right as usize).saturating_mul(bytes_per_pixel);
        if let Some(row) = pixels.get_mut(start..end) {
            removed_nonzero =
                removed_nonzero.saturating_add(row.iter().filter(|sample| **sample != 0).count());
            row.fill(0);
            touched_byte_count = touched_byte_count.saturating_add(row.len());
        }
    }
    page.nonzero_pixel_count = page.nonzero_pixel_count.saturating_sub(removed_nonzero);
    RectMutation {
        touched_byte_count,
        cleared_byte_count: touched_byte_count,
        written_byte_count: 0,
    }
}

fn page_byte_len(size: UVec2, page_key: GlyphAtlasPageKey) -> usize {
    size.x as usize * size.y as usize * page_key.format.storage_format().bytes_per_pixel() as usize
}

fn rect_bytes_per_pixel(page: &PersistentAtlasPage, _rect: SdfAtlasRect) -> usize {
    let pixel_count = page.size.x as usize * page.size.y as usize;
    page.pixels
        .len()
        .checked_div(pixel_count)
        .unwrap_or(1)
        .max(1)
}

fn mark_dirty_rect(
    dirty_pages: &mut BTreeMap<GlyphAtlasPageKey, SdfAtlasRect>,
    page_key: GlyphAtlasPageKey,
    rect: SdfAtlasRect,
) {
    dirty_pages
        .entry(page_key)
        .and_modify(|dirty| *dirty = union_rect(*dirty, rect))
        .or_insert(rect);
}

fn union_rect(left: SdfAtlasRect, right: SdfAtlasRect) -> SdfAtlasRect {
    let x = left.x.min(right.x);
    let y = left.y.min(right.y);
    let right_edge = left
        .x
        .saturating_add(left.width)
        .max(right.x.saturating_add(right.width));
    let bottom_edge = left
        .y
        .saturating_add(left.height)
        .max(right.y.saturating_add(right.height));
    SdfAtlasRect {
        x,
        y,
        width: right_edge.saturating_sub(x),
        height: bottom_edge.saturating_sub(y),
    }
}

#[cfg(test)]
#[path = "atlas_pages/hash_page_tests.rs"]
mod hash_page_tests;
