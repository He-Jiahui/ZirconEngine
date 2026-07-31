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
}

impl BackendFaceMap {
    pub(super) fn insert(&mut self, backend: fontdb::ID, face: FontFaceId) {
        if let Some(previous_backend) = self.face_to_backend.insert(face, backend) {
            self.backend_to_face.remove(&previous_backend);
        }
        if let Some(previous_face) = self.backend_to_face.insert(backend, face) {
            self.face_to_backend.remove(&previous_face);
        }
    }

    pub(super) fn font_face_id(&self, backend: fontdb::ID) -> Option<FontFaceId> {
        self.backend_to_face.get(&backend).copied()
    }

    pub(super) fn backend_face_id(&self, face: FontFaceId) -> Option<fontdb::ID> {
        self.face_to_backend.get(&face).copied()
    }

    pub(super) fn remove_face(&mut self, face: FontFaceId) -> Option<fontdb::ID> {
        let backend = self.face_to_backend.remove(&face)?;
        self.backend_to_face.remove(&backend);
        Some(backend)
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
}
