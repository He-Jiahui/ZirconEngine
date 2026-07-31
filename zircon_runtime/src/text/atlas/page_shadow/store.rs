use std::collections::{BTreeMap, BTreeSet};

use super::super::{GlyphAtlasPageKey, GlyphAtlasPageSpec};
use super::{
    GlyphAtlasBitmapPageShadow, GlyphAtlasBitmapPageShadowCommit, GlyphAtlasBitmapPageShadowPatch,
};

// This is a crate-local runtime budget: at the current 512x512 page size and
// default residency limit it covers every resident bitmap format (28 MiB),
// while retaining a hard cap if either policy changes independently.
const GLYPH_ATLAS_BITMAP_PAGE_SHADOW_MAX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPageShadowStore {
    pages: BTreeMap<GlyphAtlasPageKey, GlyphAtlasBitmapPageShadow>,
    byte_len: usize,
}

impl GlyphAtlasBitmapPageShadowStore {
    pub(crate) fn bytes_for_page(&self, page: &GlyphAtlasPageSpec) -> Option<&[u8]> {
        self.pages
            .get(&page.key)
            .filter(|shadow| {
                shadow.generation == page.generation && shadow.bytes.len() == page.byte_len()
            })
            .map(|shadow| shadow.bytes.as_slice())
    }

    pub(crate) fn apply(
        &mut self,
        resident_pages: &[GlyphAtlasPageSpec],
        commit: GlyphAtlasBitmapPageShadowCommit,
    ) {
        self.retain_current_pages(resident_pages);
        let pages_by_key = resident_pages
            .iter()
            .map(|page| (page.key, page))
            .collect::<BTreeMap<_, _>>();
        let failed_zero_initialized_pages = commit.failed_zero_initialized_pages;
        let zero_initialized_pages = commit
            .zero_initialized_pages
            .into_iter()
            .filter(|page_key| !failed_zero_initialized_pages.contains(page_key))
            .collect::<BTreeSet<_>>();

        for page_key in &zero_initialized_pages {
            if let Some(page) = pages_by_key.get(page_key) {
                self.ensure_page(page);
            }
        }

        for patch in commit.patches {
            let Some(page) = pages_by_key.get(&patch.page_key) else {
                continue;
            };
            if page.generation != patch.page_generation {
                continue;
            }
            if !self.pages.contains_key(&patch.page_key)
                && zero_initialized_pages.contains(&patch.page_key)
            {
                self.ensure_page(page);
            }
            self.apply_patch(page, patch);
        }
    }

    pub(crate) fn remove_page(&mut self, page_key: GlyphAtlasPageKey) {
        if let Some(shadow) = self.pages.remove(&page_key) {
            self.byte_len = self.byte_len.saturating_sub(shadow.bytes.len());
        }
    }

    fn retain_current_pages(&mut self, resident_pages: &[GlyphAtlasPageSpec]) {
        let current_generations = resident_pages
            .iter()
            .map(|page| (page.key, page.generation))
            .collect::<BTreeMap<_, _>>();
        self.pages.retain(|page_key, shadow| {
            current_generations.get(page_key) == Some(&shadow.generation)
        });
        self.byte_len = self.pages.values().map(|shadow| shadow.bytes.len()).sum();
    }

    fn ensure_page(&mut self, page: &GlyphAtlasPageSpec) -> bool {
        if self
            .pages
            .get(&page.key)
            .is_some_and(|shadow| shadow.generation == page.generation)
        {
            return true;
        }

        self.remove_page(page.key);
        let page_byte_len = page.byte_len();
        if self.byte_len.saturating_add(page_byte_len) > GLYPH_ATLAS_BITMAP_PAGE_SHADOW_MAX_BYTES {
            return false;
        }
        self.pages.insert(
            page.key,
            GlyphAtlasBitmapPageShadow {
                generation: page.generation,
                bytes: vec![0; page_byte_len],
            },
        );
        self.byte_len = self.byte_len.saturating_add(page_byte_len);
        true
    }

    fn apply_patch(&mut self, page: &GlyphAtlasPageSpec, patch: GlyphAtlasBitmapPageShadowPatch) {
        let bytes_per_pixel = page.storage_format.bytes_per_pixel() as usize;
        let target = patch.target_rect;
        let page_width = page.size.x as usize;
        let page_height = page.size.y as usize;
        let target_width = target.width as usize;
        let target_height = target.height as usize;
        let target_x = target.x as usize;
        let target_y = target.y as usize;
        let expected_bytes_per_row = target_width.saturating_mul(bytes_per_pixel);
        if target_x.saturating_add(target_width) > page_width
            || target_y.saturating_add(target_height) > page_height
            || patch.bytes_per_row as usize != expected_bytes_per_row
            || patch.bytes.len() != expected_bytes_per_row.saturating_mul(target_height)
        {
            return;
        }
        let Some(shadow) = self.pages.get_mut(&page.key) else {
            return;
        };
        if shadow.generation != page.generation {
            return;
        }

        for row in 0..target_height {
            let source_start = row.saturating_mul(expected_bytes_per_row);
            let source_end = source_start.saturating_add(expected_bytes_per_row);
            let destination_start = target_y
                .saturating_add(row)
                .saturating_mul(page_width)
                .saturating_add(target_x)
                .saturating_mul(bytes_per_pixel);
            let destination_end = destination_start.saturating_add(expected_bytes_per_row);
            shadow.bytes[destination_start..destination_end]
                .copy_from_slice(&patch.bytes[source_start..source_end]);
        }
    }
}
