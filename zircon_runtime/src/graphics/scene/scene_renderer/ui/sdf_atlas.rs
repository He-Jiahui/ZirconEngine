use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use super::render::ScreenSpaceUiTextBatch;
use super::text::{ScreenSpaceUiTextFrameProduct, ScreenSpaceUiTextFrameProductGeneration};
use crate::core::math::UVec2;
use crate::text::atlas::{
    GlyphAtlasAllocation, GlyphAtlasDirtyPage, GlyphAtlasFormat, GlyphAtlasPageKey,
    GlyphAtlasPageReservation, GlyphAtlasPageResidencyDecision, GlyphAtlasRect, GlyphAtlasSet,
    GlyphAtlasShelfAllocator, GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
};
use crate::text::sdf::{
    SdfAtlasGlyphGenerationFailure, SdfAtlasGlyphKey, SdfAtlasRect, SdfAtlasSlot, SdfBakeParams,
    SdfGlyphGenerationError,
};

#[path = "sdf_atlas/generation_failures.rs"]
mod generation_failures;
#[path = "sdf_atlas/prepared_texts.rs"]
mod prepared_texts;
#[path = "sdf_atlas/segment_product.rs"]
mod segment_product;
#[path = "sdf_atlas/text_keys.rs"]
mod text_keys;

use prepared_texts::PreparedSdfAtlasTexts;
use segment_product::{SdfAtlasSegmentDelta, SdfAtlasSegmentProductIndex};
use text_keys::collect_sdf_atlas_text_keys_iter;

const SDF_ATLAS_SLOT_SIZE_PX: u32 = 64;
const SDF_ATLAS_MIN_GRID_SIDE: u32 = 8;
const SDF_ATLAS_MAX_CACHED_SLOT_COUNT: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SdfAtlasQuality {
    pub(super) slot_size_px: u32,
    pub(super) min_grid_side: u32,
    pub(super) max_cached_slot_count: usize,
}

impl Default for SdfAtlasQuality {
    fn default() -> Self {
        Self {
            slot_size_px: SDF_ATLAS_SLOT_SIZE_PX,
            min_grid_side: SDF_ATLAS_MIN_GRID_SIDE,
            max_cached_slot_count: SDF_ATLAS_MAX_CACHED_SLOT_COUNT,
        }
    }
}

impl SdfAtlasQuality {
    fn normalized(self) -> Self {
        Self {
            slot_size_px: self.slot_size_px.max(1),
            min_grid_side: self.min_grid_side.max(1),
            max_cached_slot_count: self.max_cached_slot_count.max(1),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfAtlasPlan {
    pub(super) atlas_size: UVec2,
    pub(super) atlas_set: GlyphAtlasSet,
    pub(super) slots: Vec<SdfAtlasSlot>,
    pub(super) runs: Vec<SdfAtlasRun>,
    pub(super) rebuilt_pages: Vec<GlyphAtlasPageKey>,
    pub(super) allocation_failures: Vec<SdfAtlasAllocationFailure>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfAtlasCacheReport {
    pub(super) previous_slot_count: usize,
    pub(super) current_slot_count: usize,
    pub(super) retained_slot_count: usize,
    // A retained key can still move when an earlier inactive slot is evicted.
    // Partial uploads must treat relocated slots as dirty even though the glyph key survived.
    pub(super) stable_slot_count: usize,
    pub(super) relocated_slot_count: usize,
    pub(super) added_slot_count: usize,
    pub(super) evicted_slot_count: usize,
    pub(super) atlas_resized: bool,
    pub(super) dirty_rect: Option<SdfAtlasRect>,
    pub(super) dirty_pages: Vec<SdfAtlasDirtyPageReport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SdfAtlasDirtyPageReport {
    pub(super) page_key: GlyphAtlasPageKey,
    pub(super) dirty_rect: SdfAtlasRect,
}

/// 维护字形槽布局与逐页脏区；像素由文字系统烘焙，调用者在上传成功后确认清脏。
pub(super) struct ScreenSpaceUiSdfAtlas {
    plan: SdfAtlasPlan,
    cached_slots: Vec<SdfAtlasCachedSlot>,
    generation: u64,
    slot_product_generation: u64,
    quality: SdfAtlasQuality,
    prepared_texts: PreparedSdfAtlasTexts,
    segment_products: SdfAtlasSegmentProductIndex,
    slot_index_by_key: HashMap<SdfAtlasGlyphKey, usize>,
    slot_allocators: BTreeMap<GlyphAtlasFormat, Vec<GlyphAtlasShelfAllocator>>,
    retained_frame_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    recorded_generation_failures: Option<Arc<[SdfAtlasGlyphGenerationFailure]>>,
    generation_failures_by_slot: Vec<Option<SdfGlyphGenerationError>>,
    full_page_dirty_until_upload: bool,
    last_report: SdfAtlasCacheReport,
    last_segment_product_visit_count: usize,
    last_glyph_key_visit_count: usize,
    last_run_entry_visit_count: usize,
    last_prepare_was_full_rebuild: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SdfAtlasCachedSlot {
    key: SdfAtlasGlyphKey,
    last_seen_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SdfAtlasAllocationFailure {
    pub(super) key: SdfAtlasGlyphKey,
    pub(super) reason: SdfAtlasAllocationFailureReason,
    pub(super) requested_size: UVec2,
    pub(super) atlas_size: UVec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SdfAtlasAllocationFailureReason {
    PageLimit,
    OversizedSlot,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfAtlasRun {
    pub(super) glyph_slot_indices: Vec<Option<usize>>,
    pub(super) glyph_failure_reasons: Vec<Option<SdfAtlasAllocationFailureReason>>,
    pub(super) glyph_generation_failures: Vec<Option<SdfGlyphGenerationError>>,
    pub(super) allocation_failure_count: usize,
    pub(super) generation_failure_count: usize,
    pub(super) page_limit_failure_count: usize,
    pub(super) oversized_failure_count: usize,
}

impl SdfAtlasRun {
    pub(super) fn has_failures(&self) -> bool {
        self.allocation_failure_count > 0 || self.generation_failure_count > 0
    }
}

impl ScreenSpaceUiSdfAtlas {
    pub(super) fn new() -> Self {
        Self {
            plan: SdfAtlasPlan::default(),
            cached_slots: Vec::new(),
            generation: 0,
            slot_product_generation: 0,
            quality: SdfAtlasQuality::default(),
            prepared_texts: PreparedSdfAtlasTexts::default(),
            segment_products: SdfAtlasSegmentProductIndex::default(),
            slot_index_by_key: HashMap::new(),
            slot_allocators: BTreeMap::new(),
            retained_frame_generation: None,
            recorded_generation_failures: None,
            generation_failures_by_slot: Vec::new(),
            full_page_dirty_until_upload: false,
            last_report: SdfAtlasCacheReport::default(),
            last_segment_product_visit_count: 0,
            last_glyph_key_visit_count: 0,
            last_run_entry_visit_count: 0,
            last_prepare_was_full_rebuild: false,
        }
    }

    pub(super) fn prepare(&mut self, texts: &[ScreenSpaceUiTextBatch]) {
        self.prepare_with_retained_generation(texts, None);
    }

    pub(super) fn prepare_retained(
        &mut self,
        texts: &[ScreenSpaceUiTextBatch],
        generation: ScreenSpaceUiTextFrameProductGeneration,
    ) {
        self.prepare_with_retained_generation(texts, Some(generation));
    }

    pub(super) fn prepare_retained_segments<'a, Segments>(
        &mut self,
        text_segments: Segments,
        generation: ScreenSpaceUiTextFrameProductGeneration,
    ) where
        Segments: Clone + Iterator<Item = &'a [ScreenSpaceUiTextBatch]>,
    {
        self.prepare_with_retained_text_iter(
            text_segments.flat_map(|segment| segment.iter()),
            Some(generation),
        );
    }

    pub(super) fn prepare_retained_frame(&mut self, frame: &ScreenSpaceUiTextFrameProduct) {
        crate::profile_scope!("runtime", "ui_text.sdf_prepare", "sdf_atlas_plan");
        if self.segment_products.frame_matches(frame.generation()) {
            self.last_report = stable_cache_report(&self.plan);
            self.record_segment_product_profile(0, 0, 0, false);
            return;
        }
        if let Some(delta) = self.segment_products.apply_local(frame) {
            if self.apply_local_segment_delta(&delta) {
                self.prepared_texts.clear();
                self.retained_frame_generation = Some(frame.generation());
                self.recorded_generation_failures = None;
                self.record_segment_product_profile(
                    delta.segment_visit_count(),
                    delta.glyph_key_visit_count(),
                    delta.run_entry_visit_count(),
                    false,
                );
                return;
            }
        }
        // 局部路径失败前可能已修改槽或 run，必须立即完整重建以恢复相互匹配的索引与脏页。
        self.rebuild_retained_frame(frame);
        let glyph_key_visit_count = self.segment_products.active_keys().len();
        self.record_segment_product_profile(
            frame.segment_products().len(),
            glyph_key_visit_count,
            self.plan
                .runs
                .iter()
                .map(|run| run.glyph_slot_indices.len())
                .sum(),
            true,
        );
    }

    fn prepare_with_retained_generation(
        &mut self,
        texts: &[ScreenSpaceUiTextBatch],
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) {
        self.prepare_with_retained_text_iter(texts.iter(), retained_generation);
    }

    fn prepare_with_retained_text_iter<'a, Texts>(
        &mut self,
        texts: Texts,
        retained_generation: Option<ScreenSpaceUiTextFrameProductGeneration>,
    ) where
        Texts: Clone + Iterator<Item = &'a ScreenSpaceUiTextBatch>,
    {
        crate::profile_scope!("runtime", "ui_text.sdf_prepare", "sdf_atlas_plan");
        if retained_generation.is_some() && self.retained_frame_generation == retained_generation {
            self.last_report = stable_cache_report(&self.plan);
            return;
        }
        // BUG: [CR-R02-runtime_wave12_graphics_ui_atlas_sdf-0002] 字体失效后尚未确认上传而再次准备相同文本时，此快路径返回空脏页；stable_cache_report 清空 dirty_pages，font_face_invalidation_rebuilds_stable_slots_as_dirty_pages 在再次准备后要求非空脏页，与此返回值矛盾。
        if self.prepared_texts.matches_iter(texts.clone()) {
            self.retained_frame_generation = retained_generation;
            self.last_report = stable_cache_report(&self.plan);
            return;
        }
        self.prepared_texts.replace_iter(texts.clone());
        self.segment_products = SdfAtlasSegmentProductIndex::default();
        self.retained_frame_generation = retained_generation;
        let (current_keys, run_keys) = collect_sdf_atlas_text_keys_iter(texts);
        let mut next_plan = if current_keys.is_empty() {
            self.cached_slots.clear();
            plan_sdf_atlas_from_slot_keys(Vec::new(), run_keys, self.quality)
        } else {
            self.generation = self.generation.saturating_add(1).max(1);
            retain_current_slots(&mut self.cached_slots, &current_keys, self.generation);
            insert_new_slots(&mut self.cached_slots, &current_keys, self.generation);
            evict_inactive_slots(&mut self.cached_slots, &current_keys, self.quality);
            plan_sdf_atlas_from_slot_keys(
                self.cached_slots
                    .iter()
                    .map(|slot| slot.key.clone())
                    .collect(),
                run_keys,
                self.quality,
            )
        };
        if self.full_page_dirty_until_upload && !next_plan.slots.is_empty() {
            next_plan.rebuilt_pages = next_plan
                .slots
                .iter()
                .map(|slot| slot.page_key)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
        }
        self.last_report = cache_report_for_plan_transition(&self.plan, &next_plan);
        if self.plan.atlas_size != next_plan.atlas_size || self.plan.slots != next_plan.slots {
            self.slot_product_generation = self.slot_product_generation.saturating_add(1).max(1);
        }
        self.plan = next_plan;
        self.rebuild_slot_lookup_and_allocators();
        self.recorded_generation_failures = None;
    }

    fn rebuild_retained_frame(&mut self, frame: &ScreenSpaceUiTextFrameProduct) {
        self.segment_products.rebuild(frame);
        let current_keys = self.segment_products.active_keys();
        self.prepared_texts.clear();
        self.retained_frame_generation = Some(frame.generation());
        let mut next_plan = if current_keys.is_empty() {
            self.cached_slots.clear();
            plan_sdf_atlas_from_slot_keys(Vec::new(), Vec::new(), self.quality)
        } else {
            self.generation = self.generation.saturating_add(1).max(1);
            retain_current_slots(&mut self.cached_slots, &current_keys, self.generation);
            insert_new_slots(&mut self.cached_slots, &current_keys, self.generation);
            evict_inactive_slots(&mut self.cached_slots, &current_keys, self.quality);
            plan_sdf_atlas_from_slot_keys(
                self.cached_slots
                    .iter()
                    .map(|slot| slot.key.clone())
                    .collect(),
                Vec::new(),
                self.quality,
            )
        };
        self.rebuild_slot_lookup_and_allocators_for_plan(&next_plan);
        let failure_reasons = allocation_failure_map(&next_plan.allocation_failures);
        let slot_index_by_key = &self.slot_index_by_key;
        next_plan.runs = self.segment_products.rebuild_runs(|keys| {
            sdf_atlas_run_for_glyph_keys(keys, slot_index_by_key, &failure_reasons)
        });
        if self.full_page_dirty_until_upload && !next_plan.slots.is_empty() {
            next_plan.rebuilt_pages = next_plan
                .slots
                .iter()
                .map(|slot| slot.page_key)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
        }
        self.last_report = cache_report_for_plan_transition(&self.plan, &next_plan);
        if self.plan.atlas_size != next_plan.atlas_size || self.plan.slots != next_plan.slots {
            self.slot_product_generation = self.slot_product_generation.saturating_add(1).max(1);
        }
        self.plan = next_plan;
        self.recorded_generation_failures = None;
    }

    fn apply_local_segment_delta(&mut self, delta: &SdfAtlasSegmentDelta) -> bool {
        if self.full_page_dirty_until_upload
            || self.cached_slots.len() != self.plan.slots.len()
            || self.slot_index_by_key.len() != self.plan.slots.len()
        {
            return false;
        }
        let missing_key_count = delta
            .newly_active_keys()
            .iter()
            .filter(|key| !self.slot_index_by_key.contains_key(*key))
            .count();
        if missing_key_count > 0
            && self.plan.atlas_size != atlas_page_size_for_quality(self.quality)
        {
            return false;
        }
        if self.plan.slots.len().saturating_add(missing_key_count)
            > self.quality.normalized().max_cached_slot_count
        {
            return false;
        }

        self.generation = self.generation.saturating_add(1).max(1);
        self.plan.rebuilt_pages.clear();
        let previous_slot_count = self.plan.slots.len();
        let previous_layer_count = sdf_atlas_layer_count(&self.plan);
        let mut added_slots = Vec::new();
        let newly_active_keys = delta
            .newly_active_keys()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        self.plan
            .allocation_failures
            .retain(|failure| !delta.newly_active_keys().contains(&failure.key));
        for key in newly_active_keys {
            if let Some(&slot_index) = self.slot_index_by_key.get(&key) {
                if let Some(slot) = self.cached_slots.get_mut(slot_index) {
                    slot.last_seen_generation = self.generation;
                }
                continue;
            }
            match allocate_retained_sdf_slot(
                &mut self.plan.atlas_set,
                &mut self.slot_allocators,
                key.clone(),
                self.plan.atlas_size,
                self.quality,
            ) {
                Ok(slot) => {
                    let slot_index = self.plan.slots.len();
                    self.slot_index_by_key.insert(key.clone(), slot_index);
                    self.cached_slots.push(SdfAtlasCachedSlot {
                        key,
                        last_seen_generation: self.generation,
                    });
                    self.plan.slots.push(slot.clone());
                    added_slots.push(slot);
                }
                Err(_) => return false,
            }
        }
        let failure_reasons = allocation_failure_map(&self.plan.allocation_failures);
        let slot_index_by_key = &self.slot_index_by_key;
        if !self
            .segment_products
            .patch_runs(&mut self.plan.runs, delta, |keys| {
                sdf_atlas_run_for_glyph_keys(keys, slot_index_by_key, &failure_reasons)
            })
        {
            return false;
        }
        if !added_slots.is_empty() {
            self.slot_product_generation = self.slot_product_generation.saturating_add(1).max(1);
        }
        self.last_report = cache_report_for_appended_slots(
            previous_slot_count,
            previous_layer_count,
            &self.plan,
            &added_slots,
        );
        true
    }

    fn rebuild_slot_lookup_and_allocators(&mut self) {
        let plan = self.plan.clone();
        self.rebuild_slot_lookup_and_allocators_for_plan(&plan);
    }

    fn rebuild_slot_lookup_and_allocators_for_plan(&mut self, plan: &SdfAtlasPlan) {
        self.slot_index_by_key.clear();
        self.slot_allocators.clear();
        for (slot_index, slot) in plan.slots.iter().enumerate() {
            self.slot_index_by_key.insert(slot.key.clone(), slot_index);
            let allocators = self
                .slot_allocators
                .entry(slot.page_key.format)
                .or_default();
            while allocators.len() <= slot.page_key.page_index as usize {
                let page_index = allocators.len() as u32;
                allocators.push(GlyphAtlasShelfAllocator::new(
                    GlyphAtlasPageKey::new(slot.page_key.format, page_index),
                    plan.atlas_size,
                    0,
                ));
            }
            let allocation = allocators[slot.page_key.page_index as usize]
                .allocate(UVec2::splat(self.quality.normalized().slot_size_px));
            debug_assert_eq!(
                allocation.map(|allocation| allocation.rect),
                Some(GlyphAtlasRect::from(slot.rect))
            );
        }
    }

    pub(super) fn invalidate_font_faces(&mut self) {
        self.slot_product_generation = self.slot_product_generation.saturating_add(1).max(1);
        self.plan = SdfAtlasPlan::default();
        self.cached_slots.clear();
        self.prepared_texts.clear();
        self.segment_products = SdfAtlasSegmentProductIndex::default();
        self.slot_index_by_key.clear();
        self.slot_allocators.clear();
        self.retained_frame_generation = None;
        self.recorded_generation_failures = None;
        self.generation_failures_by_slot.clear();
        self.full_page_dirty_until_upload = true;
        self.last_report = SdfAtlasCacheReport::default();
    }

    pub(super) fn mark_prepared_pages_uploaded(&mut self) {
        if !self.plan.slots.is_empty() {
            self.full_page_dirty_until_upload = false;
        }
    }

    pub(super) fn plan(&self) -> &SdfAtlasPlan {
        &self.plan
    }

    pub(super) fn slot_product_generation(&self) -> u64 {
        self.slot_product_generation
    }

    pub(super) fn cache_report(&self) -> SdfAtlasCacheReport {
        self.last_report.clone()
    }

    #[cfg(test)]
    pub(super) fn segment_product_visit_report(&self) -> (usize, usize, usize, bool) {
        (
            self.last_segment_product_visit_count,
            self.last_glyph_key_visit_count,
            self.last_run_entry_visit_count,
            self.last_prepare_was_full_rebuild,
        )
    }

    fn record_segment_product_profile(
        &mut self,
        segment_visit_count: usize,
        glyph_key_visit_count: usize,
        run_entry_visit_count: usize,
        full_rebuild: bool,
    ) {
        self.last_segment_product_visit_count = segment_visit_count;
        self.last_glyph_key_visit_count = glyph_key_visit_count;
        self.last_run_entry_visit_count = run_entry_visit_count;
        self.last_prepare_was_full_rebuild = full_rebuild;
        crate::core::diagnostics::profiling::record_counter_batch(
            "runtime",
            &[
                (
                    "ui_text.sdf_atlas.segment_product_visit_count",
                    segment_visit_count as f64,
                ),
                (
                    "ui_text.sdf_atlas.glyph_key_visit_count",
                    glyph_key_visit_count as f64,
                ),
                (
                    "ui_text.sdf_atlas.run_entry_visit_count",
                    run_entry_visit_count as f64,
                ),
                (
                    "ui_text.sdf_atlas.full_rebuild_count",
                    usize::from(full_rebuild) as f64,
                ),
            ],
        );
    }

    pub(super) fn discard_cached_slots_not_in_texts(&mut self, texts: &[ScreenSpaceUiTextBatch]) {
        let (current_keys, _) = collect_sdf_atlas_text_keys_iter(texts.iter());
        self.cached_slots
            .retain(|slot| current_keys.contains(&slot.key));
    }

    #[cfg(test)]
    pub(super) fn slot_count(&self) -> usize {
        self.plan.slots.len()
    }

    #[cfg(test)]
    pub(super) fn run_count(&self) -> usize {
        self.plan.runs.len()
    }
}

fn stable_cache_report(plan: &SdfAtlasPlan) -> SdfAtlasCacheReport {
    SdfAtlasCacheReport {
        previous_slot_count: plan.slots.len(),
        current_slot_count: plan.slots.len(),
        retained_slot_count: plan.slots.len(),
        stable_slot_count: plan.slots.len(),
        relocated_slot_count: 0,
        added_slot_count: 0,
        evicted_slot_count: 0,
        atlas_resized: false,
        dirty_rect: None,
        dirty_pages: Vec::new(),
    }
}

fn retain_current_slots(
    cached_slots: &mut [SdfAtlasCachedSlot],
    current_keys: &BTreeSet<SdfAtlasGlyphKey>,
    generation: u64,
) {
    for slot in cached_slots {
        if current_keys.contains(&slot.key) {
            slot.last_seen_generation = generation;
        }
    }
}

fn insert_new_slots(
    cached_slots: &mut Vec<SdfAtlasCachedSlot>,
    current_keys: &BTreeSet<SdfAtlasGlyphKey>,
    generation: u64,
) {
    let new_keys = {
        let cached_keys = cached_slots
            .iter()
            .map(|slot| &slot.key)
            .collect::<BTreeSet<&SdfAtlasGlyphKey>>();
        current_keys
            .iter()
            .filter(|key| !cached_keys.contains(key))
            .collect::<Vec<_>>()
    };
    for key in new_keys {
        cached_slots.push(SdfAtlasCachedSlot {
            key: key.clone(),
            last_seen_generation: generation,
        });
    }
}

fn evict_inactive_slots(
    cached_slots: &mut Vec<SdfAtlasCachedSlot>,
    current_keys: &BTreeSet<SdfAtlasGlyphKey>,
    quality: SdfAtlasQuality,
) {
    let quality = quality.normalized();
    let target_slot_count = quality.max_cached_slot_count.max(current_keys.len());
    if cached_slots.len() <= target_slot_count {
        return;
    }

    let mut inactive_indices = cached_slots
        .iter()
        .enumerate()
        .filter(|(_, slot)| !current_keys.contains(&slot.key))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    inactive_indices.sort_by(|left_index, right_index| {
        let left = &cached_slots[*left_index];
        let right = &cached_slots[*right_index];
        left.last_seen_generation
            .cmp(&right.last_seen_generation)
            .then_with(|| left.key.cmp(&right.key))
            .then_with(|| left_index.cmp(right_index))
    });

    let evict_count = cached_slots.len() - target_slot_count;
    let evicted_indices = inactive_indices
        .iter()
        .take(evict_count)
        .copied()
        .collect::<BTreeSet<_>>();
    let mut index = 0;
    cached_slots.retain(|_| {
        let keep = !evicted_indices.contains(&index);
        index += 1;
        keep
    });
}

fn cache_report_for_plan_transition(
    previous: &SdfAtlasPlan,
    current: &SdfAtlasPlan,
) -> SdfAtlasCacheReport {
    let previous_keys = previous
        .slots
        .iter()
        .map(|slot| &slot.key)
        .collect::<BTreeSet<&SdfAtlasGlyphKey>>();
    let current_keys = current
        .slots
        .iter()
        .map(|slot| &slot.key)
        .collect::<BTreeSet<&SdfAtlasGlyphKey>>();
    let previous_slots = previous
        .slots
        .iter()
        .map(|slot| (&slot.key, (slot.page_key, slot.rect)))
        .collect::<BTreeMap<&SdfAtlasGlyphKey, _>>();
    let current_slots = current
        .slots
        .iter()
        .map(|slot| (&slot.key, (slot.page_key, slot.rect)))
        .collect::<BTreeMap<&SdfAtlasGlyphKey, _>>();
    let retained_slot_count = current_keys.intersection(&previous_keys).count();
    let stable_slot_count = current_keys
        .intersection(&previous_keys)
        .filter(|key| previous_slots.get(*key) == current_slots.get(*key))
        .count();
    let relocated_slot_count = retained_slot_count.saturating_sub(stable_slot_count);
    let added_slot_count = current_keys.difference(&previous_keys).count();
    let evicted_slot_count = previous_keys.difference(&current_keys).count();
    let atlas_resized = previous.atlas_size != current.atlas_size
        || sdf_atlas_layer_count(previous) != sdf_atlas_layer_count(current);
    let rebuilt_pages = current
        .rebuilt_pages
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let dirty_pages =
        dirty_pages_for_plan_transition(current, &previous_slots, atlas_resized, &rebuilt_pages);
    let dirty_rect = dirty_pages
        .iter()
        .find(|page| page.page_key == GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0))
        .map(|page| page.dirty_rect);

    SdfAtlasCacheReport {
        previous_slot_count: previous.slots.len(),
        current_slot_count: current.slots.len(),
        retained_slot_count,
        stable_slot_count,
        relocated_slot_count,
        added_slot_count,
        evicted_slot_count,
        atlas_resized,
        dirty_rect,
        dirty_pages,
    }
}

fn cache_report_for_appended_slots(
    previous_slot_count: usize,
    previous_layer_count: u32,
    current: &SdfAtlasPlan,
    added_slots: &[SdfAtlasSlot],
) -> SdfAtlasCacheReport {
    let atlas_resized = previous_layer_count != sdf_atlas_layer_count(current);
    let dirty_pages = if atlas_resized {
        let rebuilt_pages = BTreeSet::new();
        dirty_pages_for_plan_transition(current, &BTreeMap::new(), true, &rebuilt_pages)
    } else {
        dirty_pages_for_added_slots(added_slots)
    };
    let dirty_rect = dirty_pages
        .iter()
        .find(|page| page.page_key == GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0))
        .map(|page| page.dirty_rect);
    SdfAtlasCacheReport {
        previous_slot_count,
        current_slot_count: current.slots.len(),
        retained_slot_count: previous_slot_count,
        stable_slot_count: previous_slot_count,
        relocated_slot_count: 0,
        added_slot_count: added_slots.len(),
        evicted_slot_count: 0,
        atlas_resized,
        dirty_rect,
        dirty_pages,
    }
}

fn dirty_pages_for_added_slots(added_slots: &[SdfAtlasSlot]) -> Vec<SdfAtlasDirtyPageReport> {
    let mut dirty_pages = BTreeMap::<GlyphAtlasPageKey, GlyphAtlasDirtyPage>::new();
    for slot in added_slots {
        dirty_pages
            .entry(slot.page_key)
            .or_insert_with(|| GlyphAtlasDirtyPage::new(slot.page_key))
            .mark_dirty(slot.page_key, GlyphAtlasRect::from(slot.rect));
    }
    dirty_pages
        .into_iter()
        .filter_map(|(page_key, dirty_page)| {
            dirty_page
                .merged_rect()
                .map(|dirty_rect| SdfAtlasDirtyPageReport {
                    page_key,
                    dirty_rect: dirty_rect.into(),
                })
        })
        .collect()
}

fn allocation_failure_map(
    failures: &[SdfAtlasAllocationFailure],
) -> HashMap<SdfAtlasGlyphKey, SdfAtlasAllocationFailureReason> {
    failures
        .iter()
        .map(|failure| (failure.key.clone(), failure.reason))
        .collect()
}

fn dirty_pages_for_plan_transition(
    current: &SdfAtlasPlan,
    previous_slots: &BTreeMap<&SdfAtlasGlyphKey, (GlyphAtlasPageKey, SdfAtlasRect)>,
    atlas_resized: bool,
    rebuilt_pages: &BTreeSet<GlyphAtlasPageKey>,
) -> Vec<SdfAtlasDirtyPageReport> {
    let mut dirty_pages = BTreeMap::<GlyphAtlasPageKey, GlyphAtlasDirtyPage>::new();
    for page_key in rebuilt_pages {
        dirty_pages
            .entry(*page_key)
            .or_insert_with(|| GlyphAtlasDirtyPage::new(*page_key))
            .mark_dirty(*page_key, full_rect_for_page(current, *page_key));
    }
    for slot in &current.slots {
        let dirty = !rebuilt_pages.contains(&slot.page_key)
            && (atlas_resized
                || previous_slots
                    .get(&slot.key)
                    .map(|previous_slot| *previous_slot != (slot.page_key, slot.rect))
                    .unwrap_or(true));
        if dirty {
            dirty_pages
                .entry(slot.page_key)
                .or_insert_with(|| GlyphAtlasDirtyPage::new(slot.page_key))
                .mark_dirty(slot.page_key, GlyphAtlasRect::from(slot.rect));
        }
    }
    dirty_pages
        .into_iter()
        .filter_map(|(page_key, dirty_page)| {
            dirty_page
                .merged_rect()
                .map(|dirty_rect| SdfAtlasDirtyPageReport {
                    page_key,
                    dirty_rect: dirty_rect.into(),
                })
        })
        .collect()
}

fn full_rect_for_page(plan: &SdfAtlasPlan, page_key: GlyphAtlasPageKey) -> GlyphAtlasRect {
    let size = plan
        .atlas_set
        .page(page_key.format, page_key.page_index)
        .map(|page| page.size)
        .unwrap_or(plan.atlas_size);
    GlyphAtlasRect {
        x: 0,
        y: 0,
        width: size.x.max(1),
        height: size.y.max(1),
    }
}

pub(super) fn plan_sdf_atlas(texts: &[ScreenSpaceUiTextBatch]) -> SdfAtlasPlan {
    plan_sdf_atlas_with_quality(texts, SdfAtlasQuality::default())
}

fn plan_sdf_atlas_with_quality(
    texts: &[ScreenSpaceUiTextBatch],
    quality: SdfAtlasQuality,
) -> SdfAtlasPlan {
    let (unique_keys, run_keys) = collect_sdf_atlas_text_keys_iter(texts.iter());
    plan_sdf_atlas_from_slot_keys(unique_keys.into_iter().collect(), run_keys, quality)
}

fn plan_sdf_atlas_from_slot_keys(
    slot_keys: Vec<SdfAtlasGlyphKey>,
    run_keys: Vec<Vec<Option<SdfAtlasGlyphKey>>>,
    quality: SdfAtlasQuality,
) -> SdfAtlasPlan {
    let quality = quality.normalized();
    let (atlas_size, atlas_set, slots, rebuilt_pages, allocation_failures) = if slot_keys.is_empty()
    {
        (
            UVec2::new(1, 1),
            GlyphAtlasSet::default(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    } else {
        let atlas_size = atlas_page_size_for_quality(quality);
        let (atlas_set, slots, rebuilt_pages, allocation_failures) =
            allocate_sdf_atlas_slots(slot_keys, atlas_size, quality);
        (
            atlas_size,
            atlas_set,
            slots,
            rebuilt_pages,
            allocation_failures,
        )
    };
    let slot_by_glyph = slots
        .iter()
        .enumerate()
        .map(|(slot_index, slot)| (slot.key.clone(), slot_index))
        .collect::<HashMap<_, _>>();
    let failure_reasons = allocation_failure_map(&allocation_failures);
    let runs = run_keys
        .into_iter()
        .map(|glyph_keys| {
            sdf_atlas_run_for_glyph_keys(&glyph_keys, &slot_by_glyph, &failure_reasons)
        })
        .collect();

    SdfAtlasPlan {
        atlas_size,
        atlas_set,
        slots,
        runs,
        rebuilt_pages,
        allocation_failures,
    }
}

fn sdf_atlas_run_for_glyph_keys(
    glyph_keys: &[Option<SdfAtlasGlyphKey>],
    slot_by_glyph: &HashMap<SdfAtlasGlyphKey, usize>,
    failure_reasons: &HashMap<SdfAtlasGlyphKey, SdfAtlasAllocationFailureReason>,
) -> SdfAtlasRun {
    let mut run = SdfAtlasRun {
        glyph_slot_indices: Vec::with_capacity(glyph_keys.len()),
        glyph_failure_reasons: Vec::with_capacity(glyph_keys.len()),
        glyph_generation_failures: Vec::with_capacity(glyph_keys.len()),
        ..Default::default()
    };

    for key in glyph_keys {
        let Some(key) = key.as_ref() else {
            run.glyph_slot_indices.push(None);
            run.glyph_failure_reasons.push(None);
            run.glyph_generation_failures.push(None);
            continue;
        };
        let slot_index = slot_by_glyph.get(key).copied();
        let failure_reason = if slot_index.is_none() {
            failure_reasons.get(key).copied()
        } else {
            None
        };
        if let Some(reason) = failure_reason {
            run.allocation_failure_count = run.allocation_failure_count.saturating_add(1);
            match reason {
                SdfAtlasAllocationFailureReason::PageLimit => {
                    run.page_limit_failure_count = run.page_limit_failure_count.saturating_add(1);
                }
                SdfAtlasAllocationFailureReason::OversizedSlot => {
                    run.oversized_failure_count = run.oversized_failure_count.saturating_add(1);
                }
            }
        }
        run.glyph_slot_indices.push(slot_index);
        run.glyph_failure_reasons.push(failure_reason);
        run.glyph_generation_failures.push(None);
    }

    run
}

struct SdfAtlasPageAllocation {
    allocation: GlyphAtlasAllocation,
    rebuilt_page: Option<GlyphAtlasPageKey>,
}

fn allocate_retained_sdf_slot(
    atlas_set: &mut GlyphAtlasSet,
    allocators: &mut BTreeMap<GlyphAtlasFormat, Vec<GlyphAtlasShelfAllocator>>,
    key: SdfAtlasGlyphKey,
    atlas_size: UVec2,
    quality: SdfAtlasQuality,
) -> Result<SdfAtlasSlot, SdfAtlasAllocationFailure> {
    let quality = quality.normalized();
    let slot_size = UVec2::splat(quality.slot_size_px);
    if slot_size.x > atlas_size.x || slot_size.y > atlas_size.y {
        return Err(sdf_allocation_failure(
            key,
            SdfAtlasAllocationFailureReason::OversizedSlot,
            slot_size,
            atlas_size,
        ));
    }
    let format = key.bake_params.mode.atlas_format();
    let allocation = allocate_sdf_slot(
        atlas_set,
        allocators.entry(format).or_default(),
        format,
        atlas_size,
        slot_size,
    )
    .map_err(|reason| sdf_allocation_failure(key.clone(), reason, slot_size, atlas_size))?
    .allocation;
    Ok(SdfAtlasSlot {
        key,
        page_key: allocation.page_key,
        rect: allocation.rect.into(),
    })
}

fn allocate_sdf_atlas_slots(
    slot_keys: Vec<SdfAtlasGlyphKey>,
    atlas_size: UVec2,
    quality: SdfAtlasQuality,
) -> (
    GlyphAtlasSet,
    Vec<SdfAtlasSlot>,
    Vec<GlyphAtlasPageKey>,
    Vec<SdfAtlasAllocationFailure>,
) {
    let quality = quality.normalized();
    let mut atlas_set = GlyphAtlasSet::default();
    atlas_set.begin_frame();
    let mut allocators = BTreeMap::<GlyphAtlasFormat, Vec<GlyphAtlasShelfAllocator>>::new();
    let mut slots = Vec::with_capacity(slot_keys.len());
    let mut rebuilt_pages = Vec::new();
    let mut allocation_failures = Vec::new();
    let slot_size = UVec2::splat(quality.slot_size_px);
    for key in slot_keys {
        let format = key.bake_params.mode.atlas_format();
        if slot_size.x > atlas_size.x || slot_size.y > atlas_size.y {
            allocation_failures.push(sdf_allocation_failure(
                key,
                SdfAtlasAllocationFailureReason::OversizedSlot,
                slot_size,
                atlas_size,
            ));
            continue;
        }

        match allocate_sdf_slot(
            &mut atlas_set,
            allocators.entry(format).or_default(),
            format,
            atlas_size,
            slot_size,
        ) {
            Ok(page_allocation) => {
                if let Some(page_key) = page_allocation.rebuilt_page {
                    rebuilt_pages.push(page_key);
                }
                let allocation = page_allocation.allocation;
                slots.push(SdfAtlasSlot {
                    key,
                    page_key: allocation.page_key,
                    rect: SdfAtlasRect::from(allocation.rect),
                });
            }
            Err(reason) => {
                allocation_failures
                    .push(sdf_allocation_failure(key, reason, slot_size, atlas_size));
            }
        }
    }
    (atlas_set, slots, rebuilt_pages, allocation_failures)
}

fn sdf_allocation_failure(
    key: SdfAtlasGlyphKey,
    reason: SdfAtlasAllocationFailureReason,
    requested_size: UVec2,
    atlas_size: UVec2,
) -> SdfAtlasAllocationFailure {
    SdfAtlasAllocationFailure {
        key,
        reason,
        requested_size,
        atlas_size,
    }
}

fn allocate_sdf_slot(
    atlas_set: &mut GlyphAtlasSet,
    allocators: &mut Vec<GlyphAtlasShelfAllocator>,
    format: GlyphAtlasFormat,
    atlas_size: UVec2,
    slot_size: UVec2,
) -> Result<SdfAtlasPageAllocation, SdfAtlasAllocationFailureReason> {
    if let Some(page_allocation) = allocate_sdf_slot_on_existing_page(allocators, slot_size) {
        return Ok(page_allocation);
    }

    allocate_sdf_slot_on_new_page(atlas_set, allocators, format, atlas_size, slot_size)
}

fn allocate_sdf_slot_on_existing_page(
    allocators: &mut [GlyphAtlasShelfAllocator],
    slot_size: UVec2,
) -> Option<SdfAtlasPageAllocation> {
    allocators
        .last_mut()
        .and_then(|allocator| allocator.allocate(slot_size))
        .map(|allocation| SdfAtlasPageAllocation {
            allocation,
            rebuilt_page: None,
        })
}

fn allocate_sdf_slot_on_new_page(
    atlas_set: &mut GlyphAtlasSet,
    allocators: &mut Vec<GlyphAtlasShelfAllocator>,
    format: GlyphAtlasFormat,
    atlas_size: UVec2,
    slot_size: UVec2,
) -> Result<SdfAtlasPageAllocation, SdfAtlasAllocationFailureReason> {
    let page_reservation: GlyphAtlasPageReservation = atlas_set.reserve_page_for_format(
        format,
        atlas_size,
        0,
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
    );
    let Some(page) = page_reservation.page else {
        return Err(SdfAtlasAllocationFailureReason::PageLimit);
    };
    debug_assert!(matches!(
        page_reservation.decision,
        GlyphAtlasPageResidencyDecision::Allocate(_) | GlyphAtlasPageResidencyDecision::Evict(_)
    ));
    debug_assert_eq!(page.storage_format, format.storage_format());
    let mut allocator = GlyphAtlasShelfAllocator::new(page.key, page.size, 0);
    let Some(allocation) = allocator.allocate(slot_size) else {
        return Err(SdfAtlasAllocationFailureReason::OversizedSlot);
    };
    debug_assert_eq!(allocation.page_key, page.key);
    let _marked = atlas_set.mark_page_used(page.key, 0);
    debug_assert!(_marked);
    let rebuilt_page = match page_reservation.decision {
        GlyphAtlasPageResidencyDecision::Evict(page_key) => Some(page_key),
        GlyphAtlasPageResidencyDecision::Allocate(_) | GlyphAtlasPageResidencyDecision::Blocked => {
            None
        }
    };
    allocators.push(allocator);
    Ok(SdfAtlasPageAllocation {
        allocation,
        rebuilt_page,
    })
}

pub(super) fn sdf_atlas_layer_count(plan: &SdfAtlasPlan) -> u32 {
    distance_field_atlas_layer_count(plan, GlyphAtlasFormat::Sdf)
}

pub(super) fn distance_field_atlas_layer_count(
    plan: &SdfAtlasPlan,
    format: GlyphAtlasFormat,
) -> u32 {
    plan.slots
        .iter()
        .filter(|slot| slot.page_key.format == format)
        .map(|slot| slot.page_key.page_index.saturating_add(1))
        .max()
        .unwrap_or(1)
        .max(1)
}

pub(super) fn distance_field_atlas_page_keys(plan: &SdfAtlasPlan) -> Vec<GlyphAtlasPageKey> {
    plan.slots
        .iter()
        .map(|slot| slot.page_key)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

impl From<GlyphAtlasRect> for SdfAtlasRect {
    fn from(rect: GlyphAtlasRect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl From<SdfAtlasRect> for GlyphAtlasRect {
    fn from(rect: SdfAtlasRect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

fn atlas_page_size_for_quality(quality: SdfAtlasQuality) -> UVec2 {
    let quality = quality.normalized();
    let grid_side = quality.min_grid_side.next_power_of_two();
    UVec2::splat(grid_side * quality.slot_size_px)
}

#[cfg(test)]
mod tests;
