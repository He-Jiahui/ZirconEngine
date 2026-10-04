use std::collections::{hash_map::Entry, HashMap};

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
#[path = "tests/backend.rs"]
mod tests;
