use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::text::FontFaceId;
use crate::text::font::FontDatabase;
use crate::text::sdf::{
    SdfBakeParams, SdfGenerationBatchReport, SdfGenerationSourceContext, SdfGenerationSourceHandle,
    SdfGlyphGenerationError,
};

use super::distance_field::{glyph_id_for_key, raw_baked_glyph};
use super::{RawBakedGlyph, SdfAtlasGlyphKey, SdfAtlasSlot, SdfFontBakeCache, fallback_metrics};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfDynamicGenerationTotals {
    pub(super) batch_count: usize,
    pub(super) requested_glyph_count: usize,
    pub(super) unique_glyph_count: usize,
    pub(super) duplicate_glyph_count: usize,
}

impl SdfDynamicGenerationTotals {
    pub(super) fn delta_since(self, previous: Self) -> Self {
        Self {
            batch_count: self.batch_count.saturating_sub(previous.batch_count),
            requested_glyph_count: self
                .requested_glyph_count
                .saturating_sub(previous.requested_glyph_count),
            unique_glyph_count: self
                .unique_glyph_count
                .saturating_sub(previous.unique_glyph_count),
            duplicate_glyph_count: self
                .duplicate_glyph_count
                .saturating_sub(previous.duplicate_glyph_count),
        }
    }

    pub(super) fn record(&mut self, report: SdfGenerationBatchReport) {
        self.batch_count = self.batch_count.saturating_add(1);
        self.requested_glyph_count = self
            .requested_glyph_count
            .saturating_add(report.requested_glyph_count);
        self.unique_glyph_count = self
            .unique_glyph_count
            .saturating_add(report.unique_glyph_count);
        self.duplicate_glyph_count = self
            .duplicate_glyph_count
            .saturating_add(report.duplicate_glyph_count);
    }
}

struct PendingGlyph {
    key: SdfAtlasGlyphKey,
    faces: Vec<FontFaceId>,
    next_face: usize,
    last_error: Option<SdfGlyphGenerationError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BatchKey {
    source: SdfGenerationSourceHandle,
    params: SdfBakeParams,
}

struct BatchGroup {
    source: Arc<SdfGenerationSourceContext>,
    params: SdfBakeParams,
    entries: Vec<(usize, u16)>,
}

impl SdfFontBakeCache {
    pub(super) fn prepare_missing_glyphs(
        &mut self,
        slots: &[SdfAtlasSlot],
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) {
        let mut seen = HashSet::with_capacity(slots.len());
        let mut pending = Vec::with_capacity(slots.len());
        for slot in slots {
            if self.contains_baked_glyph(&slot.key) || !seen.insert(slot.key.clone()) {
                continue;
            }
            pending.push(PendingGlyph {
                faces: self.resolve_faces_for_key_cached(&slot.key, font_database),
                key: slot.key.clone(),
                next_face: 0,
                last_error: None,
            });
        }

        while !pending.is_empty() {
            let mut groups = Vec::<BatchGroup>::with_capacity(pending.len());
            let mut group_indices = HashMap::<BatchKey, usize>::with_capacity(pending.len());
            let mut resolved = vec![None; pending.len()];

            for (pending_index, glyph) in pending.iter_mut().enumerate() {
                let shaped_resolution = self
                    .shaped_face_resolutions
                    .get(&glyph.key)
                    .copied()
                    .flatten();
                let resolved_shaped_face = shaped_resolution.map(|resolution| resolution.face);
                let Some(face) = glyph.faces.get(glyph.next_face).copied() else {
                    let error = glyph
                        .last_error
                        .unwrap_or_else(|| missing_outline(&glyph.key));
                    resolved[pending_index] = Some(RawBakedGlyph::failed(
                        fallback_metrics(glyph.key.bake_params.bake_em_px_f32()),
                        error,
                    ));
                    continue;
                };
                glyph.next_face = glyph.next_face.saturating_add(1);
                let _ = self.ensure_sdf_font(face, font_database);
                let resolved_instance = shaped_resolution
                    .filter(|resolution| resolution.face == face)
                    .and_then(|resolution| resolution.instance);
                let source = match self.source_contexts.resolve(
                    &glyph.key,
                    face,
                    resolved_instance,
                    font_database,
                ) {
                    Ok(source) => source,
                    Err(error) => {
                        glyph.last_error = Some(error);
                        continue;
                    }
                };
                if let Some(offline) = self.offline_source.load_glyph(
                    &glyph.key,
                    face,
                    resolved_shaped_face,
                    &source,
                    font_database,
                    asset_manager,
                ) {
                    resolved[pending_index] = Some(offline);
                    continue;
                }
                let glyph_id =
                    match glyph_id_for_key(&glyph.key, face, resolved_shaped_face, font_database) {
                        Ok(glyph_id) => glyph_id,
                        Err(error) => {
                            glyph.last_error = Some(error);
                            continue;
                        }
                    };
                let batch_key = BatchKey {
                    source: source.handle(),
                    params: glyph.key.bake_params,
                };
                let group_index = *group_indices.entry(batch_key).or_insert_with(|| {
                    groups.push(BatchGroup {
                        source: Arc::clone(&source),
                        params: glyph.key.bake_params,
                        entries: Vec::new(),
                    });
                    groups.len() - 1
                });
                groups[group_index].entries.push((pending_index, glyph_id));
            }

            for group in groups {
                let mut glyph_ids = Vec::with_capacity(group.entries.len());
                for (_, glyph_id) in &group.entries {
                    glyph_ids.push(*glyph_id);
                }
                let batch = group.source.generate_batch(group.params, &glyph_ids);
                self.dynamic_generation_totals.record(batch.report);
                let mut results = HashMap::with_capacity(batch.glyphs.len());
                for glyph in batch.glyphs {
                    let result = glyph.result.map(raw_baked_glyph);
                    results.insert(glyph.glyph_id, result);
                }
                for (pending_index, glyph_id) in group.entries {
                    match results.get(&glyph_id) {
                        Some(Ok(glyph)) => resolved[pending_index] = Some(glyph.clone()),
                        Some(Err(error)) => pending[pending_index].last_error = Some(*error),
                        None => {
                            pending[pending_index].last_error =
                                Some(SdfGlyphGenerationError::MissingGlyphOutline(glyph_id))
                        }
                    }
                }
            }

            let mut unresolved = Vec::with_capacity(pending.len());
            for (pending_glyph, baked) in pending.into_iter().zip(resolved) {
                if let Some(baked) = baked {
                    self.insert_baked_glyph(pending_glyph.key, baked);
                } else {
                    unresolved.push(pending_glyph);
                }
            }
            pending = unresolved;
        }
    }
}

pub(super) fn missing_outline(key: &SdfAtlasGlyphKey) -> SdfGlyphGenerationError {
    SdfGlyphGenerationError::MissingGlyphOutline(
        key.glyph_id
            .and_then(|glyph_id| u16::try_from(glyph_id).ok())
            .unwrap_or(0),
    )
}

#[cfg(test)]
mod optimization_batch_20260830by_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const SLOTS_PER_SAMPLE: usize = 1_024;

    #[test]
    fn dynamic_batch_reserves_input_and_retry_collections() {
        let source = include_str!("dynamic_batch.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(slots.len())"));
        assert!(implementation.contains("Vec::<BatchGroup>::with_capacity(pending.len())"));
        assert!(
            implementation.contains("HashMap::<BatchKey, usize>::with_capacity(pending.len())")
        );
        assert!(implementation.contains("Vec::with_capacity(group.entries.len())"));
        assert!(implementation.contains("HashMap::with_capacity(batch.glyphs.len())"));
        assert!(implementation.contains("Vec::with_capacity(pending.len())"));
        assert!(!implementation.contains("let mut pending = Vec::new()"));
    }

    #[test]
    fn dynamic_batch_keeps_group_generation_before_retry_scan() {
        let source = include_str!("dynamic_batch.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let generation = implementation
            .find("generate_batch(group.params, &glyph_ids)")
            .expect("batch generation");
        let retry = implementation
            .find("let mut unresolved = Vec::with_capacity(pending.len())")
            .expect("retry collection");
        assert!(generation < retry);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830by_runtime_dynamic_batch_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME377_DYNAMIC_BATCH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} slots_per_sample={SLOTS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut pending = if optimized {
                Vec::with_capacity(SLOTS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            let mut groups = if optimized {
                Vec::with_capacity(SLOTS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            let mut results = if optimized {
                std::collections::HashMap::with_capacity(SLOTS_PER_SAMPLE)
            } else {
                std::collections::HashMap::new()
            };
            for index in 0..SLOTS_PER_SAMPLE {
                pending.push(index);
                groups.push(index);
                results.insert(index, index);
            }
            let mut unresolved = if optimized {
                Vec::with_capacity(pending.len())
            } else {
                Vec::new()
            };
            unresolved.extend(pending.into_iter().filter(|index| index % 2 == 0));
            checksum ^= groups.len() ^ results.len() ^ unresolved.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
