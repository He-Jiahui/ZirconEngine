use std::sync::Arc;

use crate::text::sdf::{SdfAtlasGlyphGenerationFailure, SdfGlyphGenerationError};

use super::ScreenSpaceUiSdfAtlas;

impl ScreenSpaceUiSdfAtlas {
    /// 把异步字形生成结果投影回当前图集槽；只接受键仍匹配的失败，避免旧 bake 污染新计划。
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
            run.generation_failure_count = replace_run_generation_failures(
                &mut run.glyph_generation_failures,
                &run.glyph_slot_indices,
                failures_by_slot,
            );
        }
        self.recorded_generation_failures = Some(Arc::clone(failures));
    }
}

fn replace_run_generation_failures(
    output: &mut Vec<Option<SdfGlyphGenerationError>>,
    glyph_slot_indices: &[Option<usize>],
    failures_by_slot: &[Option<SdfGlyphGenerationError>],
) -> usize {
    output.clear();
    let mut failure_count = 0;
    output.extend(glyph_slot_indices.iter().map(|slot_index| {
        let failure = slot_index.and_then(|index| failures_by_slot.get(index).copied().flatten());
        if failure.is_some() {
            failure_count += 1;
        }
        failure
    }));
    failure_count
}

#[cfg(test)]
#[path = "tests/generation_failures_optimization_tests.rs"]
mod optimization_tests;
