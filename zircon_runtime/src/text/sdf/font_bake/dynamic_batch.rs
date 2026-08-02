use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::text::font::FontDatabase;
use crate::text::sdf::{
    SdfBakeParams, SdfGenerationBatchReport, SdfGenerationSourceContext, SdfGenerationSourceHandle,
    SdfGlyphGenerationError,
};
use crate::text::FontFaceId;

use super::distance_field::{glyph_id_for_key, raw_baked_glyph};
use super::{fallback_metrics, RawBakedGlyph, SdfAtlasGlyphKey, SdfAtlasSlot, SdfFontBakeCache};

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
        let mut pending = Vec::new();
        for slot in slots {
            if self.contains_baked_glyph(&slot.key) || !seen.insert(slot.key.clone()) {
                continue;
            }
            pending.push(PendingGlyph {
                faces: self.resolve_faces_for_key_cached(&slot.key, font_database, asset_manager),
                key: slot.key.clone(),
                next_face: 0,
                last_error: None,
            });
        }

        while !pending.is_empty() {
            let mut groups = Vec::<BatchGroup>::new();
            let mut group_indices = HashMap::<BatchKey, usize>::new();
            let mut resolved = vec![None; pending.len()];

            for (pending_index, glyph) in pending.iter_mut().enumerate() {
                let resolved_shaped_face = self
                    .shaped_face_resolutions
                    .get(&glyph.key)
                    .copied()
                    .flatten();
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
                let source = match self
                    .source_contexts
                    .resolve(&glyph.key, face, font_database)
                {
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
                let glyph_ids = group
                    .entries
                    .iter()
                    .map(|(_, glyph_id)| *glyph_id)
                    .collect::<Vec<_>>();
                let batch = group.source.generate_batch(group.params, &glyph_ids);
                self.dynamic_generation_totals.record(batch.report);
                let results = batch
                    .glyphs
                    .into_iter()
                    .map(|glyph| {
                        let result = glyph.result.map(raw_baked_glyph);
                        (glyph.glyph_id, result)
                    })
                    .collect::<HashMap<_, _>>();
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

            let mut unresolved = Vec::new();
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
