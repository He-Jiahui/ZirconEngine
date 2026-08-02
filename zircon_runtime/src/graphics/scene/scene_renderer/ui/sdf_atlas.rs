use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use super::render::ScreenSpaceUiTextBatch;
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
#[path = "sdf_atlas/text_keys.rs"]
mod text_keys;

use prepared_texts::PreparedSdfAtlasTexts;
use text_keys::collect_sdf_atlas_text_keys;

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

pub(super) struct ScreenSpaceUiSdfAtlas {
    plan: SdfAtlasPlan,
    cached_slots: Vec<SdfAtlasCachedSlot>,
    generation: u64,
    quality: SdfAtlasQuality,
    prepared_texts: PreparedSdfAtlasTexts,
    recorded_generation_failures: Option<Arc<[SdfAtlasGlyphGenerationFailure]>>,
    generation_failures_by_slot: Vec<Option<SdfGlyphGenerationError>>,
    full_page_dirty_until_upload: bool,
    last_report: SdfAtlasCacheReport,
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
            quality: SdfAtlasQuality::default(),
            prepared_texts: PreparedSdfAtlasTexts::default(),
            recorded_generation_failures: None,
            generation_failures_by_slot: Vec::new(),
            full_page_dirty_until_upload: false,
            last_report: SdfAtlasCacheReport::default(),
        }
    }

    pub(super) fn prepare(&mut self, texts: &[ScreenSpaceUiTextBatch]) {
        if self.prepared_texts.matches(texts) {
            self.last_report = stable_cache_report(&self.plan);
            return;
        }
        self.prepared_texts.replace(texts);
        let (current_keys, run_keys) = collect_sdf_atlas_text_keys(texts);
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
        self.plan = next_plan;
        self.recorded_generation_failures = None;
    }

    pub(super) fn invalidate_font_faces(&mut self) {
        self.plan = SdfAtlasPlan::default();
        self.cached_slots.clear();
        self.prepared_texts.clear();
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

    pub(super) fn cache_report(&self) -> SdfAtlasCacheReport {
        self.last_report.clone()
    }

    pub(super) fn discard_cached_slots_not_in_texts(&mut self, texts: &[ScreenSpaceUiTextBatch]) {
        let (current_keys, _) = collect_sdf_atlas_text_keys(texts);
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
    let cached_keys = cached_slots
        .iter()
        .map(|slot| slot.key.clone())
        .collect::<BTreeSet<_>>();
    for key in current_keys {
        if !cached_keys.contains(key) {
            cached_slots.push(SdfAtlasCachedSlot {
                key: key.clone(),
                last_seen_generation: generation,
            });
        }
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
        .map(|(index, slot)| (slot.last_seen_generation, slot.key.clone(), index))
        .collect::<Vec<_>>();
    inactive_indices.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let evict_count = cached_slots.len() - target_slot_count;
    let evicted_indices = inactive_indices
        .iter()
        .take(evict_count)
        .map(|(_, _, index)| *index)
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
        .map(|slot| slot.key.clone())
        .collect::<BTreeSet<_>>();
    let current_keys = current
        .slots
        .iter()
        .map(|slot| slot.key.clone())
        .collect::<BTreeSet<_>>();
    let previous_slots = previous
        .slots
        .iter()
        .map(|slot| (slot.key.clone(), (slot.page_key, slot.rect)))
        .collect::<BTreeMap<_, _>>();
    let current_slots = current
        .slots
        .iter()
        .map(|slot| (slot.key.clone(), (slot.page_key, slot.rect)))
        .collect::<BTreeMap<_, _>>();
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

fn dirty_pages_for_plan_transition(
    current: &SdfAtlasPlan,
    previous_slots: &BTreeMap<SdfAtlasGlyphKey, (GlyphAtlasPageKey, SdfAtlasRect)>,
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
    let (unique_keys, run_keys) = collect_sdf_atlas_text_keys(texts);
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
    let failure_reasons = allocation_failures
        .iter()
        .map(|failure| (failure.key.clone(), failure.reason))
        .collect::<HashMap<_, _>>();
    let runs = run_keys
        .into_iter()
        .map(|glyph_keys| {
            sdf_atlas_run_for_glyph_keys(glyph_keys, &slot_by_glyph, &failure_reasons)
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
    glyph_keys: Vec<Option<SdfAtlasGlyphKey>>,
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
        let Some(key) = key else {
            run.glyph_slot_indices.push(None);
            run.glyph_failure_reasons.push(None);
            run.glyph_generation_failures.push(None);
            continue;
        };
        let slot_index = slot_by_glyph.get(&key).copied();
        let failure_reason = if slot_index.is_none() {
            failure_reasons.get(&key).copied()
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
    debug_assert!(atlas_set.mark_page_used(page.key, 0));
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
