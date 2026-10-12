use std::collections::{BTreeSet, HashMap};
use std::ops::Range;

use crate::graphics::scene::scene_renderer::ui::text::{
    ScreenSpaceUiTextFrameChangeJournal, ScreenSpaceUiTextFrameProduct,
    ScreenSpaceUiTextFrameProductGeneration,
};
use crate::text::sdf::SdfAtlasGlyphKey;

use super::text_keys::collect_sdf_atlas_text_keys_iter;
use super::SdfAtlasRun;

#[derive(Default)]
pub(super) struct SdfAtlasSegmentProductIndex {
    frame_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    products: Vec<SdfAtlasSegmentProduct>,
    active_key_ref_counts: HashMap<SdfAtlasGlyphKey, usize>,
    run_ranges: Vec<Range<usize>>,
}

struct SdfAtlasSegmentProduct {
    unique_keys: BTreeSet<SdfAtlasGlyphKey>,
    run_keys: Vec<Vec<Option<SdfAtlasGlyphKey>>>,
}

pub(super) struct SdfAtlasSegmentDelta {
    changed_segment_indices: Vec<usize>,
    appended_segment_start: usize,
    appended_segment_count: usize,
    truncated_segment_count: usize,
    newly_active_keys: BTreeSet<SdfAtlasGlyphKey>,
    glyph_key_visit_count: usize,
    run_entry_visit_count: usize,
}

impl SdfAtlasSegmentProductIndex {
    pub(super) fn frame_matches(
        &self,
        generation: ScreenSpaceUiTextFrameProductGeneration,
    ) -> bool {
        self.frame_generation == Some(generation)
    }

    pub(super) fn rebuild(&mut self, frame: &ScreenSpaceUiTextFrameProduct) {
        self.products.clear();
        self.active_key_ref_counts.clear();
        self.run_ranges.clear();
        for segment in frame.segment_products() {
            let product = SdfAtlasSegmentProduct::from_frame_segment(segment.sdf_texts());
            add_active_keys(&mut self.active_key_ref_counts, &product.unique_keys);
            self.products.push(product);
        }
        self.frame_generation = Some(frame.generation());
    }

    pub(super) fn apply_local(
        &mut self,
        frame: &ScreenSpaceUiTextFrameProduct,
    ) -> Option<SdfAtlasSegmentDelta> {
        let journal = frame.change_journal();
        if journal.is_full_rebuild()
            || journal.base_generation() != self.frame_generation
            || journal.current_generation() != frame.generation()
        {
            return None;
        }
        let retained_segment_count = self
            .products
            .len()
            .checked_sub(journal.truncated_segment_count())?;
        if retained_segment_count.checked_add(journal.appended_segment_count())?
            != frame.segment_products().len()
            || journal
                .changed_segment_indices()
                .iter()
                .any(|&index| index >= retained_segment_count)
        {
            return None;
        }

        let mut newly_active_keys = BTreeSet::new();
        let mut glyph_key_visit_count = 0_usize;
        let mut run_entry_visit_count = 0_usize;
        for &index in journal.changed_segment_indices() {
            let replacement = SdfAtlasSegmentProduct::from_frame_segment(
                frame.segment_products()[index].sdf_texts(),
            );
            glyph_key_visit_count =
                glyph_key_visit_count.saturating_add(replacement.unique_keys.len());
            run_entry_visit_count = run_entry_visit_count
                .saturating_add(replacement.run_keys.iter().map(Vec::len).sum::<usize>());
            remove_active_keys(
                &mut self.active_key_ref_counts,
                &self.products[index].unique_keys,
            );
            add_active_keys_and_collect_new(
                &mut self.active_key_ref_counts,
                &replacement.unique_keys,
                &mut newly_active_keys,
            );
            self.products[index] = replacement;
        }

        let truncated_segment_count = journal.truncated_segment_count();
        for product in self.products.drain(retained_segment_count..) {
            glyph_key_visit_count = glyph_key_visit_count.saturating_add(product.unique_keys.len());
            remove_active_keys(&mut self.active_key_ref_counts, &product.unique_keys);
        }
        let appended_segment_start = self.products.len();
        for segment in &frame.segment_products()[appended_segment_start..] {
            let product = SdfAtlasSegmentProduct::from_frame_segment(segment.sdf_texts());
            glyph_key_visit_count = glyph_key_visit_count.saturating_add(product.unique_keys.len());
            run_entry_visit_count = run_entry_visit_count
                .saturating_add(product.run_keys.iter().map(Vec::len).sum::<usize>());
            add_active_keys_and_collect_new(
                &mut self.active_key_ref_counts,
                &product.unique_keys,
                &mut newly_active_keys,
            );
            self.products.push(product);
        }
        self.frame_generation = Some(frame.generation());

        Some(SdfAtlasSegmentDelta {
            changed_segment_indices: journal.changed_segment_indices().to_vec(),
            appended_segment_start,
            appended_segment_count: journal.appended_segment_count(),
            truncated_segment_count,
            newly_active_keys,
            glyph_key_visit_count,
            run_entry_visit_count,
        })
    }

    pub(super) fn active_keys(&self) -> BTreeSet<SdfAtlasGlyphKey> {
        self.active_key_ref_counts.keys().cloned().collect()
    }

    pub(super) fn rebuild_runs<Resolve>(&mut self, mut resolve: Resolve) -> Vec<SdfAtlasRun>
    where
        Resolve: FnMut(&[Option<SdfAtlasGlyphKey>]) -> SdfAtlasRun,
    {
        self.run_ranges.clear();
        let mut runs = Vec::new();
        for product in &self.products {
            let start = runs.len();
            runs.extend(product.run_keys.iter().map(|keys| resolve(keys)));
            self.run_ranges.push(start..runs.len());
        }
        runs
    }

    pub(super) fn patch_runs<Resolve>(
        &mut self,
        runs: &mut Vec<SdfAtlasRun>,
        delta: &SdfAtlasSegmentDelta,
        mut resolve: Resolve,
    ) -> bool
    where
        Resolve: FnMut(&[Option<SdfAtlasGlyphKey>]) -> SdfAtlasRun,
    {
        if self
            .run_ranges
            .len()
            .saturating_sub(delta.truncated_segment_count)
            != delta.appended_segment_start
        {
            return false;
        }
        if delta.truncated_segment_count > 0 {
            let truncate_run_start = self
                .run_ranges
                .get(delta.appended_segment_start)
                .map_or(runs.len(), |range| range.start);
            runs.truncate(truncate_run_start);
            self.run_ranges.truncate(delta.appended_segment_start);
        }

        for &index in &delta.changed_segment_indices {
            let Some(previous_range) = self.run_ranges.get(index).cloned() else {
                return false;
            };
            let replacements = self.products[index]
                .run_keys
                .iter()
                .map(|keys| resolve(keys))
                .collect::<Vec<_>>();
            let previous_len = previous_range.len();
            let replacement_len = replacements.len();
            runs.splice(previous_range.clone(), replacements);
            self.run_ranges[index] =
                previous_range.start..previous_range.start.saturating_add(replacement_len);
            shift_run_ranges(
                &mut self.run_ranges[index + 1..],
                previous_len,
                replacement_len,
            );
        }

        for index in delta.appended_segment_start
            ..delta
                .appended_segment_start
                .saturating_add(delta.appended_segment_count)
        {
            let start = runs.len();
            runs.extend(
                self.products[index]
                    .run_keys
                    .iter()
                    .map(|keys| resolve(keys)),
            );
            self.run_ranges.push(start..runs.len());
        }
        true
    }
}

impl SdfAtlasSegmentDelta {
    pub(super) fn newly_active_keys(&self) -> &BTreeSet<SdfAtlasGlyphKey> {
        &self.newly_active_keys
    }

    pub(super) fn segment_visit_count(&self) -> usize {
        self.changed_segment_indices
            .len()
            .saturating_add(self.appended_segment_count)
            .saturating_add(self.truncated_segment_count)
    }

    pub(super) fn glyph_key_visit_count(&self) -> usize {
        self.glyph_key_visit_count
    }

    pub(super) fn run_entry_visit_count(&self) -> usize {
        self.run_entry_visit_count
    }
}

impl SdfAtlasSegmentProduct {
    fn from_frame_segment(
        texts: &[crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiTextBatch],
    ) -> Self {
        let (unique_keys, run_keys) = collect_sdf_atlas_text_keys_iter(texts.iter());
        Self {
            unique_keys,
            run_keys,
        }
    }
}

fn add_active_keys(
    active_key_ref_counts: &mut HashMap<SdfAtlasGlyphKey, usize>,
    keys: &BTreeSet<SdfAtlasGlyphKey>,
) {
    for key in keys {
        let count = active_key_ref_counts.entry(key.clone()).or_default();
        *count = count.saturating_add(1);
    }
}

fn add_active_keys_and_collect_new(
    active_key_ref_counts: &mut HashMap<SdfAtlasGlyphKey, usize>,
    keys: &BTreeSet<SdfAtlasGlyphKey>,
    newly_active_keys: &mut BTreeSet<SdfAtlasGlyphKey>,
) {
    for key in keys {
        let count = active_key_ref_counts.entry(key.clone()).or_default();
        if *count == 0 {
            newly_active_keys.insert(key.clone());
        }
        *count = count.saturating_add(1);
    }
}

fn remove_active_keys(
    active_key_ref_counts: &mut HashMap<SdfAtlasGlyphKey, usize>,
    keys: &BTreeSet<SdfAtlasGlyphKey>,
) {
    for key in keys {
        let Some(count) = active_key_ref_counts.get_mut(key) else {
            continue;
        };
        *count = count.saturating_sub(1);
        if *count == 0 {
            active_key_ref_counts.remove(key);
        }
    }
}

fn shift_run_ranges(ranges: &mut [Range<usize>], old_len: usize, new_len: usize) {
    if old_len == new_len {
        return;
    }
    if new_len > old_len {
        let delta = new_len - old_len;
        for range in ranges {
            range.start = range.start.saturating_add(delta);
            range.end = range.end.saturating_add(delta);
        }
    } else {
        let delta = old_len - new_len;
        for range in ranges {
            range.start = range.start.saturating_sub(delta);
            range.end = range.end.saturating_sub(delta);
        }
    }
}
