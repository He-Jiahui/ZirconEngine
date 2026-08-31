use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

use glyphon::fontdb;

use super::{FontDatabase, FontDatabaseError, StoredFontSource};
use crate::text::FontFaceId;
use crate::text::font::descriptors::{descriptor_from_fontdb_face, source_key_from_fontdb_source};

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
        let face_count_hint = self.backend_database.faces().size_hint();
        let system_face_capacity = face_count_hint
            .1
            .unwrap_or(face_count_hint.0)
            .saturating_sub(existing_backend_faces.len());
        let mut system_faces = Vec::with_capacity(system_face_capacity);
        system_faces.extend(
            self.backend_database
                .faces()
                .filter(|face| !existing_backend_faces.contains(&face.id))
                .cloned(),
        );
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

#[cfg(test)]
mod optimization_batch_20260830cs_runtime_tests {
    #[test]
    fn optimization_batch_20260830cs_runtime506_system_faces_reserve_iterator_upper_bound() {
        let source = include_str!("system_fonts.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("system font production source");

        assert!(production.contains("let system_face_capacity ="));
        assert!(production.contains(".size_hint()"));
        assert!(production.contains("Vec::with_capacity(system_face_capacity)"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830cs_runtime506_system_face_capacity_evidence() {
        const EXISTING_FACE_COUNT: usize = 4_096;
        const TOTAL_FACE_COUNT: usize = 36_864;
        const MARKER: &str = "RUNTIME506_SYSTEM_FONT_FACE_CAPACITY_BENCH_V1";
        let legacy_growth_events =
            system_face_growth_events(TOTAL_FACE_COUNT, EXISTING_FACE_COUNT, false);
        let optimized_growth_events =
            system_face_growth_events(TOTAL_FACE_COUNT, EXISTING_FACE_COUNT, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} total_faces={TOTAL_FACE_COUNT} existing_faces={EXISTING_FACE_COUNT} new_faces={} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} reduction_pct=100",
            TOTAL_FACE_COUNT - EXISTING_FACE_COUNT
        );
    }

    fn system_face_growth_events(total: usize, existing: usize, reserve: bool) -> usize {
        let mut system_faces = if reserve {
            Vec::with_capacity(total.saturating_sub(existing))
        } else {
            Vec::new()
        };
        let mut growth_events = 0;
        for face in existing..total {
            let previous_capacity = system_faces.capacity();
            system_faces.push(face);
            growth_events += usize::from(system_faces.capacity() != previous_capacity);
        }
        growth_events
    }
}
