use glyphon::{fontdb, FontSystem};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeDigestV1;
use zr_rhi::UiSurfaceFontSnapshot;

/// Installs only admitted SFNT/TTC bytes. It never opens a file or discovers fonts.
pub(super) fn install(
    snapshot: &UiSurfaceFontSnapshot,
    fonts: &mut FontSystem,
) -> Result<(), String> {
    if let Some(error) = &snapshot.admission_error {
        return Err(error.clone());
    }
    if snapshot.sources.is_empty() {
        return Err("font collection admission is pending".to_owned());
    }
    let mut database = fontdb::Database::new();
    for source in &snapshot.sources {
        if ZrRuntimeDigestV1::sha256(source.bytes.as_ref()).as_str() != source.raster_sha256 {
            return Err("admitted font byte digest mismatch".to_owned());
        }
        let mut parsed = fontdb::Database::new();
        parsed.load_font_source(fontdb::Source::Binary(std::sync::Arc::new(
            source.bytes.clone(),
        )));
        let mut face = parsed
            .faces()
            .find(|face| face.index == source.face_index)
            .cloned()
            .ok_or_else(|| "admitted TTC face index is unavailable".to_owned())?;
        let language = face
            .families
            .first()
            .map(|(_, language)| *language)
            .ok_or_else(|| "admitted face has no family metadata".to_owned())?;
        face.families = source
            .families
            .iter()
            .map(|name| (name.clone(), language))
            .collect();
        face.post_script_name = source.post_script_name.clone();
        face.weight = fontdb::Weight(source.weight);
        face.stretch = match source.stretch {
            1 => fontdb::Stretch::UltraCondensed,
            2 => fontdb::Stretch::ExtraCondensed,
            3 => fontdb::Stretch::Condensed,
            4 => fontdb::Stretch::SemiCondensed,
            5 => fontdb::Stretch::Normal,
            6 => fontdb::Stretch::SemiExpanded,
            7 => fontdb::Stretch::Expanded,
            8 => fontdb::Stretch::ExtraExpanded,
            9 => fontdb::Stretch::UltraExpanded,
            _ => return Err("invalid admitted stretch".to_owned()),
        };
        face.style = match source.style {
            0 => fontdb::Style::Normal,
            1 => fontdb::Style::Italic,
            2 => fontdb::Style::Oblique,
            _ => return Err("invalid admitted style".to_owned()),
        };
        face.monospaced = source.monospaced;
        database.push_face_info(face);
    }
    database.set_serif_family(snapshot.generic_families[0].clone());
    database.set_sans_serif_family(snapshot.generic_families[1].clone());
    database.set_cursive_family(snapshot.generic_families[2].clone());
    database.set_fantasy_family(snapshot.generic_families[3].clone());
    database.set_monospace_family(snapshot.generic_families[4].clone());
    *fonts = FontSystem::new_with_locale_and_db(fonts.locale().to_owned(), database);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const BYTES: &[u8] = include_bytes!("../../../../../assets/fonts/FiraMono-subset.ttf");
    fn packet(generation: u64, family: &str) -> UiSurfaceFontSnapshot {
        UiSurfaceFontSnapshot {
            collection_id: 1,
            generation,
            sources: vec![zr_rhi::UiSurfaceFontSource {
                bytes: std::sync::Arc::from(BYTES),
                face_index: 0,
                raster_sha256: ZrRuntimeDigestV1::sha256(BYTES).as_str().to_owned(),
                resource_path: Some("admitted/FiraMono-subset.ttf".to_owned()),
                resource_sha256: None,
                families: vec![family.to_owned()],
                post_script_name: "admitted-mono".to_owned(),
                weight: 400,
                stretch: 5,
                style: 0,
                monospaced: true,
            }],
            ..Default::default()
        }
    }
    #[test]
    fn installed_asset_face_uses_exact_bytes_and_replacement_retires_old_family() {
        let mut fonts =
            FontSystem::new_with_locale_and_db("en-US".to_owned(), fontdb::Database::new());
        install(&packet(1, "First admitted alias"), &mut fonts).unwrap();
        let id = fonts
            .db()
            .query(&fontdb::Query {
                families: &[fontdb::Family::Name("First admitted alias")],
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            fonts
                .db()
                .with_face_data(id, |bytes, index| (bytes.to_vec(), index)),
            Some((BYTES.to_vec(), 0))
        );
        install(&packet(2, "Replacement admitted alias"), &mut fonts).unwrap();
        assert!(fonts
            .db()
            .query(&fontdb::Query {
                families: &[fontdb::Family::Name("First admitted alias")],
                ..Default::default()
            })
            .is_none());
        assert!(fonts
            .db()
            .query(&fontdb::Query {
                families: &[fontdb::Family::Name("Replacement admitted alias")],
                ..Default::default()
            })
            .is_some());
        let mut invalid = packet(3, "Invalid index");
        invalid.sources[0].face_index = 7;
        assert!(install(&invalid, &mut fonts).is_err());
        assert!(fonts
            .db()
            .query(&fontdb::Query {
                families: &[fontdb::Family::Name("Replacement admitted alias")],
                ..Default::default()
            })
            .is_some());
    }
}
