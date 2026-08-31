use std::sync::Arc;

use crate::text::sdf::{SdfAtlasGlyphGenerationFailure, SdfGlyphGenerationError};

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
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::text::sdf::SdfGlyphGenerationError;

    use super::replace_run_generation_failures;

    #[test]
    fn optimization_batch_dh_reused_failure_projection_matches_legacy_output() {
        let slot_indices = [Some(0), None, Some(2), Some(8), Some(1)];
        let failures_by_slot = [
            Some(SdfGlyphGenerationError::GenerationPending),
            None,
            Some(SdfGlyphGenerationError::MissingGlyphOutline(7)),
        ];
        let mut optimized = Vec::with_capacity(slot_indices.len());

        let optimized_count =
            replace_run_generation_failures(&mut optimized, &slot_indices, &failures_by_slot);
        let (legacy, legacy_count) = legacy_failure_projection(&slot_indices, &failures_by_slot);

        assert_eq!(optimized, legacy);
        assert_eq!(optimized_count, legacy_count);
    }

    #[test]
    fn optimization_batch_dh_failure_projection_reuses_capacity_in_one_pass() {
        let source = include_str!("generation_failures.rs");

        assert!(source.contains("output.clear();"));
        assert!(source.contains("output.extend(glyph_slot_indices.iter().map"));
        assert!(source.contains("if failure.is_some()"));
        assert!(!source.contains("run.glyph_generation_failures = run"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dh_sdf_failure_projection_reuse_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const PROJECTIONS_PER_SAMPLE: usize = 64;
        const GLYPHS_PER_RUN: usize = 4_096;
        const SLOT_COUNT: usize = 1_024;

        let slot_indices = (0..GLYPHS_PER_RUN)
            .map(|index| (index % 11 != 0).then_some(index % SLOT_COUNT))
            .collect::<Vec<_>>();
        let failures_by_slot = (0..SLOT_COUNT)
            .map(|index| {
                (index % 7 == 0)
                    .then_some(SdfGlyphGenerationError::MissingGlyphOutline(index as u16))
            })
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy_projection(
                    &slot_indices,
                    &failures_by_slot,
                    PROJECTIONS_PER_SAMPLE,
                ));
                optimized_samples.push(measure_reused_projection(
                    &slot_indices,
                    &failures_by_slot,
                    PROJECTIONS_PER_SAMPLE,
                ));
            } else {
                optimized_samples.push(measure_reused_projection(
                    &slot_indices,
                    &failures_by_slot,
                    PROJECTIONS_PER_SAMPLE,
                ));
                legacy_samples.push(measure_legacy_projection(
                    &slot_indices,
                    &failures_by_slot,
                    PROJECTIONS_PER_SAMPLE,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME416_SDF_FAILURE_PROJECTION_REUSE_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "reused SDF failure projection p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn legacy_failure_projection(
        slot_indices: &[Option<usize>],
        failures_by_slot: &[Option<SdfGlyphGenerationError>],
    ) -> (Vec<Option<SdfGlyphGenerationError>>, usize) {
        let output = slot_indices
            .iter()
            .map(|slot_index| {
                slot_index.and_then(|index| failures_by_slot.get(index).copied().flatten())
            })
            .collect::<Vec<_>>();
        let count = output.iter().filter(|failure| failure.is_some()).count();
        (output, count)
    }

    fn measure_legacy_projection(
        slot_indices: &[Option<usize>],
        failures_by_slot: &[Option<SdfGlyphGenerationError>],
        projections: usize,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0;
        for _ in 0..projections {
            let (output, count) =
                legacy_failure_projection(black_box(slot_indices), black_box(failures_by_slot));
            checksum += output.len() + count;
            black_box(output);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn measure_reused_projection(
        slot_indices: &[Option<usize>],
        failures_by_slot: &[Option<SdfGlyphGenerationError>],
        projections: usize,
    ) -> u128 {
        let mut output = Vec::with_capacity(slot_indices.len());
        let started_at = Instant::now();
        let mut checksum = 0;
        for _ in 0..projections {
            let count = replace_run_generation_failures(
                black_box(&mut output),
                black_box(slot_indices),
                black_box(failures_by_slot),
            );
            checksum += output.len() + count;
        }
        black_box((checksum, output));
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = samples
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        samples[index]
    }
}
