use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use glyphon::{fontdb, FontSystem};
use ttf_parser::{name_id, Face, Style as TtfStyle};

use crate::asset::FontAsset;
use crate::core::framework::render::{
    CompositeFontDescriptor, FontFaceDescriptor, FontFaceId, FontFamilyName, FontMatch, FontQuery,
    FontScript, FontStretch, FontStyle, FontWeight, InstancedFaceId, SubFontRange, VariationCoords,
};

use super::asset_registration::{font_asset_descriptors, FontAssetSourceKey};
use super::coverage::FontCoverage;
use super::default_families::default_runtime_font_families;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FontDatabaseError {
    EmptyFamily,
    EmptyBytes,
    ReadFailed,
    FaceBytesUnavailable(FontFaceId),
    UnknownFace(FontFaceId),
}

#[derive(Clone, Debug)]
struct StoredFontFace {
    descriptor: FontFaceDescriptor,
    source: StoredFontSource,
    coverage: FontCoverage,
}

#[derive(Clone, Debug)]
enum StoredFontSource {
    SharedBytes(Arc<[u8]>),
    FontDb { source: fontdb::Source },
}

impl StoredFontSource {
    fn is_empty(&self) -> bool {
        match self {
            Self::SharedBytes(bytes) => bytes.is_empty(),
            Self::FontDb { .. } => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FontSourceKey {
    path: PathBuf,
    face_index: u32,
}

impl FontSourceKey {
    fn from_path(path: impl AsRef<Path>, face_index: u32) -> Self {
        Self {
            path: canonical_source_key(path.as_ref()),
            face_index,
        }
    }
}

#[derive(Clone, Debug)]
struct SharedFontBytes {
    bytes: Arc<[u8]>,
}

#[derive(Clone, Debug)]
pub(crate) struct FontDatabase {
    faces: Vec<StoredFontFace>,
    family_index: HashMap<String, Vec<FontFaceId>>,
    source_face_index: HashMap<FontSourceKey, FontFaceId>,
    asset_source_index: HashMap<FontAssetSourceKey, FontFaceId>,
    fallback_families: Vec<FontFamilyName>,
}

impl Default for FontDatabase {
    fn default() -> Self {
        Self {
            faces: Vec::new(),
            family_index: HashMap::new(),
            source_face_index: HashMap::new(),
            asset_source_index: HashMap::new(),
            fallback_families: Vec::new(),
        }
    }
}

impl FontDatabase {
    pub(crate) fn with_default_fallbacks() -> Self {
        let mut database = Self::default();
        database.fallback_families = default_runtime_font_families();
        database
    }

    pub(crate) fn fallback_families(&self) -> &[FontFamilyName] {
        &self.fallback_families
    }

    pub(crate) fn register_font_file(
        &mut self,
        source_path: impl AsRef<Path>,
        family: Option<&str>,
        face_index: u32,
    ) -> Result<FontFaceId, FontDatabaseError> {
        let source_path = source_path.as_ref();
        let source_key = FontSourceKey::from_path(source_path, face_index);
        if let Some(face) = self.source_face_index.get(&source_key) {
            return Ok(*face);
        }

        let bytes = std::fs::read(source_path).map_err(|_| FontDatabaseError::ReadFailed)?;
        let descriptor = descriptor_from_font_bytes(&bytes, family, source_path, face_index);
        self.register_stored_face(
            descriptor,
            Arc::from(bytes.into_boxed_slice()),
            Some(source_key.path),
        )
    }

    pub(crate) fn register_font_asset(
        &mut self,
        asset: &FontAsset,
        source_path: impl AsRef<Path>,
    ) -> Result<Vec<FontFaceId>, FontDatabaseError> {
        let source_path = source_path.as_ref();
        let bytes = std::fs::read(source_path).map_err(|_| FontDatabaseError::ReadFailed)?;
        let bytes: Arc<[u8]> = Arc::from(bytes.into_boxed_slice());
        let mut faces = Vec::new();

        for descriptor in font_asset_descriptors(asset, bytes.as_ref(), source_path) {
            let face =
                self.register_asset_descriptor(descriptor, Arc::clone(&bytes), source_path)?;
            if !faces.contains(&face) {
                faces.push(face);
            }
        }
        self.extend_fallback_families(asset.fallback_families.iter().map(String::as_str));
        Ok(faces)
    }

    pub(crate) fn load_system_fonts(&mut self) -> usize {
        let mut system_database = fontdb::Database::new();
        system_database.load_system_fonts();
        let before = self.faces.len();
        for info in system_database.faces() {
            let _ = self.register_system_face(info);
        }
        self.faces.len().saturating_sub(before)
    }

    pub(crate) fn load_face_into_font_system(
        &self,
        face: FontFaceId,
        font_system: &mut FontSystem,
    ) -> Result<(), FontDatabaseError> {
        let source = self.glyphon_source(face)?;
        font_system.db_mut().load_font_source(source);
        Ok(())
    }

    fn register_stored_face(
        &mut self,
        descriptor: FontFaceDescriptor,
        bytes: Arc<[u8]>,
        source_path: Option<PathBuf>,
    ) -> Result<FontFaceId, FontDatabaseError> {
        let coverage = FontCoverage::from_sfnt_bytes(bytes.as_ref(), descriptor.face_index);
        self.register_stored_font_source(
            descriptor,
            StoredFontSource::SharedBytes(bytes),
            coverage,
            source_path,
        )
    }

    fn register_stored_font_source(
        &mut self,
        descriptor: FontFaceDescriptor,
        source: StoredFontSource,
        coverage: FontCoverage,
        source_path: Option<PathBuf>,
    ) -> Result<FontFaceId, FontDatabaseError> {
        if descriptor.family.is_empty() {
            return Err(FontDatabaseError::EmptyFamily);
        }
        if source.is_empty() {
            return Err(FontDatabaseError::EmptyBytes);
        }

        let id = FontFaceId(self.faces.len() as u64 + 1);
        let family_key = normalized_family_key(descriptor.family.as_str());
        let face_index = descriptor.face_index;
        self.faces.push(StoredFontFace {
            descriptor,
            source,
            coverage,
        });
        self.family_index.entry(family_key).or_default().push(id);
        if let Some(source_path) = source_path {
            self.source_face_index.insert(
                FontSourceKey {
                    path: source_path,
                    face_index,
                },
                id,
            );
        }
        let _ = self.instance(id, &VariationCoords::default())?;
        Ok(id)
    }

    fn register_asset_descriptor(
        &mut self,
        descriptor: FontFaceDescriptor,
        bytes: Arc<[u8]>,
        source_path: &Path,
    ) -> Result<FontFaceId, FontDatabaseError> {
        let source_key = FontAssetSourceKey::from_descriptor(source_path, &descriptor);
        if let Some(face) = self.asset_source_index.get(&source_key) {
            return Ok(*face);
        }
        let face = self.register_stored_face(descriptor, bytes, None)?;
        self.asset_source_index.insert(source_key, face);
        Ok(face)
    }

    fn extend_fallback_families<'a>(&mut self, families: impl IntoIterator<Item = &'a str>) {
        for family in families {
            let family = FontFamilyName::from(family);
            if family.is_empty() {
                continue;
            }
            let key = normalized_family_key(family.as_str());
            if self
                .fallback_families
                .iter()
                .any(|existing| normalized_family_key(existing.as_str()) == key)
            {
                continue;
            }
            self.fallback_families.push(family);
        }
    }

    pub(crate) fn match_face(&self, query: &FontQuery) -> Option<FontMatch> {
        let mut families = query.families.clone();
        families.extend(self.fallback_families.iter().cloned());
        self.match_face_in_family_order(&families, query)
    }

    pub(crate) fn fallback_candidates(
        &self,
        codepoint: char,
        query: &FontQuery,
        composite: Option<&CompositeFontDescriptor>,
    ) -> Vec<FontFaceId> {
        let mut families = Vec::new();
        if let Some(composite) = composite {
            for sub_font in &composite.sub_fonts {
                if sub_font_matches(codepoint, sub_font) {
                    families.push(sub_font.family.clone());
                }
            }
            families.push(composite.default_family.clone());
        }
        families.extend(query.families.iter().cloned());
        families.extend(self.fallback_families.iter().cloned());

        let mut candidates = Vec::new();
        for family in dedupe_families(families) {
            for face in self.family_candidates_for_codepoint(&family, query, codepoint) {
                if !candidates.contains(&face) {
                    candidates.push(face);
                }
            }
        }
        candidates
    }

    pub(crate) fn face_bytes(&self, face: FontFaceId) -> Result<Arc<[u8]>, FontDatabaseError> {
        let stored = self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        match &stored.source {
            StoredFontSource::SharedBytes(bytes) => Ok(Arc::clone(bytes)),
            StoredFontSource::FontDb { .. } => Err(FontDatabaseError::FaceBytesUnavailable(face)),
        }
    }

    pub(crate) fn face_index(&self, face: FontFaceId) -> Result<u32, FontDatabaseError> {
        Ok(self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?
            .descriptor
            .face_index)
    }

    pub(crate) fn instance(
        &self,
        face: FontFaceId,
        variations: &VariationCoords,
    ) -> Result<InstancedFaceId, FontDatabaseError> {
        if self.face(face).is_none() {
            return Err(FontDatabaseError::UnknownFace(face));
        }

        let mut hash = face.0.wrapping_mul(1_099_511_628_211);
        for (tag, value) in &variations.0 {
            hash ^= *tag as u64;
            hash = hash.wrapping_mul(1_099_511_628_211);
            hash ^= value.to_bits() as u64;
            hash = hash.wrapping_mul(1_099_511_628_211);
        }
        Ok(InstancedFaceId(hash))
    }

    fn face(&self, face: FontFaceId) -> Option<&StoredFontFace> {
        let index = face.0.checked_sub(1)? as usize;
        self.faces.get(index)
    }

    fn glyphon_source(&self, face: FontFaceId) -> Result<fontdb::Source, FontDatabaseError> {
        let stored = self
            .face(face)
            .ok_or(FontDatabaseError::UnknownFace(face))?;
        match &stored.source {
            StoredFontSource::SharedBytes(bytes) => {
                Ok(fontdb::Source::Binary(Arc::new(SharedFontBytes {
                    bytes: Arc::clone(bytes),
                })))
            }
            StoredFontSource::FontDb { source, .. } => Ok(source.clone()),
        }
    }

    fn match_face_in_family_order(
        &self,
        families: &[FontFamilyName],
        query: &FontQuery,
    ) -> Option<FontMatch> {
        dedupe_families(families.iter().cloned())
            .into_iter()
            .filter_map(|family| self.family_candidates(&family, query).into_iter().next())
            .next()
            .map(|face| FontMatch {
                face,
                synthetic_bold: false,
                synthetic_oblique: false,
            })
    }

    fn family_candidates(&self, family: &FontFamilyName, query: &FontQuery) -> Vec<FontFaceId> {
        let mut candidates = self
            .family_index
            .get(&normalized_family_key(family.as_str()))
            .cloned()
            .unwrap_or_default();
        candidates.sort_by_key(|id| self.match_score(*id, query));
        candidates
    }

    fn family_candidates_for_codepoint(
        &self,
        family: &FontFamilyName,
        query: &FontQuery,
        codepoint: char,
    ) -> Vec<FontFaceId> {
        self.family_candidates(family, query)
            .into_iter()
            .filter(|face| self.face_covers_codepoint(*face, codepoint))
            .collect()
    }

    fn face_covers_codepoint(&self, face: FontFaceId, codepoint: char) -> bool {
        self.face(face)
            .is_some_and(|stored| stored.coverage.contains(codepoint))
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

        let id = self.register_stored_font_source(
            descriptor,
            StoredFontSource::FontDb {
                source: info.source.clone(),
            },
            FontCoverage::Unknown,
            None,
        )?;
        self.source_face_index.insert(source_key, id);
        Ok(Some(id))
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

impl AsRef<[u8]> for SharedFontBytes {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

pub(super) fn normalized_family_key(family: &str) -> String {
    family.trim().to_ascii_lowercase()
}

pub(super) fn canonical_source_key(source_path: &Path) -> PathBuf {
    std::fs::canonicalize(source_path).unwrap_or_else(|_| source_path.to_path_buf())
}

fn family_from_source_path(source_path: &Path) -> String {
    source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(['_', '-'], " "))
        .unwrap_or_else(|| "Zircon Sans".to_string())
}

pub(super) fn descriptor_from_font_bytes(
    bytes: &[u8],
    family: Option<&str>,
    source_path: &Path,
    face_index: u32,
) -> FontFaceDescriptor {
    let parsed = Face::parse(bytes, face_index).ok();
    let family = family
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .map(FontFamilyName::from)
        .or_else(|| {
            parsed
                .as_ref()
                .and_then(face_family_name)
                .map(FontFamilyName::from)
        })
        .unwrap_or_else(|| FontFamilyName::from(family_from_source_path(source_path)));

    let Some(face) = parsed else {
        let mut descriptor = FontFaceDescriptor::regular(family);
        descriptor.face_index = face_index;
        return descriptor;
    };

    FontFaceDescriptor {
        family,
        weight: FontWeight::clamped(face.weight().to_number()),
        style: style_from_ttf(face.style()),
        stretch: stretch_from_ttf_width_class(face.width().to_number()),
        face_index,
        variations: VariationCoords::default(),
    }
}

fn face_family_name(face: &Face<'_>) -> Option<String> {
    ttf_name_by_id(face, name_id::TYPOGRAPHIC_FAMILY)
        .or_else(|| ttf_name_by_id(face, name_id::FAMILY))
}

fn ttf_name_by_id(face: &Face<'_>, id: u16) -> Option<String> {
    face.names()
        .into_iter()
        .filter(|name| name.name_id == id)
        .filter_map(|name| name.to_string())
        .find(|value| !value.trim().is_empty())
}

fn style_from_ttf(style: TtfStyle) -> FontStyle {
    match style {
        TtfStyle::Normal => FontStyle::Normal,
        TtfStyle::Italic => FontStyle::Italic,
        TtfStyle::Oblique => FontStyle::Oblique(0.0),
    }
}

pub(super) fn stretch_from_ttf_width_class(width_class: u16) -> FontStretch {
    let percent = match width_class {
        1 => 50,
        2 => 63,
        3 => 75,
        4 => 88,
        5 => 100,
        6 => 113,
        7 => 125,
        8 => 150,
        9 => 200,
        _ => 100,
    };
    FontStretch::clamped(percent)
}

fn descriptor_from_fontdb_face(info: &fontdb::FaceInfo) -> Option<FontFaceDescriptor> {
    let family = info.families.first()?.0.as_str();
    Some(FontFaceDescriptor {
        family: FontFamilyName::from(family),
        weight: FontWeight::clamped(info.weight.0),
        style: style_from_fontdb(info.style),
        stretch: FontStretch::clamped(info.stretch.to_number()),
        face_index: info.index,
        variations: VariationCoords::default(),
    })
}

fn style_from_fontdb(style: fontdb::Style) -> FontStyle {
    match style {
        fontdb::Style::Normal => FontStyle::Normal,
        fontdb::Style::Italic => FontStyle::Italic,
        fontdb::Style::Oblique => FontStyle::Oblique(0.0),
    }
}

fn source_key_from_fontdb_source(
    source: &fontdb::Source,
    face_index: u32,
) -> Option<FontSourceKey> {
    match source {
        fontdb::Source::File(path) => Some(FontSourceKey::from_path(path, face_index)),
        fontdb::Source::SharedFile(path, _) => Some(FontSourceKey::from_path(path, face_index)),
        fontdb::Source::Binary(_) => None,
    }
}

fn dedupe_families(families: impl IntoIterator<Item = FontFamilyName>) -> Vec<FontFamilyName> {
    let mut result: Vec<FontFamilyName> = Vec::new();
    for family in families {
        if family.is_empty() {
            continue;
        }
        let key = normalized_family_key(family.as_str());
        if result
            .iter()
            .any(|existing| normalized_family_key(existing.as_str()) == key)
        {
            continue;
        }
        result.push(family);
    }
    result
}

fn sub_font_matches(codepoint: char, sub_font: &SubFontRange) -> bool {
    let cp = codepoint as u32;
    let range_match = sub_font
        .ranges
        .iter()
        .any(|(start, end)| *start <= cp && cp <= *end);
    let script_match = sub_font
        .scripts
        .iter()
        .any(|script| *script == script_for_char(codepoint));

    (!sub_font.ranges.is_empty() && range_match) || (!sub_font.scripts.is_empty() && script_match)
}

fn script_for_char(codepoint: char) -> FontScript {
    match codepoint as u32 {
        0x0041..=0x007A | 0x00C0..=0x024F | 0x1E00..=0x1EFF => FontScript::Latin,
        0x0370..=0x03FF | 0x1F00..=0x1FFF => FontScript::Greek,
        0x0400..=0x052F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F => FontScript::Cyrillic,
        0x0590..=0x05FF => FontScript::Hebrew,
        0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF => FontScript::Arabic,
        0x0900..=0x097F => FontScript::Devanagari,
        0x3040..=0x309F => FontScript::Hiragana,
        0x30A0..=0x30FF | 0x31F0..=0x31FF => FontScript::Katakana,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF => FontScript::Han,
        0xAC00..=0xD7AF | 0x1100..=0x11FF | 0x3130..=0x318F => FontScript::Hangul,
        other => FontScript::Other(other),
    }
}

fn weight_distance(candidate: FontWeight, requested: FontWeight) -> u16 {
    candidate.0.abs_diff(requested.0)
}

fn stretch_distance(candidate: FontStretch, requested: FontStretch) -> u16 {
    candidate.0.abs_diff(requested.0)
}

fn style_distance(candidate: FontStyle, requested: FontStyle) -> u8 {
    match (candidate, requested) {
        (FontStyle::Normal, FontStyle::Normal) => 0,
        (FontStyle::Italic, FontStyle::Italic) => 0,
        (FontStyle::Oblique(_), FontStyle::Oblique(_)) => 0,
        (FontStyle::Italic, FontStyle::Oblique(_)) | (FontStyle::Oblique(_), FontStyle::Italic) => {
            1
        }
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{
        FontAsset, FontAssetFaceStyle, FontAssetFamilyMember, FontAssetRenderStrategy,
    };
    use crate::core::framework::render::{
        CompositeFontDescriptor, FontFaceDescriptor, FontFamilyName, FontScript, SubFontRange,
    };
    use crate::graphics::text::font::test_font_fixtures::{
        write_ttc_fixture, write_weight_fixture,
    };

    #[test]
    fn text_font_database_query_best_match_weight_distance() {
        let mut database = FontDatabase::default();
        let regular = database
            .register_stored_face(
                FontFaceDescriptor::regular("Inter"),
                Arc::from([1_u8, 2, 3].as_slice()),
                None,
            )
            .unwrap();
        let mut bold_face = FontFaceDescriptor::regular("Inter");
        bold_face.weight = FontWeight::BOLD;
        let bold = database
            .register_stored_face(bold_face, Arc::from([4_u8, 5, 6].as_slice()), None)
            .unwrap();

        let query = FontQuery {
            families: vec![FontFamilyName::from("Inter")],
            weight: FontWeight::BOLD,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
        };

        assert_eq!(database.match_face(&query).unwrap().face, bold);
        assert_ne!(database.match_face(&query).unwrap().face, regular);
    }

    #[test]
    fn text_font_face_shares_arc_bytes_across_backends() {
        let mut database = FontDatabase::default();
        let bytes: Arc<[u8]> = Arc::from([9_u8, 8, 7, 6].as_slice());
        let face = database
            .register_stored_face(
                FontFaceDescriptor::regular("Inter"),
                Arc::clone(&bytes),
                None,
            )
            .unwrap();

        let glyphon_bytes = database.face_bytes(face).unwrap();
        let sdf_bytes = database.face_bytes(face).unwrap();

        assert!(Arc::ptr_eq(&glyphon_bytes, &sdf_bytes));
        assert!(Arc::ptr_eq(&glyphon_bytes, &bytes));
    }

    #[test]
    fn text_font_variations_hash_stable() {
        let mut database = FontDatabase::default();
        let face = database
            .register_stored_face(
                FontFaceDescriptor::regular("Inter"),
                Arc::from([1_u8].as_slice()),
                None,
            )
            .unwrap();
        let variations = VariationCoords(vec![(u32::from_be_bytes(*b"wght"), 650.0)]);
        let same = VariationCoords(vec![(u32::from_be_bytes(*b"wght"), 650.0)]);
        let different = VariationCoords(vec![(u32::from_be_bytes(*b"wght"), 700.0)]);

        assert_eq!(
            database.instance(face, &variations).unwrap(),
            database.instance(face, &same).unwrap()
        );
        assert_ne!(
            database.instance(face, &variations).unwrap(),
            database.instance(face, &different).unwrap()
        );
    }

    #[test]
    fn text_font_database_registers_file_once_and_feeds_glyphon_fontdb() {
        let source_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraMono-subset.ttf");
        let mut database = FontDatabase::default();

        let first = database
            .register_font_file(&source_path, Some("Fira Mono"), 0)
            .unwrap();
        let second = database
            .register_font_file(&source_path, Some("Fira Mono"), 0)
            .unwrap();

        assert_eq!(first, second);
        assert!(database.face_bytes(first).unwrap().len() > 0);

        let mut font_system = FontSystem::new();
        database
            .load_face_into_font_system(first, &mut font_system)
            .unwrap();

        let families = [fontdb::Family::Name("Fira Mono")];
        let query = fontdb::Query {
            families: &families,
            ..fontdb::Query::default()
        };
        assert!(font_system.db_mut().query(&query).is_some());
    }

    #[test]
    fn text_font_database_reads_file_weight_for_best_match() {
        let regular_path = write_weight_fixture("regular", 400);
        let bold_path = write_weight_fixture("bold", 700);
        let mut database = FontDatabase::default();

        let regular = database
            .register_font_file(&regular_path, Some("Fira Metadata Test"), 0)
            .unwrap();
        let bold = database
            .register_font_file(&bold_path, Some("Fira Metadata Test"), 0)
            .unwrap();
        let query = FontQuery {
            families: vec![FontFamilyName::from("Fira Metadata Test")],
            weight: FontWeight::BOLD,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
        };

        assert_eq!(database.match_face(&query).unwrap().face, bold);
        assert_ne!(database.match_face(&query).unwrap().face, regular);

        let _ = std::fs::remove_file(regular_path);
        let _ = std::fs::remove_file(bold_path);
    }

    #[test]
    fn text_font_database_registers_ttc_faces_by_index() {
        let collection_path = write_ttc_fixture();
        let mut database = FontDatabase::default();

        let regular = database
            .register_font_file(&collection_path, Some("Fira TTC Test"), 0)
            .unwrap();
        let bold = database
            .register_font_file(&collection_path, Some("Fira TTC Test"), 1)
            .unwrap();
        let repeated_bold = database
            .register_font_file(&collection_path, Some("Fira TTC Test"), 1)
            .unwrap();
        let query = FontQuery {
            families: vec![FontFamilyName::from("Fira TTC Test")],
            weight: FontWeight::BOLD,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
        };

        assert_ne!(regular, bold);
        assert_eq!(bold, repeated_bold);
        assert_eq!(database.face_index(regular).unwrap(), 0);
        assert_eq!(database.face_index(bold).unwrap(), 1);
        assert_eq!(database.match_face(&query).unwrap().face, bold);

        let _ = std::fs::remove_file(collection_path);
    }

    #[test]
    fn text_font_database_registers_font_asset_family_members_and_fallbacks() {
        let source_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraMono-subset.ttf");
        let mut database = FontDatabase::default();
        let asset = FontAsset {
            source: "FiraMono-subset.ttf".to_string(),
            family: Some("Primary Sans".to_string()),
            render_mode: None,
            face_index: 0,
            family_members: vec![
                FontAssetFamilyMember {
                    family: "Primary Sans".to_string(),
                    face_index: 0,
                    weight: Some(400),
                    width_class: Some(5),
                    style: Some(FontAssetFaceStyle::Normal),
                    variations: Vec::new(),
                },
                FontAssetFamilyMember {
                    family: "Primary Sans".to_string(),
                    face_index: 0,
                    weight: Some(700),
                    width_class: Some(3),
                    style: Some(FontAssetFaceStyle::Italic),
                    variations: Vec::new(),
                },
            ],
            variable_instances: Vec::new(),
            fallback_families: vec!["Fallback Sans".to_string()],
            render_strategy: FontAssetRenderStrategy::default(),
            metadata: None,
        };
        let fallback = database
            .register_stored_face(
                FontFaceDescriptor::regular("Fallback Sans"),
                Arc::from([7_u8, 8, 9].as_slice()),
                None,
            )
            .unwrap();

        let registered = database.register_font_asset(&asset, &source_path).unwrap();
        let registered_again = database.register_font_asset(&asset, &source_path).unwrap();
        let regular_query = FontQuery {
            families: vec![FontFamilyName::from("Primary Sans")],
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
        };
        let bold_query = FontQuery {
            families: vec![FontFamilyName::from("Primary Sans")],
            weight: FontWeight::BOLD,
            style: FontStyle::Italic,
            stretch: FontStretch::clamped(75),
        };
        let fallback_query = FontQuery::single_family("Missing Primary");

        assert_eq!(registered.len(), 2);
        assert_eq!(registered_again, registered);
        assert_ne!(registered[0], registered[1]);
        assert_eq!(
            database.match_face(&regular_query).unwrap().face,
            registered[0]
        );
        assert_eq!(
            database.match_face(&bold_query).unwrap().face,
            registered[1]
        );
        assert_eq!(database.match_face(&fallback_query).unwrap().face, fallback);
        assert!(database
            .fallback_families()
            .iter()
            .any(|family| family.as_str() == "Fallback Sans"));
    }

    #[test]
    fn text_font_database_composite_candidates_prioritize_matching_subfont() {
        let mut database = FontDatabase::with_default_fallbacks();
        let latin = database
            .register_stored_face(
                FontFaceDescriptor::regular("Inter"),
                Arc::from([1_u8, 2, 3].as_slice()),
                None,
            )
            .unwrap();
        let cjk = database
            .register_stored_face(
                FontFaceDescriptor::regular("Noto Sans CJK SC"),
                Arc::from([4_u8, 5, 6].as_slice()),
                None,
            )
            .unwrap();
        let query = FontQuery::single_family("Inter");
        let composite = CompositeFontDescriptor {
            default_family: FontFamilyName::from("Inter"),
            sub_fonts: vec![SubFontRange {
                family: FontFamilyName::from("Noto Sans CJK SC"),
                scripts: vec![FontScript::Han],
                ranges: vec![(0x4E00, 0x9FFF)],
            }],
        };

        let cjk_candidates = database.fallback_candidates('界', &query, Some(&composite));
        assert_eq!(cjk_candidates.first().copied(), Some(cjk));
        assert!(cjk_candidates.contains(&latin));

        let latin_candidates = database.fallback_candidates('A', &query, Some(&composite));
        assert_eq!(latin_candidates.first().copied(), Some(latin));
    }

    #[test]
    fn text_font_fallback_candidates_filter_known_cmap_coverage() {
        let source_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
        let mut database = FontDatabase::default();
        let latin = database
            .register_font_file(&source_path, Some("Mixed Coverage"), 0)
            .unwrap();
        let unknown = database
            .register_stored_face(
                FontFaceDescriptor::regular("Mixed Coverage"),
                Arc::from([1_u8, 2, 3].as_slice()),
                None,
            )
            .unwrap();
        let query = FontQuery::single_family("Mixed Coverage");

        let latin_candidates = database.fallback_candidates('A', &query, None);
        let cjk_candidates = database.fallback_candidates('界', &query, None);

        assert!(latin_candidates.contains(&latin));
        assert!(latin_candidates.contains(&unknown));
        assert!(!cjk_candidates.contains(&latin));
        assert!(cjk_candidates.contains(&unknown));
    }
}
