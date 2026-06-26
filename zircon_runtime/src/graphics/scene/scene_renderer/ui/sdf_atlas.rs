use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::core::math::UVec2;

use super::render::ScreenSpaceUiTextBatch;

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
    pub(super) slots: Vec<SdfAtlasSlot>,
    pub(super) runs: Vec<SdfAtlasRun>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
}

pub(super) struct ScreenSpaceUiSdfAtlas {
    plan: SdfAtlasPlan,
    cached_slots: Vec<SdfAtlasCachedSlot>,
    generation: u64,
    quality: SdfAtlasQuality,
    last_report: SdfAtlasCacheReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SdfAtlasCachedSlot {
    key: SdfAtlasGlyphKey,
    last_seen_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SdfAtlasSlot {
    pub(super) key: SdfAtlasGlyphKey,
    pub(super) rect: SdfAtlasRect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfAtlasRect {
    pub(super) x: u32,
    pub(super) y: u32,
    pub(super) width: u32,
    pub(super) height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct SdfAtlasGlyphKey {
    pub(super) glyph: char,
    pub(super) font: Option<String>,
    pub(super) font_family: Option<String>,
    pub(super) font_size_milli: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SdfAtlasRun {
    pub(super) glyph_slot_indices: Vec<Option<usize>>,
}

impl ScreenSpaceUiSdfAtlas {
    pub(super) fn new() -> Self {
        Self {
            plan: SdfAtlasPlan::default(),
            cached_slots: Vec::new(),
            generation: 0,
            quality: SdfAtlasQuality::default(),
            last_report: SdfAtlasCacheReport::default(),
        }
    }

    pub(super) fn prepare(&mut self, texts: &[ScreenSpaceUiTextBatch]) {
        let (current_keys, run_keys) = collect_sdf_atlas_text_keys(texts);
        let next_plan = if current_keys.is_empty() {
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
        self.last_report = cache_report_for_plan_transition(&self.plan, &next_plan);
        self.plan = next_plan;
    }

    pub(super) fn plan(&self) -> &SdfAtlasPlan {
        &self.plan
    }

    pub(super) fn cache_report(&self) -> SdfAtlasCacheReport {
        self.last_report
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
    let previous_rects = previous
        .slots
        .iter()
        .map(|slot| (slot.key.clone(), slot.rect))
        .collect::<BTreeMap<_, _>>();
    let current_rects = current
        .slots
        .iter()
        .map(|slot| (slot.key.clone(), slot.rect))
        .collect::<BTreeMap<_, _>>();
    let retained_slot_count = current_keys.intersection(&previous_keys).count();
    let stable_slot_count = current_keys
        .intersection(&previous_keys)
        .filter(|key| previous_rects.get(*key) == current_rects.get(*key))
        .count();
    let relocated_slot_count = retained_slot_count.saturating_sub(stable_slot_count);
    let added_slot_count = current_keys.difference(&previous_keys).count();
    let evicted_slot_count = previous_keys.difference(&current_keys).count();

    SdfAtlasCacheReport {
        previous_slot_count: previous.slots.len(),
        current_slot_count: current.slots.len(),
        retained_slot_count,
        stable_slot_count,
        relocated_slot_count,
        added_slot_count,
        evicted_slot_count,
        atlas_resized: previous.atlas_size != current.atlas_size,
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

fn collect_sdf_atlas_text_keys(
    texts: &[ScreenSpaceUiTextBatch],
) -> (
    BTreeSet<SdfAtlasGlyphKey>,
    Vec<Vec<Option<SdfAtlasGlyphKey>>>,
) {
    let mut unique_keys = BTreeSet::<SdfAtlasGlyphKey>::new();
    let mut run_keys = Vec::with_capacity(texts.len());

    for text in texts {
        let mut glyph_keys = Vec::new();
        for glyph in text.text.chars() {
            if glyph.is_whitespace() {
                glyph_keys.push(None);
                continue;
            }
            let key = SdfAtlasGlyphKey {
                glyph,
                font: text.font.clone(),
                font_family: text.font_family.clone(),
                font_size_milli: font_size_milli(text.font_size),
            };
            unique_keys.insert(key.clone());
            glyph_keys.push(Some(key));
        }
        run_keys.push(glyph_keys);
    }

    (unique_keys, run_keys)
}

fn plan_sdf_atlas_from_slot_keys(
    slot_keys: Vec<SdfAtlasGlyphKey>,
    run_keys: Vec<Vec<Option<SdfAtlasGlyphKey>>>,
    quality: SdfAtlasQuality,
) -> SdfAtlasPlan {
    let quality = quality.normalized();
    let mut slot_by_glyph = HashMap::<SdfAtlasGlyphKey, usize>::new();
    let mut slots = Vec::with_capacity(slot_keys.len());
    for key in slot_keys {
        let slot_index = slots.len();
        slot_by_glyph.insert(key.clone(), slot_index);
        slots.push(SdfAtlasSlot {
            key,
            rect: SdfAtlasRect::default(),
        });
    }
    let runs = run_keys
        .into_iter()
        .map(|glyph_keys| SdfAtlasRun {
            glyph_slot_indices: glyph_keys
                .into_iter()
                .map(|key| key.and_then(|key| slot_by_glyph.get(&key).copied()))
                .collect(),
        })
        .collect();

    let atlas_size = atlas_size_for_slot_count(slots.len(), quality);
    assign_slot_rects(&mut slots, atlas_size, quality);
    SdfAtlasPlan {
        atlas_size,
        slots,
        runs,
    }
}

fn font_size_milli(font_size: f32) -> u32 {
    (font_size.max(1.0) * 1000.0).round() as u32
}

fn assign_slot_rects(slots: &mut [SdfAtlasSlot], atlas_size: UVec2, quality: SdfAtlasQuality) {
    let quality = quality.normalized();
    let columns = (atlas_size.x / quality.slot_size_px).max(1) as usize;
    for (slot_index, slot) in slots.iter_mut().enumerate() {
        slot.rect = slot_rect(slot_index, columns, quality);
    }
}

fn slot_rect(slot_index: usize, columns: usize, quality: SdfAtlasQuality) -> SdfAtlasRect {
    let quality = quality.normalized();
    let x = (slot_index % columns) as u32 * quality.slot_size_px;
    let y = (slot_index / columns) as u32 * quality.slot_size_px;
    SdfAtlasRect {
        x,
        y,
        width: quality.slot_size_px,
        height: quality.slot_size_px,
    }
}

fn atlas_size_for_slot_count(slot_count: usize, quality: SdfAtlasQuality) -> UVec2 {
    let quality = quality.normalized();
    if slot_count == 0 {
        return UVec2::new(1, 1);
    }

    let required_side = ceil_sqrt(slot_count as u32).max(quality.min_grid_side);
    let grid_side = required_side.next_power_of_two();
    UVec2::splat(grid_side * quality.slot_size_px)
}

fn ceil_sqrt(value: u32) -> u32 {
    let mut side = 1;
    while side * side < value {
        side += 1;
    }
    side
}

#[cfg(test)]
mod tests;
