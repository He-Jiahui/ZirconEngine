use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::text::{
    FontFaceId, FontFamilyName, FontMatch, FontQuery, FontStretch, FontStyle, FontWeight,
};

use super::super::fallback_cache::family_candidate_cache_key;
use super::super::matching::{
    dedupe_families, font_family_identity, stretch_distance, style_distance, weight_distance,
    FontFamilyIdentity,
};
use super::FontDatabase;

const MAX_FACE_MATCH_CACHE_ENTRIES: usize = 64;

#[derive(Clone, Debug, Default)]
pub(super) struct FaceMatchCache {
    entries: HashMap<FontMatchCacheKey, Option<FontMatch>>,
    insertion_order: VecDeque<FontMatchCacheKey>,
}

impl FaceMatchCache {
    fn get(&self, key: &FontMatchCacheKey) -> Option<Option<FontMatch>> {
        self.entries.get(key).copied()
    }

    fn insert(&mut self, key: FontMatchCacheKey, value: Option<FontMatch>) {
        if self.entries.contains_key(&key) {
            self.entries.insert(key, value);
            return;
        }
        while self.entries.len() >= MAX_FACE_MATCH_CACHE_ENTRIES {
            let Some(oldest) = self.insertion_order.pop_front() else {
                return;
            };
            self.entries.remove(&oldest);
        }
        self.insertion_order.push_back(key.clone());
        self.entries.insert(key, value);
    }

    #[cfg(test)]
    fn contains(&self, key: &FontMatchCacheKey) -> bool {
        self.entries.contains_key(key)
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_font_face_match_cache_evicts_only_its_oldest_entry_at_capacity() {
        let mut cache = FaceMatchCache::default();
        let keys = (0..=MAX_FACE_MATCH_CACHE_ENTRIES)
            .map(|index| {
                FontMatchCacheKey::from(&FontQuery::single_family(format!("Family {index}")))
            })
            .collect::<Vec<_>>();

        for key in keys.iter().take(MAX_FACE_MATCH_CACHE_ENTRIES) {
            cache.insert(key.clone(), None);
        }
        cache.insert(keys[MAX_FACE_MATCH_CACHE_ENTRIES].clone(), None);

        assert_eq!(cache.len(), MAX_FACE_MATCH_CACHE_ENTRIES);
        assert!(!cache.contains(&keys[0]));
        assert!(keys[1..].iter().all(|key| cache.contains(key)));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct FontMatchCacheKey {
    families: Vec<FontFamilyIdentity>,
    weight: FontWeight,
    style: FontMatchStyleKey,
    stretch: FontStretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FontMatchStyleKey {
    Normal,
    Italic,
    Oblique(u32),
}

impl From<&FontQuery> for FontMatchCacheKey {
    fn from(query: &FontQuery) -> Self {
        Self {
            families: query
                .families
                .iter()
                .map(|family| font_family_identity(family.as_str()))
                .collect(),
            weight: query.weight,
            style: match query.style {
                FontStyle::Normal => FontMatchStyleKey::Normal,
                FontStyle::Italic => FontMatchStyleKey::Italic,
                FontStyle::Oblique(angle) => FontMatchStyleKey::Oblique(angle.to_bits()),
            },
            stretch: query.stretch,
        }
    }
}

impl FontDatabase {
    pub(crate) fn match_face(&self, query: &FontQuery) -> Option<FontMatch> {
        let key = FontMatchCacheKey::from(query);
        {
            let cache = self
                .face_match_cache
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(cached) = cache.get(&key) {
                return cached;
            }
        }
        let mut families = query.families.clone();
        families.extend(self.fallback_families.iter().cloned());
        let matched = self.match_face_in_family_order(&families, query);
        let mut cache = self
            .face_match_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        cache.insert(key, matched);
        matched
    }

    fn match_face_in_family_order(
        &self,
        families: &[FontFamilyName],
        query: &FontQuery,
    ) -> Option<FontMatch> {
        dedupe_families(families.iter().cloned())
            .into_iter()
            .filter_map(|family| self.family_candidates(&family, query).first().copied())
            .next()
            .map(|face| FontMatch {
                face,
                synthetic_bold: false,
                synthetic_oblique: false,
            })
    }

    fn family_candidates(&self, family: &FontFamilyName, query: &FontQuery) -> Arc<[FontFaceId]> {
        let family_identity = font_family_identity(family.as_str());
        let cache_key = family_candidate_cache_key(family_identity, query);
        if let Some(candidates) = self.fallback_caches.family_candidates(cache_key) {
            return candidates;
        }
        let mut candidates = self
            .family_index
            .get(&family_identity)
            .cloned()
            .unwrap_or_default();
        if let Some(aliases) = self.family_alias_index.get(&family_identity) {
            let aliases = aliases
                .iter()
                .copied()
                .filter(|candidate| !candidates.contains(candidate))
                .collect::<Vec<_>>();
            candidates.extend(aliases);
        }
        candidates.sort_by_key(|id| self.match_score(*id, query));
        let candidates = Arc::from(candidates.into_boxed_slice());
        self.fallback_caches
            .insert_family_candidates(cache_key, Arc::clone(&candidates));
        candidates
    }

    pub(in crate::text::font) fn family_candidates_for_codepoint(
        &self,
        family: &FontFamilyName,
        query: &FontQuery,
        codepoint: char,
    ) -> Vec<FontFaceId> {
        let candidates = self.family_candidates(family, query);
        self.fallback_caches.record_face_visits(candidates.len());
        candidates
            .iter()
            .copied()
            .filter(|face| self.face_covers_codepoint(*face, codepoint))
            .collect()
    }

    fn match_score(&self, face: FontFaceId, query: &FontQuery) -> (u16, u16, u8) {
        let Some(stored) = self.face(face) else {
            return (u16::MAX, u16::MAX, u8::MAX);
        };
        (
            weight_distance(stored.descriptor.weight, query.weight),
            stretch_distance(stored.descriptor.stretch, query.stretch),
            style_distance(stored.descriptor.style, query.style),
        )
    }
}
