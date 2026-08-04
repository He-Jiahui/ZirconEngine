use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

use glyphon::fontdb;

use super::{FontDatabase, FontDatabaseError, StoredFontSource};
use crate::text::font::descriptors::{descriptor_from_fontdb_face, source_key_from_fontdb_source};
use crate::text::FontFaceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SystemFontPolicy {
    #[default]
    Disabled,
    Discover,
}

impl FontDatabase {
    pub(crate) fn apply_system_font_policy(&mut self, policy: SystemFontPolicy) -> usize {
        if policy == SystemFontPolicy::Disabled || self.system_fonts_discovered {
            return 0;
        }
        let existing_backend_faces = self
            .backend_database
            .faces()
            .map(|face| face.id)
            .collect::<HashSet<_>>();
        self.backend_database.load_system_fonts();
        let system_faces = self
            .backend_database
            .faces()
            .filter(|face| !existing_backend_faces.contains(&face.id))
            .cloned()
            .collect::<Vec<_>>();
        let before = self.faces.len();
        for info in &system_faces {
            let _ = self.register_system_face(info);
        }
        if self.faces.len() > before {
            self.detach_face_dependent_caches();
        }
        self.system_fonts_discovered = true;
        self.faces.len().saturating_sub(before)
    }

    fn register_system_face(
        &mut self,
        info: &fontdb::FaceInfo,
    ) -> Result<Option<FontFaceId>, FontDatabaseError> {
        let Some(descriptor) = descriptor_from_fontdb_face(info) else {
            return Ok(None);
        };
        let Some(source_key) = source_key_from_fontdb_source(&info.source, info.index) else {
            return Ok(None);
        };
        if let Some(face) = self.source_face_index.get(&source_key) {
            return Ok(Some(*face));
        }

        let id = self.register_stored_font_source_with_backend(
            descriptor,
            StoredFontSource::FontDb {
                source: info.source.clone(),
            },
            Arc::new(OnceLock::new()),
            None,
            Some(info.id),
            false,
        )?;
        self.source_face_index.insert(source_key, id);
        Ok(Some(id))
    }
}
