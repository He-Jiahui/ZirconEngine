use super::{FontDatabase, FontDatabaseError};
use glyphon::fontdb;
use zr_rhi::{UiSurfaceFontSnapshot, UiSurfaceFontSource};

impl FontDatabase {
    pub(crate) fn render_font_snapshot(
        &self,
        collection_id: u64,
        generation: u64,
    ) -> UiSurfaceFontSnapshot {
        let mut packet = UiSurfaceFontSnapshot {
            collection_id,
            generation,
            ..Default::default()
        };
        packet.generic_families = [
            fontdb::Family::Serif,
            fontdb::Family::SansSerif,
            fontdb::Family::Cursive,
            fontdb::Family::Fantasy,
            fontdb::Family::Monospace,
        ]
        .map(|family| self.backend_database.family_name(&family).to_owned());
        let result = (|| -> Result<(), FontDatabaseError> {
            // Backend entries include the existing private asset-family aliases.
            // Use their explicit mapping, never reconstruct ownership from names.
            for backend in self.backend_database.faces() {
                let Some(face) = self.font_face_id(backend.id) else {
                    continue;
                };
                let receipt = self.face_receipt_metadata(face)?;
                packet.sources.push(UiSurfaceFontSource {
                    bytes: self.face_bytes(face)?,
                    face_index: self.face_index(face)?,
                    raster_sha256: hex(&receipt.raster_sha256),
                    resource_path: receipt
                        .resource_path
                        .map(|path| path.to_string_lossy().into_owned()),
                    resource_sha256: receipt.resource_sha256.map(|sha| hex(&sha)),
                    families: backend
                        .families
                        .iter()
                        .map(|(name, _)| name.clone())
                        .collect(),
                    post_script_name: backend.post_script_name.clone(),
                    weight: backend.weight.0,
                    stretch: backend.stretch.to_number(),
                    style: match backend.style {
                        fontdb::Style::Normal => 0,
                        fontdb::Style::Italic => 1,
                        fontdb::Style::Oblique => 2,
                    },
                    monospaced: backend.monospaced,
                });
            }
            Ok(())
        })();
        if let Err(error) = result {
            packet.sources.clear();
            packet.admission_error = Some(error.to_string());
        }
        packet
    }
}

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exported_collection_retains_admitted_face_bytes_and_resource_receipt() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/fonts/FiraMono-subset.ttf");
        let bytes = include_bytes!("../../../../assets/fonts/FiraMono-subset.ttf");
        let mut database = FontDatabase::default();
        let face = database
            .register_font_file(&path, Some("Admitted owner alias"), 0)
            .unwrap();
        assert_eq!(
            database.face_family_name(face).unwrap().as_str(),
            "Admitted owner alias"
        );
        let backend = database.backend_face_id(face).unwrap();
        assert_eq!(database.font_face_id(backend), Some(face));
        let renderer_family = database.backend_database.face(backend).unwrap().families[0]
            .0
            .clone();
        assert_ne!(renderer_family, "Admitted owner alias");
        let renderer_alias = "Explicit renderer owner alias";
        assert!(database.register_font_family_alias(face, renderer_alias.into()));
        let families = [fontdb::Family::Name(renderer_alias)];
        let alias_backend = database
            .backend_database
            .query(&fontdb::Query {
                families: &families,
                ..fontdb::Query::default()
            })
            .unwrap();
        assert_ne!(alias_backend, backend);
        assert_eq!(database.font_face_id(alias_backend), Some(face));
        assert_eq!(database.backend_face_id(face), Some(backend));
        let packet = database.render_font_snapshot(42, 7);
        assert!(packet.admission_error.is_none());
        assert_eq!((packet.collection_id, packet.generation), (42, 7));
        assert!(packet.sources.iter().any(|source| source
            .families
            .iter()
            .any(|family| family == &renderer_family)));
        let source = packet
            .sources
            .iter()
            .find(|source| {
                source
                    .families
                    .iter()
                    .any(|family| family == renderer_alias)
            })
            .unwrap();
        assert_eq!(source.face_index, 0);
        assert_eq!(source.bytes.as_ref(), bytes);
        assert_eq!(source.resource_sha256.as_ref(), Some(&source.raster_sha256));
        assert_eq!(
            source.resource_path.as_deref(),
            path.canonicalize().unwrap().to_str()
        );
        drop(database);
        assert_eq!(source.bytes.as_ref(), bytes);
    }
}
