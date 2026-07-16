use std::collections::HashMap;
use std::sync::Arc;

use ttf_parser::Face;

use crate::asset::ProjectAssetManager;
use crate::text::font::{load_text_font_source, FontDatabase};
use crate::text::sdf::{
    sdf_font_source_hash, sdf_offline_artifact_path, sdf_variation_hash, SdfOfflineArtifact,
    SdfOfflineArtifactIdentity,
};
use crate::text::FontFaceId;

use super::distance_field::glyph_id_for_key;
use super::{
    resolve_font_face, RawBakedGlyph, RawBakedGlyphSource, SdfAtlasGlyphKey, SdfGlyphMetrics,
};
#[derive(Default)]
pub(super) struct SdfOfflineSourceCache {
    artifacts: HashMap<SdfOfflineArtifactIdentity, Arc<SdfOfflineArtifact>>,
}

impl SdfOfflineSourceCache {
    pub(super) fn load_glyph(
        &mut self,
        key: &SdfAtlasGlyphKey,
        face_id: FontFaceId,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Option<RawBakedGlyph> {
        let font_ref = key.font.as_deref()?;
        let primary_face = resolve_font_face(Some(font_ref), font_database, asset_manager)?;
        if primary_face != face_id {
            return None;
        }
        let manifest = load_text_font_source(font_ref, Some(asset_manager))?;
        let asset_uuid = manifest.asset_uuid?;
        let face_bytes = font_database.standalone_face_bytes(face_id).ok()?;
        let instance = key
            .font_instance_id
            .and_then(crate::text::font::resolve_font_instance_handle)
            .and_then(|instance| font_database.font_instance(instance))
            .or_else(|| font_database.default_font_instance(face_id).ok())?;
        if instance.face != face_id {
            return None;
        }
        let variations = font_database
            .effective_instance_variations(
                face_id,
                key.font_instance_id
                    .and_then(crate::text::font::resolve_font_instance_handle),
                key.font_weight,
            )
            .ok()?;
        let identity = SdfOfflineArtifactIdentity {
            asset_guid: asset_uuid.to_string(),
            face_index: manifest.face_index,
            variation_hash: sdf_variation_hash(&variations),
            source_hash: sdf_font_source_hash(face_bytes.as_ref()),
            params: key.bake_params,
        };
        let project = asset_manager.current_project_manager()?;
        let artifact = if let Some(artifact) = self.artifacts.get(&identity) {
            Arc::clone(artifact)
        } else {
            let path = sdf_offline_artifact_path(project.paths().cache_root(), &identity);
            let bytes = std::fs::read(path).ok()?;
            let artifact = SdfOfflineArtifact::decode(&bytes).ok()?;
            artifact.validate_identity(&identity).ok()?;
            let artifact = Arc::new(artifact);
            self.artifacts.insert(identity, Arc::clone(&artifact));
            artifact
        };
        let face = Face::parse(face_bytes.as_ref(), 0).ok()?;
        let glyph_id = glyph_id_for_key(&face, key, face_id, font_database).ok()?.0;
        let glyph = artifact.glyph(u32::from(glyph_id))?;
        let bitmap = artifact.glyph_pixels(u32::from(glyph_id))?;
        let visible = bitmap.iter().any(|sample| *sample != 0);
        Some(RawBakedGlyph {
            metrics: SdfGlyphMetrics {
                bitmap_width: glyph.rect.width,
                bitmap_height: glyph.rect.height,
                bitmap_left: glyph.metrics.bitmap_left,
                bitmap_bottom: glyph.metrics.bitmap_bottom,
                advance: glyph.metrics.advance,
                ascent: glyph.metrics.ascent,
            },
            bitmap,
            visible,
            generation_error: None,
            source: RawBakedGlyphSource::Offline,
        })
    }
}
