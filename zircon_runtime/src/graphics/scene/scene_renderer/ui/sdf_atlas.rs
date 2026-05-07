use std::collections::{BTreeSet, HashMap};

use crate::core::math::UVec2;

use super::render::ScreenSpaceUiTextBatch;

const SDF_ATLAS_SLOT_SIZE_PX: u32 = 64;
const SDF_ATLAS_MIN_GRID_SIDE: u32 = 8;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfAtlasPlan {
    pub(super) atlas_size: UVec2,
    pub(super) slots: Vec<SdfAtlasSlot>,
    pub(super) runs: Vec<SdfAtlasRun>,
}

pub(super) struct ScreenSpaceUiSdfAtlas {
    plan: SdfAtlasPlan,
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
        }
    }

    pub(super) fn prepare(&mut self, texts: &[ScreenSpaceUiTextBatch]) {
        self.plan = plan_sdf_atlas(texts);
    }

    pub(super) fn plan(&self) -> &SdfAtlasPlan {
        &self.plan
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

pub(super) fn plan_sdf_atlas(texts: &[ScreenSpaceUiTextBatch]) -> SdfAtlasPlan {
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

    let mut slot_by_glyph = HashMap::<SdfAtlasGlyphKey, usize>::new();
    let mut slots = Vec::with_capacity(unique_keys.len());
    for key in unique_keys {
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

    let atlas_size = atlas_size_for_slot_count(slots.len());
    assign_slot_rects(&mut slots, atlas_size);
    SdfAtlasPlan {
        atlas_size,
        slots,
        runs,
    }
}

fn font_size_milli(font_size: f32) -> u32 {
    (font_size.max(1.0) * 1000.0).round() as u32
}

fn assign_slot_rects(slots: &mut [SdfAtlasSlot], atlas_size: UVec2) {
    let columns = (atlas_size.x / SDF_ATLAS_SLOT_SIZE_PX).max(1) as usize;
    for (slot_index, slot) in slots.iter_mut().enumerate() {
        slot.rect = slot_rect(slot_index, columns);
    }
}

fn slot_rect(slot_index: usize, columns: usize) -> SdfAtlasRect {
    let x = (slot_index % columns) as u32 * SDF_ATLAS_SLOT_SIZE_PX;
    let y = (slot_index / columns) as u32 * SDF_ATLAS_SLOT_SIZE_PX;
    SdfAtlasRect {
        x,
        y,
        width: SDF_ATLAS_SLOT_SIZE_PX,
        height: SDF_ATLAS_SLOT_SIZE_PX,
    }
}

fn atlas_size_for_slot_count(slot_count: usize) -> UVec2 {
    if slot_count == 0 {
        return UVec2::new(1, 1);
    }

    let required_side = ceil_sqrt(slot_count as u32).max(SDF_ATLAS_MIN_GRID_SIDE);
    let grid_side = required_side.next_power_of_two();
    UVec2::splat(grid_side * SDF_ATLAS_SLOT_SIZE_PX)
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
    use super::{plan_sdf_atlas, ScreenSpaceUiSdfAtlas};
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
    fn sdf_atlas_owner_replaces_previous_frame_plan() {
        let mut atlas = ScreenSpaceUiSdfAtlas::new();

        atlas.prepare(&[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))]);
        assert_eq!(atlas.slot_count(), 2);

        atlas.prepare(&[text_batch("C", UiFrame::new(0.0, 16.0, 12.0, 12.0))]);
        assert_eq!(atlas.slot_count(), 1);
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
}
