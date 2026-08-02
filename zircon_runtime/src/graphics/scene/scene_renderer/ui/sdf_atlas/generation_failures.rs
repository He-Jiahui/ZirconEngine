use std::sync::Arc;

use crate::text::sdf::SdfAtlasGlyphGenerationFailure;

use super::ScreenSpaceUiSdfAtlas;

impl ScreenSpaceUiSdfAtlas {
    pub(in crate::graphics::scene::scene_renderer::ui) fn record_generation_failures(
        &mut self,
        failures: &Arc<[SdfAtlasGlyphGenerationFailure]>,
    ) {
        if self
            .recorded_generation_failures
            .as_ref()
            .is_some_and(|recorded| Arc::ptr_eq(recorded, failures))
        {
            return;
        }
        self.generation_failures_by_slot.clear();
        self.generation_failures_by_slot
            .resize(self.plan.slots.len(), None);
        for failure in failures.iter() {
            let slot_matches_bake = self
                .plan
                .slots
                .get(failure.slot_index)
                .is_some_and(|slot| slot.key == failure.key);
            if slot_matches_bake {
                self.generation_failures_by_slot[failure.slot_index] = Some(failure.error);
            }
        }
        let failures_by_slot = &self.generation_failures_by_slot;
        for run in &mut self.plan.runs {
            run.glyph_generation_failures = run
                .glyph_slot_indices
                .iter()
                .map(|slot_index| {
                    slot_index.and_then(|index| failures_by_slot.get(index).copied().flatten())
                })
                .collect();
            run.generation_failure_count = run
                .glyph_generation_failures
                .iter()
                .filter(|failure| failure.is_some())
                .count();
        }
        self.recorded_generation_failures = Some(Arc::clone(failures));
    }
}
