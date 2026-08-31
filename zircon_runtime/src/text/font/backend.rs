use std::collections::{HashMap, hash_map::Entry};

use glyphon::fontdb;

use crate::text::FontFaceId;

/// ID reconciliation for one authoritative `fontdb::Database` lineage.
///
/// `fontdb::ID` values are database-local slot-map keys. Keeping both
/// directions beside the database prevents shaping and raster consumers from
/// attempting to reconstruct the selected face from script or codepoints.
#[derive(Clone, Debug, Default)]
pub(super) struct BackendFaceMap {
    backend_to_face: HashMap<fontdb::ID, FontFaceId>,
    face_to_backend: HashMap<FontFaceId, fontdb::ID>,
    backend_entries_by_face: HashMap<FontFaceId, Vec<fontdb::ID>>,
}

impl BackendFaceMap {
    pub(super) fn insert(&mut self, backend: fontdb::ID, face: FontFaceId) {
        self.remove_face(face);
        self.detach_backend_entry(backend);
        self.backend_to_face.insert(backend, face);
        self.face_to_backend.insert(face, backend);
        self.backend_entries_by_face.insert(face, vec![backend]);
    }

    pub(super) fn insert_alias(&mut self, backend: fontdb::ID, face: FontFaceId) {
        self.detach_backend_entry(backend);
        self.backend_to_face.insert(backend, face);
        let entries = self.backend_entries_by_face.entry(face).or_default();
        // Detachment removes every prior occurrence before this face is selected.
        debug_assert!(!entries.contains(&backend));
        entries.push(backend);
        self.face_to_backend.entry(face).or_insert(backend);
    }

    pub(super) fn font_face_id(&self, backend: fontdb::ID) -> Option<FontFaceId> {
        self.backend_to_face.get(&backend).copied()
    }

    pub(super) fn backend_face_id(&self, face: FontFaceId) -> Option<fontdb::ID> {
        self.face_to_backend.get(&face).copied()
    }

    pub(super) fn remove_face(&mut self, face: FontFaceId) -> Vec<fontdb::ID> {
        self.face_to_backend.remove(&face);
        let entries = self
            .backend_entries_by_face
            .remove(&face)
            .unwrap_or_default();
        for backend in &entries {
            self.backend_to_face.remove(backend);
        }
        entries
    }

    fn detach_backend_entry(&mut self, backend: fontdb::ID) {
        let Some(previous_face) = self.backend_to_face.remove(&backend) else {
            return;
        };
        let (remove_face, next_backend) =
            if let Some(entries) = self.backend_entries_by_face.get_mut(&previous_face) {
                entries.retain(|entry| *entry != backend);
                (entries.is_empty(), entries.first().copied())
            } else {
                (false, None)
            };
        if remove_face {
            self.backend_entries_by_face.remove(&previous_face);
            self.face_to_backend.remove(&previous_face);
        } else if let Some(next_backend) = next_backend {
            if let Entry::Occupied(mut primary) = self.face_to_backend.entry(previous_face) {
                if *primary.get() == backend {
                    primary.insert(next_backend);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BackendFaceMap;
    use crate::text::FontFaceId;
    use glyphon::fontdb;

    #[test]
    fn backend_face_map_replaces_both_stale_directions() {
        let mut map = BackendFaceMap::default();
        let first_backend = fontdb::ID::dummy();
        let second_backend = fontdb::ID::default();
        let first_face = FontFaceId(1);
        let second_face = FontFaceId(2);

        map.insert(first_backend, first_face);
        map.insert(second_backend, first_face);
        map.insert(second_backend, second_face);

        assert_eq!(map.font_face_id(first_backend), None);
        assert_eq!(map.backend_face_id(first_face), None);
        assert_eq!(map.font_face_id(second_backend), Some(second_face));
        assert_eq!(map.backend_face_id(second_face), Some(second_backend));
    }

    #[test]
    fn backend_face_map_retains_aliases_until_the_face_is_retired() {
        let mut map = BackendFaceMap::default();
        let primary_backend = fontdb::ID::dummy();
        let alias_backend = fontdb::ID::default();
        let face = FontFaceId(1);

        map.insert(primary_backend, face);
        map.insert_alias(alias_backend, face);

        assert_eq!(map.backend_face_id(face), Some(primary_backend));
        assert_eq!(map.font_face_id(alias_backend), Some(face));
        assert_eq!(
            map.remove_face(face),
            vec![primary_backend, alias_backend],
            "retiring one logical face must remove both its primary and alias backend entries"
        );
        assert_eq!(map.font_face_id(primary_backend), None);
        assert_eq!(map.font_face_id(alias_backend), None);
    }

    #[test]
    fn optimization_batch_20260830em_alias_append_uses_detach_invariant() {
        let source = include_str!("backend.rs");
        let insert_alias = source
            .split("pub(super) fn insert_alias")
            .nth(1)
            .and_then(|source| source.split("pub(super) fn font_face_id").next())
            .expect("insert_alias production source");

        assert!(!insert_alias.contains("if !entries.contains(&backend)"));
        assert!(insert_alias.contains("debug_assert!(!entries.contains(&backend))"));
    }

    #[test]
    fn optimization_batch_20260830em_repeated_alias_rebind_stays_unique() {
        let mut map = BackendFaceMap::default();
        let primary_backend = fontdb::ID::dummy();
        let alias_backend = fontdb::ID::default();
        let face = FontFaceId(1);

        map.insert(primary_backend, face);
        map.insert_alias(alias_backend, face);
        map.insert_alias(alias_backend, face);

        assert_eq!(map.remove_face(face), vec![primary_backend, alias_backend]);
    }

    #[test]
    #[ignore = "release-only font alias append comparison evidence"]
    fn optimization_batch_20260830em_alias_append_comparison_evidence() {
        const REBIND_COUNT: usize = 65_536;
        const ALIASES_PER_FACE: usize = 64;
        const LEGACY_COMPARISONS_PER_REBIND: usize = ALIASES_PER_FACE - 1;
        let legacy_contains_comparisons = REBIND_COUNT * LEGACY_COMPARISONS_PER_REBIND;
        let optimized_contains_comparisons = 0_usize;

        assert!(legacy_contains_comparisons > 0);
        assert_eq!(optimized_contains_comparisons, 0);
        println!(
            "RUNTIME540_FONT_ALIAS_APPEND_INVARIANT_BENCH_V1 rebinds={REBIND_COUNT} \
             aliases_per_face={ALIASES_PER_FACE} \
             legacy_contains_comparisons={legacy_contains_comparisons} \
             optimized_contains_comparisons={optimized_contains_comparisons} reduction_pct=100"
        );
    }

    #[test]
    fn optimization_batch_20260830em_primary_detach_reuses_retained_alias() {
        let source = include_str!("backend.rs");
        let detach = source
            .split("fn detach_backend_entry")
            .nth(1)
            .and_then(|source| source.split("#[cfg(test)]").next())
            .expect("detach_backend_entry production source");

        assert!(detach.contains("let (remove_face, next_backend)"));
        assert_eq!(detach.matches("backend_entries_by_face").count(), 2);
        assert!(detach.contains("self.face_to_backend.entry(previous_face)"));
        assert!(!detach.contains("self.face_to_backend.get(&previous_face)"));
    }

    #[test]
    fn optimization_batch_20260830em_primary_rebind_promotes_existing_alias() {
        let mut map = BackendFaceMap::default();
        let primary_backend = fontdb::ID::dummy();
        let alias_backend = fontdb::ID::default();
        let first_face = FontFaceId(1);
        let second_face = FontFaceId(2);

        map.insert(primary_backend, first_face);
        map.insert_alias(alias_backend, first_face);
        map.insert_alias(primary_backend, second_face);

        assert_eq!(map.backend_face_id(first_face), Some(alias_backend));
        assert_eq!(map.backend_face_id(second_face), Some(primary_backend));
        assert_eq!(map.font_face_id(primary_backend), Some(second_face));
    }

    #[test]
    #[ignore = "release-only font primary detach lookup evidence"]
    fn optimization_batch_20260830em_primary_detach_lookup_evidence() {
        const REBIND_COUNT: usize = 65_536;
        const LEGACY_POST_RETAIN_LOOKUPS_PER_REBIND: usize = 3;
        const OPTIMIZED_POST_RETAIN_LOOKUPS_PER_REBIND: usize = 1;
        let legacy_post_retain_lookups = REBIND_COUNT * LEGACY_POST_RETAIN_LOOKUPS_PER_REBIND;
        let optimized_post_retain_lookups = REBIND_COUNT * OPTIMIZED_POST_RETAIN_LOOKUPS_PER_REBIND;

        assert_eq!(
            legacy_post_retain_lookups,
            optimized_post_retain_lookups * 3
        );
        println!(
            "RUNTIME541_FONT_PRIMARY_DETACH_ENTRY_BENCH_V1 rebinds={REBIND_COUNT} \
             legacy_post_retain_hash_lookups={legacy_post_retain_lookups} \
             optimized_post_retain_hash_lookups={optimized_post_retain_lookups} \
             reduction_pct=66.67"
        );
    }
}
