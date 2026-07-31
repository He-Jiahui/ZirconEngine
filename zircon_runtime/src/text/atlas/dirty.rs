use super::{GlyphAtlasPageKey, GlyphAtlasRect};

const GLYPH_ATLAS_DIRTY_MAX_REGIONS_PER_PAGE: usize = 8;
const GLYPH_ATLAS_DIRTY_MAX_MERGE_EXTRA_BYTES: u64 = 4 * 1024;
const GLYPH_ATLAS_DIRTY_FULL_PAGE_THRESHOLD_PERCENT: u64 = 75;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasDirtyPage {
    page_key: GlyphAtlasPageKey,
    merged_rect: Option<GlyphAtlasRect>,
    regions: Vec<GlyphAtlasRect>,
    retained_regions: Vec<GlyphAtlasRect>,
    can_clear_unretained_pixels: bool,
    can_replay_retained_pixels: bool,
    full_page_requested: bool,
}

impl GlyphAtlasDirtyPage {
    pub(crate) fn new(page_key: GlyphAtlasPageKey) -> Self {
        Self::new_with_retained_regions_inner(page_key, Vec::new(), false, false)
    }

    pub(crate) fn new_with_retained_regions(
        page_key: GlyphAtlasPageKey,
        retained_regions: Vec<GlyphAtlasRect>,
    ) -> Self {
        Self::new_with_retained_regions_inner(page_key, retained_regions, true, false)
    }

    pub(crate) fn new_with_replayable_shadow(
        page_key: GlyphAtlasPageKey,
        retained_regions: Vec<GlyphAtlasRect>,
    ) -> Self {
        Self::new_with_retained_regions_inner(page_key, retained_regions, true, true)
    }

    fn new_with_retained_regions_inner(
        page_key: GlyphAtlasPageKey,
        retained_regions: Vec<GlyphAtlasRect>,
        can_clear_unretained_pixels: bool,
        can_replay_retained_pixels: bool,
    ) -> Self {
        Self {
            page_key,
            merged_rect: None,
            regions: Vec::new(),
            retained_regions,
            can_clear_unretained_pixels,
            can_replay_retained_pixels,
            full_page_requested: false,
        }
    }

    pub(crate) fn mark_dirty(&mut self, page_key: GlyphAtlasPageKey, rect: GlyphAtlasRect) {
        if page_key != self.page_key || rect.width == 0 || rect.height == 0 {
            return;
        }

        self.merged_rect = Some(match self.merged_rect {
            Some(existing) => existing.union(rect),
            None => rect,
        });
        if self.full_page_requested {
            return;
        }

        // Compact staging contains only this frame's source copies, so a partial
        // target may never grow into pixels retained by another persistent slot.
        self.regions.push(rect);
        self.merge_safe_regions();
        self.enforce_write_limit_with_shadow();
    }

    pub(crate) fn mark_full_page_dirty(
        &mut self,
        page_key: GlyphAtlasPageKey,
        page_rect: GlyphAtlasRect,
    ) {
        if page_key != self.page_key || page_rect.width == 0 || page_rect.height == 0 {
            return;
        }

        self.merged_rect = Some(page_rect);
        self.regions.clear();
        self.full_page_requested = true;
    }

    pub(crate) fn page_key(&self) -> GlyphAtlasPageKey {
        self.page_key
    }

    pub(crate) fn regions(&self) -> &[GlyphAtlasRect] {
        &self.regions
    }

    pub(crate) fn regions_for_page(&self, page_rect: GlyphAtlasRect) -> Vec<GlyphAtlasRect> {
        if self.merged_rect.is_none() {
            return Vec::new();
        }
        if self.full_page_requested || self.should_upload_full_page(page_rect) {
            return vec![page_rect];
        }
        self.regions.clone()
    }

    pub(crate) fn merged_rect(&self) -> Option<GlyphAtlasRect> {
        self.merged_rect
    }

    fn merge_safe_regions(&mut self) {
        loop {
            let Some((left_index, right_index, merged)) = self.safe_region_merge() else {
                return;
            };

            self.regions[left_index] = merged;
            self.regions.remove(right_index);
        }
    }

    fn safe_region_merge(&self) -> Option<(usize, usize, GlyphAtlasRect)> {
        let mut best = None;
        for left_index in 0..self.regions.len() {
            for right_index in left_index.saturating_add(1)..self.regions.len() {
                let left = self.regions[left_index];
                let right = self.regions[right_index];
                let merged = left.union(right);
                if self.intersects_retained_region(merged) && !self.can_replay_retained_pixels {
                    continue;
                }
                let extra_byte_cost = self.merge_extra_byte_cost(left, right, merged);
                if self.has_exact_coverage(left, right, merged)
                    || ((self.can_clear_unretained_pixels || self.can_replay_retained_pixels)
                        && extra_byte_cost <= GLYPH_ATLAS_DIRTY_MAX_MERGE_EXTRA_BYTES)
                {
                    if best.is_none_or(|(_, _, _, best_cost)| extra_byte_cost < best_cost) {
                        best = Some((left_index, right_index, merged, extra_byte_cost));
                    }
                }
            }
        }
        best.map(|(left_index, right_index, merged, _)| (left_index, right_index, merged))
    }

    fn enforce_write_limit_with_shadow(&mut self) {
        if !self.can_replay_retained_pixels {
            return;
        }

        while self.regions.len() > GLYPH_ATLAS_DIRTY_MAX_REGIONS_PER_PAGE {
            let Some((left_index, right_index, merged)) = self.lowest_cost_region_merge() else {
                return;
            };
            self.regions[left_index] = merged;
            self.regions.remove(right_index);
        }
    }

    fn lowest_cost_region_merge(&self) -> Option<(usize, usize, GlyphAtlasRect)> {
        let mut best = None;
        for left_index in 0..self.regions.len() {
            for right_index in left_index.saturating_add(1)..self.regions.len() {
                let left = self.regions[left_index];
                let right = self.regions[right_index];
                let merged = left.union(right);
                let extra_byte_cost = self.merge_extra_byte_cost(left, right, merged);
                if best.is_none_or(|(_, _, _, best_cost)| extra_byte_cost < best_cost) {
                    best = Some((left_index, right_index, merged, extra_byte_cost));
                }
            }
        }
        best.map(|(left_index, right_index, merged, _)| (left_index, right_index, merged))
    }

    fn should_upload_full_page(&self, page_rect: GlyphAtlasRect) -> bool {
        if !self.can_replay_retained_pixels
            && (!self.can_clear_unretained_pixels || !self.retained_regions.is_empty())
        {
            return false;
        }
        if self.regions.len() > GLYPH_ATLAS_DIRTY_MAX_REGIONS_PER_PAGE
            && !self.can_replay_retained_pixels
        {
            return true;
        }

        let page_byte_len = self.rect_byte_len(page_rect);
        let region_byte_len = self
            .regions
            .iter()
            .copied()
            .map(|region| self.rect_byte_len(region))
            .fold(0_u64, u64::saturating_add);
        page_byte_len > 0
            && region_byte_len.saturating_mul(100)
                >= page_byte_len.saturating_mul(GLYPH_ATLAS_DIRTY_FULL_PAGE_THRESHOLD_PERCENT)
    }

    fn intersects_retained_region(&self, rect: GlyphAtlasRect) -> bool {
        self.retained_regions
            .iter()
            .copied()
            .any(|retained| rect_intersection_area(rect, retained) > 0)
    }

    fn merge_extra_byte_cost(
        &self,
        left: GlyphAtlasRect,
        right: GlyphAtlasRect,
        merged: GlyphAtlasRect,
    ) -> u64 {
        self.rect_byte_len(merged).saturating_sub(
            self.rect_byte_len(left)
                .saturating_add(self.rect_byte_len(right))
                .saturating_sub(self.rect_byte_len_for_area(rect_intersection_area(left, right))),
        )
    }

    fn rect_byte_len(&self, rect: GlyphAtlasRect) -> u64 {
        self.rect_byte_len_for_area(rect_area(rect))
    }

    fn rect_byte_len_for_area(&self, area: u64) -> u64 {
        area.saturating_mul(u64::from(
            self.page_key.format.storage_format().bytes_per_pixel(),
        ))
    }

    fn has_exact_coverage(
        &self,
        left: GlyphAtlasRect,
        right: GlyphAtlasRect,
        merged: GlyphAtlasRect,
    ) -> bool {
        rect_area(merged)
            == rect_area(left)
                .saturating_add(rect_area(right))
                .saturating_sub(rect_intersection_area(left, right))
    }
}

fn rect_area(rect: GlyphAtlasRect) -> u64 {
    u64::from(rect.width).saturating_mul(u64::from(rect.height))
}

fn rect_intersection_area(left: GlyphAtlasRect, right: GlyphAtlasRect) -> u64 {
    let intersection_left = left.x.max(right.x);
    let intersection_top = left.y.max(right.y);
    let intersection_right = left
        .x
        .saturating_add(left.width)
        .min(right.x.saturating_add(right.width));
    let intersection_bottom = left
        .y
        .saturating_add(left.height)
        .min(right.y.saturating_add(right.height));

    u64::from(intersection_right.saturating_sub(intersection_left)).saturating_mul(u64::from(
        intersection_bottom.saturating_sub(intersection_top),
    ))
}

#[cfg(test)]
mod tests;
