use std::collections::HashMap;

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
        if !entries.contains(&backend) {
            entries.push(backend);
        }
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
        let remove_face =
            if let Some(entries) = self.backend_entries_by_face.get_mut(&previous_face) {
                entries.retain(|entry| *entry != backend);
                entries.is_empty()
            } else {
                false
            };
        if remove_face {
            self.backend_entries_by_face.remove(&previous_face);
            self.face_to_backend.remove(&previous_face);
        } else if self.face_to_backend.get(&previous_face) == Some(&backend) {
            if let Some(next) = self
                .backend_entries_by_face
                .get(&previous_face)
                .and_then(|entries| entries.first())
                .copied()
            {
                self.face_to_backend.insert(previous_face, next);
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
}
