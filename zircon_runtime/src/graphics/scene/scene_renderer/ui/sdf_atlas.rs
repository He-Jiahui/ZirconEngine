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
mod tests {
    use super::{
        plan_sdf_atlas, plan_sdf_atlas_with_quality, ScreenSpaceUiSdfAtlas, SdfAtlasCacheReport,
        SdfAtlasQuality, SDF_ATLAS_MAX_CACHED_SLOT_COUNT,
    };
    use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiTextBatch;
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextWrap};

    #[test]
    fn sdf_atlas_plan_deduplicates_glyph_slots_across_batches() {
        let plan = plan_sdf_atlas(&[
            text_batch("ABBA", UiFrame::new(10.0, 20.0, 40.0, 12.0)),
            text_batch("CAB", UiFrame::new(10.0, 36.0, 30.0, 12.0)),
        ]);

        assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(512));
        assert_eq!(plan.slots.len(), 3);
        assert_eq!(plan.slots[0].key.glyph, 'A');
        assert_eq!(plan.slots[0].rect.x, 0);
        assert_eq!(plan.slots[0].rect.y, 0);
        assert_eq!(plan.slots[1].key.glyph, 'B');
        assert_eq!(plan.slots[1].rect.x, 64);
        assert_eq!(plan.slots[1].rect.y, 0);
        assert_eq!(plan.slots[2].key.glyph, 'C');
        assert_eq!(plan.slots[2].rect.x, 128);
        assert_eq!(plan.slots[2].rect.y, 0);
        assert_eq!(plan.runs.len(), 2);
        assert_eq!(plan.runs[0].glyph_slot_indices, glyph_slots(&[0, 1, 1, 0]));
        assert_eq!(plan.runs[1].glyph_slot_indices, glyph_slots(&[2, 0, 1]));
    }

    #[test]
    fn sdf_atlas_plan_keys_glyph_slots_by_font_identity_and_size() {
        let mut small_default = text_batch("A", UiFrame::new(0.0, 0.0, 12.0, 12.0));
        small_default.font_size = 12.0;
        let mut large_default = text_batch("A", UiFrame::new(0.0, 16.0, 24.0, 24.0));
        large_default.font_size = 24.0;
        let mut small_icon = text_batch("A", UiFrame::new(0.0, 48.0, 12.0, 12.0));
        small_icon.font = Some("res://fonts/icons.font.toml".to_string());
        small_icon.font_family = Some("Zircon Icons".to_string());

        let plan = plan_sdf_atlas(&[small_default, large_default, small_icon]);

        assert_eq!(plan.slots.len(), 3);
        assert_eq!(plan.slots[0].key.glyph, 'A');
        assert_eq!(
            plan.slots[0].key.font.as_deref(),
            Some("res://fonts/default.font.toml")
        );
        assert_eq!(
            plan.slots[0].key.font_family.as_deref(),
            Some("Zircon Sans")
        );
        assert_eq!(plan.slots[0].key.font_size_milli, 12_000);
        assert_eq!(plan.slots[1].key.glyph, 'A');
        assert_eq!(plan.slots[1].key.font_size_milli, 24_000);
        assert_eq!(plan.slots[2].key.glyph, 'A');
        assert_eq!(
            plan.slots[2].key.font.as_deref(),
            Some("res://fonts/icons.font.toml")
        );
        assert_eq!(
            plan.slots[2].key.font_family.as_deref(),
            Some("Zircon Icons")
        );
        assert_eq!(plan.runs[0].glyph_slot_indices, glyph_slots(&[0]));
        assert_eq!(plan.runs[1].glyph_slot_indices, glyph_slots(&[1]));
        assert_eq!(plan.runs[2].glyph_slot_indices, glyph_slots(&[2]));
    }

    #[test]
    fn sdf_atlas_plan_preserves_whitespace_advances_without_slots() {
        let plan = plan_sdf_atlas(&[text_batch("A B", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);

        assert_eq!(plan.slots.len(), 2);
        assert_eq!(plan.slots[0].key.glyph, 'A');
        assert_eq!(plan.slots[1].key.glyph, 'B');
        assert_eq!(
            plan.runs[0].glyph_slot_indices,
            vec![Some(0), None, Some(1)]
        );
    }

    #[test]
    fn sdf_atlas_plan_assigns_slot_rects_by_key_not_batch_order() {
        let first = plan_sdf_atlas(&[text_batch("AB", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);
        let second = plan_sdf_atlas(&[text_batch("BA", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);

        assert_eq!(first.slots, second.slots);
    }

    #[test]
    fn sdf_atlas_quality_controls_slot_size_and_min_grid() {
        let plan = plan_sdf_atlas_with_quality(
            &[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))],
            SdfAtlasQuality {
                slot_size_px: 32,
                min_grid_side: 2,
                max_cached_slot_count: 8,
            },
        );

        assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(64));
        assert_eq!(plan.slots.len(), 2);
        assert_eq!(plan.slots[0].rect.width, 32);
        assert_eq!(plan.slots[0].rect.height, 32);
        assert_eq!(plan.slots[1].rect.x, 32);
        assert_eq!(plan.slots[1].rect.y, 0);
    }

    #[test]
    fn sdf_atlas_owner_retains_inactive_slots_between_non_empty_frames() {
        let mut atlas = ScreenSpaceUiSdfAtlas::new();

        atlas.prepare(&[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))]);
        assert_eq!(atlas.slot_count(), 2);
        assert_eq!(
            atlas.cache_report(),
            SdfAtlasCacheReport {
                previous_slot_count: 0,
                current_slot_count: 2,
                retained_slot_count: 0,
                stable_slot_count: 0,
                relocated_slot_count: 0,
                added_slot_count: 2,
                evicted_slot_count: 0,
                atlas_resized: true,
            }
        );

        atlas.prepare(&[text_batch("C", UiFrame::new(0.0, 16.0, 12.0, 12.0))]);
        assert_eq!(atlas.slot_count(), 3);
        assert_eq!(
            atlas.cache_report(),
            SdfAtlasCacheReport {
                previous_slot_count: 2,
                current_slot_count: 3,
                retained_slot_count: 2,
                stable_slot_count: 2,
                relocated_slot_count: 0,
                added_slot_count: 1,
                evicted_slot_count: 0,
                atlas_resized: false,
            }
        );
    }

    #[test]
    fn sdf_atlas_owner_reports_retained_and_added_slots() {
        let mut atlas = ScreenSpaceUiSdfAtlas::new();

        atlas.prepare(&[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))]);
        atlas.prepare(&[text_batch("BC", UiFrame::new(0.0, 16.0, 24.0, 12.0))]);

        assert_eq!(
            atlas.cache_report(),
            SdfAtlasCacheReport {
                previous_slot_count: 2,
                current_slot_count: 3,
                retained_slot_count: 2,
                stable_slot_count: 2,
                relocated_slot_count: 0,
                added_slot_count: 1,
                evicted_slot_count: 0,
                atlas_resized: false,
            }
        );
    }

    #[test]
    fn sdf_atlas_owner_reuses_retained_slot_without_readding_glyph() {
        let mut atlas = ScreenSpaceUiSdfAtlas::new();

        atlas.prepare(&[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))]);
        atlas.prepare(&[text_batch("C", UiFrame::new(0.0, 16.0, 12.0, 12.0))]);
        atlas.prepare(&[text_batch("A", UiFrame::new(0.0, 32.0, 12.0, 12.0))]);

        assert_eq!(atlas.slot_count(), 3);
        assert_eq!(
            atlas.cache_report(),
            SdfAtlasCacheReport {
                previous_slot_count: 3,
                current_slot_count: 3,
                retained_slot_count: 3,
                stable_slot_count: 3,
                relocated_slot_count: 0,
                added_slot_count: 0,
                evicted_slot_count: 0,
                atlas_resized: false,
            }
        );
        assert_eq!(atlas.plan().runs[0].glyph_slot_indices, glyph_slots(&[0]));
    }

    #[test]
    fn sdf_atlas_owner_clears_previous_plan_for_native_only_frames() {
        let mut atlas = ScreenSpaceUiSdfAtlas::new();

        atlas.prepare(&[text_batch("SDF", UiFrame::new(0.0, 0.0, 36.0, 12.0))]);
        assert_eq!(atlas.slot_count(), 3);
        assert_eq!(atlas.run_count(), 1);

        atlas.prepare(&[]);
        assert_eq!(atlas.slot_count(), 0);
        assert_eq!(atlas.run_count(), 0);
        assert_eq!(
            atlas.cache_report(),
            SdfAtlasCacheReport {
                previous_slot_count: 3,
                current_slot_count: 0,
                retained_slot_count: 0,
                stable_slot_count: 0,
                relocated_slot_count: 0,
                added_slot_count: 0,
                evicted_slot_count: 3,
                atlas_resized: true,
            }
        );
    }

    #[test]
    fn sdf_atlas_owner_preserves_whitespace_runs_without_cache_slots() {
        let mut atlas = ScreenSpaceUiSdfAtlas::new();

        atlas.prepare(&[text_batch("A", UiFrame::new(0.0, 0.0, 12.0, 12.0))]);
        atlas.prepare(&[text_batch("  ", UiFrame::new(0.0, 16.0, 24.0, 12.0))]);

        assert_eq!(atlas.slot_count(), 0);
        assert_eq!(atlas.run_count(), 1);
        assert_eq!(atlas.plan().runs[0].glyph_slot_indices, vec![None, None]);
        assert_eq!(
            atlas.cache_report(),
            SdfAtlasCacheReport {
                previous_slot_count: 1,
                current_slot_count: 0,
                retained_slot_count: 0,
                stable_slot_count: 0,
                relocated_slot_count: 0,
                added_slot_count: 0,
                evicted_slot_count: 1,
                atlas_resized: true,
            }
        );
    }

    #[test]
    fn sdf_atlas_owner_evicts_old_inactive_slots_when_cache_limit_is_exceeded() {
        let mut atlas = ScreenSpaceUiSdfAtlas::new();
        let full_cache = glyph_range_string(0x1000, SDF_ATLAS_MAX_CACHED_SLOT_COUNT);

        atlas.prepare(&[text_batch(
            &full_cache,
            UiFrame::new(0.0, 0.0, 4096.0, 12.0),
        )]);
        assert_eq!(atlas.slot_count(), SDF_ATLAS_MAX_CACHED_SLOT_COUNT);

        atlas.prepare(&[text_batch("\u{2200}", UiFrame::new(0.0, 16.0, 12.0, 12.0))]);

        assert_eq!(atlas.slot_count(), SDF_ATLAS_MAX_CACHED_SLOT_COUNT);
        assert_eq!(
            atlas.cache_report(),
            SdfAtlasCacheReport {
                previous_slot_count: SDF_ATLAS_MAX_CACHED_SLOT_COUNT,
                current_slot_count: SDF_ATLAS_MAX_CACHED_SLOT_COUNT,
                retained_slot_count: SDF_ATLAS_MAX_CACHED_SLOT_COUNT - 1,
                stable_slot_count: 0,
                relocated_slot_count: SDF_ATLAS_MAX_CACHED_SLOT_COUNT - 1,
                added_slot_count: 1,
                evicted_slot_count: 1,
                atlas_resized: false,
            }
        );
    }

    #[test]
    fn sdf_atlas_plan_grows_to_fit_more_than_default_grid() {
        let many_glyphs = (0..70)
            .map(|index| char::from_u32(0x1000 + index).unwrap())
            .collect::<String>();

        let plan = plan_sdf_atlas(&[text_batch(
            &many_glyphs,
            UiFrame::new(0.0, 0.0, 4096.0, 12.0),
        )]);

        assert_eq!(plan.slots.len(), 70);
        assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(1024));
        assert_eq!(plan.slots[64].rect.x, 0);
        assert_eq!(plan.slots[64].rect.y, 256);
    }

    fn text_batch(text: &str, frame: UiFrame) -> ScreenSpaceUiTextBatch {
        ScreenSpaceUiTextBatch {
            text: text.to_string(),
            frame,
            clip_frame: None,
            color: [1.0, 1.0, 1.0, 1.0],
            font: Some("res://fonts/default.font.toml".to_string()),
            font_family: Some("Zircon Sans".to_string()),
            font_size: 12.0,
            line_height: 14.0,
            text_align: UiTextAlign::Left,
            wrap: UiTextWrap::None,
            style: Default::default(),
        }
    }

    fn glyph_slots(indices: &[usize]) -> Vec<Option<usize>> {
        indices.iter().copied().map(Some).collect()
    }

    fn glyph_range_string(start: u32, count: usize) -> String {
        (0..count)
            .map(|index| char::from_u32(start + index as u32).unwrap())
            .collect()
    }
}
