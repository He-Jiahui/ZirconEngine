use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::text::font::FontDatabase;
use crate::text::sdf::{
    SdfBakeParams, SdfGenerationCompletionDrainBudget, SdfGenerationInactiveWorkOutcome,
    SdfGenerationScheduler, SdfGenerationSourceContext, SdfGenerationSourceHandle,
    SdfGenerationWorkId, SdfGlyphGenerationError,
};
use crate::text::FontFaceId;

use super::distance_field::{glyph_id_for_key, raw_baked_glyph};
use super::dynamic_batch::missing_outline;
use super::{fallback_metrics, RawBakedGlyph, SdfAtlasGlyphKey, SdfAtlasSlot, SdfFontBakeCache};

const COMPLETION_DRAIN_BATCH_BUDGET: usize = 64;
const COMPLETION_DRAIN_BYTE_BUDGET: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug)]
struct AsyncBatchEntry {
    key: SdfAtlasGlyphKey,
    face: FontFaceId,
    glyph_id: u16,
}

struct AsyncBatchGroup {
    source: Arc<SdfGenerationSourceContext>,
    params: SdfBakeParams,
    entries: Vec<AsyncBatchEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct AsyncBatchKey {
    source: SdfGenerationSourceHandle,
    params: SdfBakeParams,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FaceFailureKey {
    key: SdfAtlasGlyphKey,
    face: FontFaceId,
}

#[derive(Default)]
pub(super) struct SdfAsyncGenerationState {
    next_work_index: u64,
    last_drained_frame: Option<u64>,
    pending_batches: HashMap<SdfGenerationWorkId, Vec<AsyncBatchEntry>>,
    pending_keys: HashSet<SdfAtlasGlyphKey>,
    face_failures: HashMap<FaceFailureKey, SdfGlyphGenerationError>,
    transient_failures: HashMap<SdfAtlasGlyphKey, SdfGlyphGenerationError>,
}

impl SdfAsyncGenerationState {
    fn next_work_id(&mut self, generation: u64) -> SdfGenerationWorkId {
        let id = SdfGenerationWorkId::new(generation, self.next_work_index);
        self.next_work_index = self.next_work_index.wrapping_add(1);
        id
    }

    pub(super) fn transient_failure(
        &self,
        key: &SdfAtlasGlyphKey,
    ) -> Option<SdfGlyphGenerationError> {
        self.transient_failures.get(key).copied()
    }

    pub(super) fn has_pending_work(&self) -> bool {
        !self.pending_batches.is_empty()
    }

    fn finish_pending_batch_with_failure(
        &mut self,
        work_id: SdfGenerationWorkId,
        error: SdfGlyphGenerationError,
    ) {
        let Some(entries) = self.pending_batches.remove(&work_id) else {
            return;
        };
        for entry in entries {
            self.pending_keys.remove(&entry.key);
            self.face_failures.insert(
                FaceFailureKey {
                    key: entry.key,
                    face: entry.face,
                },
                error,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> SdfAtlasGlyphKey {
        SdfAtlasGlyphKey {
            glyph: 'A',
            glyph_id: Some(1),
            font_id: None,
            font_instance_id: None,
            font: Some("res://fonts/default.font.toml".into()),
            font_family: Some("Zircon Sans".into()),
            language: None,
            font_weight: 400,
            bake_params: SdfBakeParams::default(),
        }
    }

    #[test]
    fn worker_panic_releases_pending_key_and_marks_the_face_terminal() {
        let key = test_key();
        let face = FontFaceId(9);
        let work_id = SdfGenerationWorkId::new(3, 5);
        let mut state = SdfAsyncGenerationState::default();
        state.pending_keys.insert(key.clone());
        state.pending_batches.insert(
            work_id,
            vec![AsyncBatchEntry {
                key: key.clone(),
                face,
                glyph_id: 1,
            }],
        );

        state.finish_pending_batch_with_failure(work_id, SdfGlyphGenerationError::WorkerPanic);

        assert!(!state.pending_keys.contains(&key));
        assert!(state.pending_batches.is_empty());
        assert_eq!(
            state.face_failures.get(&FaceFailureKey { key, face }),
            Some(&SdfGlyphGenerationError::WorkerPanic)
        );
    }
}

impl SdfFontBakeCache {
    pub(super) fn prepare_missing_glyphs_async(
        &mut self,
        slots: &[SdfAtlasSlot],
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
        scheduler: &SdfGenerationScheduler,
        frame_index: u64,
    ) {
        self.begin_async_generation_frame(scheduler, frame_index);
        let mut seen = HashSet::with_capacity(slots.len());
        let mut groups = Vec::<AsyncBatchGroup>::new();
        let mut group_indices = HashMap::<AsyncBatchKey, usize>::new();

        for slot in slots {
            if self.contains_baked_glyph(&slot.key) || !seen.insert(slot.key.clone()) {
                continue;
            }
            if self.async_generation.pending_keys.contains(&slot.key) {
                self.async_generation
                    .transient_failures
                    .insert(slot.key.clone(), SdfGlyphGenerationError::GenerationPending);
                continue;
            }

            let faces = self.resolve_faces_for_key_cached(&slot.key, font_database, asset_manager);
            let resolved_shaped_face = self
                .shaped_face_resolutions
                .get(&slot.key)
                .copied()
                .flatten();
            let mut last_error = None;
            let mut resolved = false;
            for face in faces {
                let failure_key = FaceFailureKey {
                    key: slot.key.clone(),
                    face,
                };
                if let Some(error) = self
                    .async_generation
                    .face_failures
                    .get(&failure_key)
                    .copied()
                {
                    last_error = Some(error);
                    continue;
                }
                let _ = self.ensure_sdf_font(face, font_database);
                let source = match self.source_contexts.resolve(&slot.key, face, font_database) {
                    Ok(source) => source,
                    Err(error) => {
                        self.async_generation
                            .face_failures
                            .insert(failure_key, error);
                        last_error = Some(error);
                        continue;
                    }
                };
                if let Some(offline) = self.offline_source.load_glyph(
                    &slot.key,
                    face,
                    resolved_shaped_face,
                    &source,
                    font_database,
                    asset_manager,
                ) {
                    self.insert_baked_glyph(slot.key.clone(), offline);
                    resolved = true;
                    break;
                }
                let glyph_id =
                    match glyph_id_for_key(&slot.key, face, resolved_shaped_face, font_database) {
                        Ok(glyph_id) => glyph_id,
                        Err(error) => {
                            self.async_generation
                                .face_failures
                                .insert(failure_key, error);
                            last_error = Some(error);
                            continue;
                        }
                    };
                let batch_key = AsyncBatchKey {
                    source: source.handle(),
                    params: slot.key.bake_params,
                };
                let group_index = *group_indices.entry(batch_key).or_insert_with(|| {
                    groups.push(AsyncBatchGroup {
                        source: Arc::clone(&source),
                        params: slot.key.bake_params,
                        entries: Vec::new(),
                    });
                    groups.len() - 1
                });
                groups[group_index].entries.push(AsyncBatchEntry {
                    key: slot.key.clone(),
                    face,
                    glyph_id,
                });
                resolved = true;
                break;
            }

            if !resolved {
                let error = last_error.unwrap_or_else(|| missing_outline(&slot.key));
                let failed = RawBakedGlyph::failed(
                    fallback_metrics(slot.key.bake_params.bake_em_px_f32()),
                    error,
                );
                self.insert_baked_glyph(slot.key.clone(), failed);
            }
        }

        for group in groups {
            for entries in group.entries.chunks(scheduler.max_glyphs_per_batch()) {
                let entries = entries.to_vec();
                let glyph_ids = entries
                    .iter()
                    .map(|entry| entry.glyph_id)
                    .collect::<Vec<_>>();
                let work_id = self
                    .async_generation
                    .next_work_id(self.observed_font_generation);
                match scheduler.try_submit(
                    work_id,
                    frame_index,
                    Arc::clone(&group.source),
                    group.params,
                    glyph_ids,
                ) {
                    Ok(()) => {
                        for entry in &entries {
                            self.async_generation.pending_keys.insert(entry.key.clone());
                            self.async_generation.transient_failures.insert(
                                entry.key.clone(),
                                SdfGlyphGenerationError::GenerationPending,
                            );
                        }
                        self.async_generation
                            .pending_batches
                            .insert(work_id, entries);
                    }
                    Err(_) => {
                        for entry in entries {
                            self.async_generation.transient_failures.insert(
                                entry.key,
                                SdfGlyphGenerationError::GenerationBudgetDeferred,
                            );
                        }
                    }
                }
            }
        }
    }

    pub(super) fn cancel_async_generation(&mut self, scheduler: &SdfGenerationScheduler) {
        for work_id in self.async_generation.pending_batches.keys().copied() {
            let _ = scheduler.cancel(work_id);
        }
        self.async_generation = SdfAsyncGenerationState::default();
    }

    fn begin_async_generation_frame(
        &mut self,
        scheduler: &SdfGenerationScheduler,
        frame_index: u64,
    ) {
        if self.async_generation.last_drained_frame == Some(frame_index) {
            return;
        }
        self.async_generation.last_drained_frame = Some(frame_index);
        self.async_generation.transient_failures.clear();
        let completions = scheduler.drain_completed(
            frame_index,
            SdfGenerationCompletionDrainBudget::new(
                COMPLETION_DRAIN_BATCH_BUDGET,
                COMPLETION_DRAIN_BYTE_BUDGET,
            ),
        );
        for completion in completions {
            let Some(entries) = self.async_generation.pending_batches.remove(&completion.id) else {
                continue;
            };
            self.dynamic_generation_totals
                .record(completion.batch.report);
            let results = completion
                .batch
                .glyphs
                .into_iter()
                .map(|glyph| (glyph.glyph_id, glyph.result.map(raw_baked_glyph)))
                .collect::<HashMap<_, _>>();
            for entry in entries {
                self.async_generation.pending_keys.remove(&entry.key);
                match results.get(&entry.glyph_id) {
                    Some(Ok(glyph)) => {
                        self.insert_baked_glyph(entry.key, glyph.clone());
                    }
                    Some(Err(error)) => {
                        self.async_generation.face_failures.insert(
                            FaceFailureKey {
                                key: entry.key,
                                face: entry.face,
                            },
                            *error,
                        );
                    }
                    None => {
                        self.async_generation.face_failures.insert(
                            FaceFailureKey {
                                key: entry.key,
                                face: entry.face,
                            },
                            SdfGlyphGenerationError::MissingGlyphOutline(entry.glyph_id),
                        );
                    }
                }
            }
        }
        let pending_ids = self
            .async_generation
            .pending_batches
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for (inactive_id, outcome) in scheduler.take_inactive_work_outcomes(pending_ids) {
            match outcome {
                SdfGenerationInactiveWorkOutcome::WorkerPanic => {
                    self.async_generation.finish_pending_batch_with_failure(
                        inactive_id,
                        SdfGlyphGenerationError::WorkerPanic,
                    )
                }
                SdfGenerationInactiveWorkOutcome::Retryable => {
                    let Some(entries) = self.async_generation.pending_batches.remove(&inactive_id)
                    else {
                        continue;
                    };
                    for entry in entries {
                        self.async_generation.pending_keys.remove(&entry.key);
                        self.async_generation
                            .transient_failures
                            .insert(entry.key, SdfGlyphGenerationError::GenerationBudgetDeferred);
                    }
                }
            }
        }
    }
}
