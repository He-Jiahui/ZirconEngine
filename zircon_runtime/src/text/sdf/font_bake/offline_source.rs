use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::text::font::{load_text_font_source, FontDatabase, LoadedTextFontSource};
use crate::text::sdf::{
    sdf_font_source_hash, sdf_offline_artifact_path, sdf_variation_hash, SdfOfflineArtifact,
    SdfOfflineArtifactIdentity,
};
use crate::text::FontFaceId;

use super::distance_field::glyph_id_for_key;
use super::{RawBakedGlyph, RawBakedGlyphSource, SdfAtlasGlyphKey, SdfGlyphMetrics};
#[derive(Default)]
pub(super) struct SdfOfflineSourceCache {
    manifests: HashMap<String, Option<LoadedTextFontSource>>,
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
        let manifest = self.load_manifest_cached(font_ref, asset_manager)?;
        let asset_uuid = manifest.asset_uuid?;
        let face_bytes = font_database.standalone_face_bytes(face_id).ok()?;
        let instance_id = key
            .font_instance_id
            .and_then(crate::text::font::resolve_font_instance_handle);
        let instance = instance_id
            .and_then(|instance| font_database.font_instance(instance))
            .or_else(|| font_database.default_font_instance(face_id).ok())?;
        if instance.face != face_id {
            return None;
        }
        let variations = font_database
            .effective_instance_variations_shared(face_id, instance_id, key.font_weight)
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
        let glyph_id = u32::from(glyph_id_for_key(key, face_id, font_database).ok()?);
        let glyph = artifact.glyph(glyph_id)?;
        let bitmap = artifact.glyph_pixels(glyph_id)?;
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

    fn load_manifest_cached(
        &mut self,
        font_ref: &str,
        asset_manager: &ProjectAssetManager,
    ) -> Option<LoadedTextFontSource> {
        if let Some(manifest) = self.manifests.get(font_ref) {
            return manifest.clone();
        }
        let manifest = load_text_font_source(font_ref, Some(asset_manager));
        self.manifests.insert(font_ref.to_owned(), manifest.clone());
        manifest
    }

    #[cfg(test)]
    pub(super) fn load_manifest_for_test(
        &mut self,
        font_ref: &str,
        asset_manager: &ProjectAssetManager,
    ) -> Option<LoadedTextFontSource> {
        self.load_manifest_cached(font_ref, asset_manager)
    }

    #[cfg(test)]
    pub(super) fn manifest_cache_len(&self) -> usize {
        self.manifests.len()
    }
}
