use std::collections::HashMap;
use std::sync::Arc;

use crate::text::font::FontDatabase;
use crate::text::sdf::{
    sdf_variation_hash, SdfGenerationSourceContext, SdfGenerationSourceHandle,
    SdfGlyphGenerationError,
};
use crate::text::{FontFaceId, InstancedFaceId};

use super::SdfAtlasGlyphKey;

const MAX_RESIDENT_SOURCE_CONTEXT_COUNT: usize = 64;
const MAX_RESIDENT_SOURCE_BYTE_COUNT: usize = 128 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SdfGenerationSourceKey {
    face: FontFaceId,
    variation_hash: [u8; 32],
}

#[derive(Clone)]
struct SdfGenerationFontSource {
    bytes: Arc<[u8]>,
    source_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfGenerationSourceCacheReport {
    pub(super) resident_context_count: usize,
    pub(super) resident_source_byte_count: usize,
    pub(super) context_created_count: usize,
    pub(super) context_eviction_count: usize,
    pub(super) oldest_context_idle_access_count: u64,
    pub(super) source_hash_count: usize,
    pub(super) face_parse_count: usize,
}

#[derive(Default)]
pub(super) struct SdfGenerationSourceCache {
    generation: u64,
    next_handle: u64,
    sources: HashMap<FontFaceId, SdfGenerationFontSource>,
    contexts: HashMap<SdfGenerationSourceKey, Arc<SdfGenerationSourceContext>>,
    context_recency: HashMap<SdfGenerationSourceKey, u64>,
    access_epoch: u64,
    resident_source_byte_count: usize,
    context_created_count: usize,
    context_eviction_count: usize,
    source_hash_count: usize,
    face_parse_count: usize,
}

impl SdfGenerationSourceCacheReport {
    pub(super) fn delta_since(self, previous: Self) -> Self {
        Self {
            resident_context_count: self.resident_context_count,
            resident_source_byte_count: self.resident_source_byte_count,
            context_created_count: self
                .context_created_count
                .saturating_sub(previous.context_created_count),
            context_eviction_count: self
                .context_eviction_count
                .saturating_sub(previous.context_eviction_count),
            oldest_context_idle_access_count: self.oldest_context_idle_access_count,
            source_hash_count: self
                .source_hash_count
                .saturating_sub(previous.source_hash_count),
            face_parse_count: self
                .face_parse_count
                .saturating_sub(previous.face_parse_count),
        }
    }
}

impl SdfGenerationSourceCache {
    pub(super) fn new(generation: u64) -> Self {
        Self {
            generation,
            ..Self::default()
        }
    }

    pub(super) fn resolve(
        &mut self,
        key: &SdfAtlasGlyphKey,
        face: FontFaceId,
        instance: Option<InstancedFaceId>,
        font_database: &FontDatabase,
    ) -> Result<Arc<SdfGenerationSourceContext>, SdfGlyphGenerationError> {
        let variations = font_database
            .effective_instance_variations_shared(face, instance, key.font_weight)
            .map_err(|_| SdfGlyphGenerationError::InvalidFaceIndex(0))?;
        let source_key = SdfGenerationSourceKey {
            face,
            variation_hash: sdf_variation_hash(variations.as_ref()),
        };
        if let Some(context) = self.contexts.get(&source_key).cloned() {
            self.touch_context(source_key);
            return Ok(context);
        }
        let (source, inserted_source) = if let Some(source) = self.sources.get(&face) {
            (source.clone(), false)
        } else {
            let bytes = font_database
                .standalone_face_bytes(face)
                .map_err(|_| SdfGlyphGenerationError::InvalidFaceIndex(0))?;
            let source = SdfGenerationFontSource {
                source_hash: crate::text::sdf::sdf_font_source_hash(bytes.as_ref()),
                bytes,
            };
            self.source_hash_count = self.source_hash_count.saturating_add(1);
            self.sources.insert(face, source.clone());
            self.resident_source_byte_count = self
                .resident_source_byte_count
                .saturating_add(source.bytes.len());
            (source, true)
        };
        let handle = SdfGenerationSourceHandle::for_generation(self.generation, self.next_handle);
        self.next_handle = self.next_handle.wrapping_add(1);
        let context = match SdfGenerationSourceContext::from_hashed_source(
            handle,
            source.bytes,
            source.source_hash,
            0,
            0,
            variations,
        ) {
            Ok(context) => Arc::new(context),
            Err(error) => {
                if inserted_source {
                    self.remove_source(face);
                }
                return Err(error);
            }
        };
        let context_report = context.report();
        self.context_created_count = self.context_created_count.saturating_add(1);
        self.face_parse_count = self
            .face_parse_count
            .saturating_add(context_report.face_parse_count);
        self.contexts.insert(source_key, Arc::clone(&context));
        self.touch_context(source_key);
        self.enforce_budget();
        Ok(context)
    }

    pub(super) fn report(&self) -> SdfGenerationSourceCacheReport {
        SdfGenerationSourceCacheReport {
            resident_context_count: self.contexts.len(),
            resident_source_byte_count: self.resident_source_byte_count,
            context_created_count: self.context_created_count,
            context_eviction_count: self.context_eviction_count,
            oldest_context_idle_access_count: self
                .context_recency
                .values()
                .copied()
                .min()
                .map(|epoch| self.access_epoch.saturating_sub(epoch))
                .unwrap_or(0),
            source_hash_count: self.source_hash_count,
            face_parse_count: self.face_parse_count,
        }
    }

    fn touch_context(&mut self, key: SdfGenerationSourceKey) {
        self.access_epoch = self.access_epoch.saturating_add(1).max(1);
        self.context_recency.insert(key, self.access_epoch);
    }

    fn enforce_budget(&mut self) {
        while self.contexts.len() > MAX_RESIDENT_SOURCE_CONTEXT_COUNT
            || (self.contexts.len() > 1
                && self.resident_source_byte_count > MAX_RESIDENT_SOURCE_BYTE_COUNT)
        {
            let Some(victim) = self
                .context_recency
                .iter()
                .min_by_key(|(_, epoch)| *epoch)
                .map(|(key, _)| *key)
            else {
                break;
            };
            self.contexts.remove(&victim);
            self.context_recency.remove(&victim);
            if !self.contexts.keys().any(|key| key.face == victim.face) {
                self.remove_source(victim.face);
            }
            self.context_eviction_count = self.context_eviction_count.saturating_add(1);
        }
    }

    fn remove_source(&mut self, face: FontFaceId) {
        if let Some(source) = self.sources.remove(&face) {
            self.resident_source_byte_count = self
                .resident_source_byte_count
                .saturating_sub(source.bytes.len());
        }
    }
}
