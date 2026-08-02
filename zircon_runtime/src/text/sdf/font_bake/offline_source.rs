use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::text::font::{load_text_font_source, FontDatabase, LoadedTextFontSource};
use crate::text::sdf::{
    sdf_offline_artifact_path, SdfGenerationSourceContext, SdfOfflineArtifact,
    SdfOfflineArtifactIdentity,
};
use crate::text::FontFaceId;

use super::distance_field::glyph_id_for_key;
use super::{RawBakedGlyph, RawBakedGlyphSource, SdfAtlasGlyphKey, SdfGlyphMetrics};

const MAX_RESIDENT_MANIFEST_COUNT: usize = 128;
const MAX_RESIDENT_ARTIFACT_IDENTITY_COUNT: usize = 32;
const MAX_RESIDENT_ARTIFACT_BYTE_COUNT: usize = 128 * 1024 * 1024;
const MAX_RESIDENT_GLYPH_BITMAP_COUNT: usize = 4 * 1024;
const MAX_RESIDENT_GLYPH_BITMAP_BYTE_COUNT: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SdfOfflineGlyphKey {
    artifact: SdfOfflineArtifactIdentity,
    glyph_id: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfOfflineSourceCacheReport {
    pub(super) resident_manifest_count: usize,
    pub(super) resident_artifact_identity_count: usize,
    pub(super) resident_artifact_byte_count: usize,
    pub(super) resident_glyph_bitmap_count: usize,
    pub(super) resident_glyph_bitmap_byte_count: usize,
    pub(super) manifest_parse_count: usize,
    pub(super) artifact_stat_count: usize,
    pub(super) artifact_read_count: usize,
    pub(super) artifact_read_byte_count: usize,
    pub(super) artifact_decode_count: usize,
    pub(super) pixel_copy_count: usize,
    pub(super) pixel_copy_byte_count: usize,
    pub(super) manifest_eviction_count: usize,
    pub(super) artifact_eviction_count: usize,
    pub(super) glyph_bitmap_eviction_count: usize,
    pub(super) oldest_artifact_idle_access_count: u64,
    pub(super) oldest_glyph_bitmap_idle_access_count: u64,
}

#[derive(Default)]
pub(super) struct SdfOfflineSourceCache {
    manifests: HashMap<String, Option<LoadedTextFontSource>>,
    artifacts: HashMap<SdfOfflineArtifactIdentity, Option<Arc<SdfOfflineArtifact>>>,
    glyph_bitmaps: HashMap<SdfOfflineGlyphKey, Arc<[u8]>>,
    manifest_recency: HashMap<String, u64>,
    artifact_recency: HashMap<SdfOfflineArtifactIdentity, u64>,
    glyph_bitmap_recency: HashMap<SdfOfflineGlyphKey, u64>,
    access_epoch: u64,
    resident_artifact_byte_count: usize,
    resident_glyph_bitmap_byte_count: usize,
    manifest_parse_count: usize,
    artifact_stat_count: usize,
    artifact_read_count: usize,
    artifact_read_byte_count: usize,
    artifact_decode_count: usize,
    pixel_copy_count: usize,
    pixel_copy_byte_count: usize,
    manifest_eviction_count: usize,
    artifact_eviction_count: usize,
    glyph_bitmap_eviction_count: usize,
}

impl SdfOfflineSourceCacheReport {
    pub(super) fn delta_since(self, previous: Self) -> Self {
        Self {
            resident_manifest_count: self.resident_manifest_count,
            resident_artifact_identity_count: self.resident_artifact_identity_count,
            resident_artifact_byte_count: self.resident_artifact_byte_count,
            resident_glyph_bitmap_count: self.resident_glyph_bitmap_count,
            resident_glyph_bitmap_byte_count: self.resident_glyph_bitmap_byte_count,
            manifest_parse_count: self
                .manifest_parse_count
                .saturating_sub(previous.manifest_parse_count),
            artifact_stat_count: self
                .artifact_stat_count
                .saturating_sub(previous.artifact_stat_count),
            artifact_read_count: self
                .artifact_read_count
                .saturating_sub(previous.artifact_read_count),
            artifact_read_byte_count: self
                .artifact_read_byte_count
                .saturating_sub(previous.artifact_read_byte_count),
            artifact_decode_count: self
                .artifact_decode_count
                .saturating_sub(previous.artifact_decode_count),
            pixel_copy_count: self
                .pixel_copy_count
                .saturating_sub(previous.pixel_copy_count),
            pixel_copy_byte_count: self
                .pixel_copy_byte_count
                .saturating_sub(previous.pixel_copy_byte_count),
            manifest_eviction_count: self
                .manifest_eviction_count
                .saturating_sub(previous.manifest_eviction_count),
            artifact_eviction_count: self
                .artifact_eviction_count
                .saturating_sub(previous.artifact_eviction_count),
            glyph_bitmap_eviction_count: self
                .glyph_bitmap_eviction_count
                .saturating_sub(previous.glyph_bitmap_eviction_count),
            oldest_artifact_idle_access_count: self.oldest_artifact_idle_access_count,
            oldest_glyph_bitmap_idle_access_count: self.oldest_glyph_bitmap_idle_access_count,
        }
    }
}

impl SdfOfflineSourceCache {
    pub(super) fn load_glyph(
        &mut self,
        key: &SdfAtlasGlyphKey,
        face_id: FontFaceId,
        resolved_shaped_face: Option<FontFaceId>,
        source: &SdfGenerationSourceContext,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Option<RawBakedGlyph> {
        let font_ref = key.font.as_deref()?;
        let manifest = self.load_manifest_cached(font_ref, asset_manager)?;
        let asset_uuid = manifest.asset_uuid?;
        let identity = SdfOfflineArtifactIdentity {
            asset_guid: asset_uuid.to_string(),
            face_index: manifest.face_index,
            variation_hash: source.variation_hash(),
            source_hash: source.source_hash(),
            params: key.bake_params,
        };
        let project = asset_manager.current_project_manager()?;
        let artifact = self.load_artifact_cached(project.paths().cache_root(), &identity)?;
        let glyph_id =
            u32::from(glyph_id_for_key(key, face_id, resolved_shaped_face, font_database).ok()?);
        let glyph = artifact.glyph(glyph_id)?;
        let bitmap_key = SdfOfflineGlyphKey {
            artifact: identity,
            glyph_id,
        };
        let bitmap = if let Some(bitmap) = self.glyph_bitmaps.get(&bitmap_key).cloned() {
            self.touch_glyph_bitmap(bitmap_key.clone());
            bitmap
        } else {
            let bitmap: Arc<[u8]> = artifact.glyph_pixels(glyph_id)?.into();
            self.pixel_copy_count = self.pixel_copy_count.saturating_add(1);
            self.pixel_copy_byte_count = self.pixel_copy_byte_count.saturating_add(bitmap.len());
            self.glyph_bitmaps
                .insert(bitmap_key.clone(), Arc::clone(&bitmap));
            self.resident_glyph_bitmap_byte_count = self
                .resident_glyph_bitmap_byte_count
                .saturating_add(bitmap.len());
            self.touch_glyph_bitmap(bitmap_key);
            self.enforce_glyph_bitmap_budget();
            bitmap
        };
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
        if let Some(manifest) = self.manifests.get(font_ref).cloned() {
            self.touch_manifest(font_ref.to_owned());
            return manifest;
        }
        self.manifest_parse_count = self.manifest_parse_count.saturating_add(1);
        let manifest = load_text_font_source(font_ref, Some(asset_manager));
        self.manifests.insert(font_ref.to_owned(), manifest.clone());
        self.touch_manifest(font_ref.to_owned());
        self.enforce_manifest_budget();
        manifest
    }

    fn load_artifact_cached(
        &mut self,
        cache_root: &std::path::Path,
        identity: &SdfOfflineArtifactIdentity,
    ) -> Option<Arc<SdfOfflineArtifact>> {
        if let Some(artifact) = self.artifacts.get(identity).cloned() {
            self.touch_artifact(identity.clone());
            return artifact;
        }

        let path = sdf_offline_artifact_path(cache_root, identity);
        self.artifact_stat_count = self.artifact_stat_count.saturating_add(1);
        let artifact = std::fs::metadata(&path).ok().and_then(|_| {
            self.artifact_read_count = self.artifact_read_count.saturating_add(1);
            let bytes = std::fs::read(&path).ok()?;
            self.artifact_read_byte_count =
                self.artifact_read_byte_count.saturating_add(bytes.len());
            self.artifact_decode_count = self.artifact_decode_count.saturating_add(1);
            let artifact = SdfOfflineArtifact::decode(&bytes).ok()?;
            artifact.validate_identity(identity).ok()?;
            Some(Arc::new(artifact))
        });
        self.resident_artifact_byte_count = self.resident_artifact_byte_count.saturating_add(
            artifact
                .as_ref()
                .map_or(0, |artifact| artifact_payload_byte_count(artifact)),
        );
        self.artifacts.insert(identity.clone(), artifact.clone());
        self.touch_artifact(identity.clone());
        self.enforce_artifact_budget();
        artifact
    }

    pub(super) fn report(&self) -> SdfOfflineSourceCacheReport {
        SdfOfflineSourceCacheReport {
            resident_manifest_count: self.manifests.len(),
            resident_artifact_identity_count: self.artifacts.len(),
            resident_artifact_byte_count: self.resident_artifact_byte_count,
            resident_glyph_bitmap_count: self.glyph_bitmaps.len(),
            resident_glyph_bitmap_byte_count: self.resident_glyph_bitmap_byte_count,
            manifest_parse_count: self.manifest_parse_count,
            artifact_stat_count: self.artifact_stat_count,
            artifact_read_count: self.artifact_read_count,
            artifact_read_byte_count: self.artifact_read_byte_count,
            artifact_decode_count: self.artifact_decode_count,
            pixel_copy_count: self.pixel_copy_count,
            pixel_copy_byte_count: self.pixel_copy_byte_count,
            manifest_eviction_count: self.manifest_eviction_count,
            artifact_eviction_count: self.artifact_eviction_count,
            glyph_bitmap_eviction_count: self.glyph_bitmap_eviction_count,
            oldest_artifact_idle_access_count: oldest_idle_access_count(
                self.access_epoch,
                &self.artifact_recency,
            ),
            oldest_glyph_bitmap_idle_access_count: oldest_idle_access_count(
                self.access_epoch,
                &self.glyph_bitmap_recency,
            ),
        }
    }

    fn touch_manifest(&mut self, key: String) {
        let epoch = self.next_access_epoch();
        self.manifest_recency.insert(key, epoch);
    }

    fn touch_artifact(&mut self, key: SdfOfflineArtifactIdentity) {
        let epoch = self.next_access_epoch();
        self.artifact_recency.insert(key, epoch);
    }

    fn touch_glyph_bitmap(&mut self, key: SdfOfflineGlyphKey) {
        let epoch = self.next_access_epoch();
        self.glyph_bitmap_recency.insert(key, epoch);
    }

    fn next_access_epoch(&mut self) -> u64 {
        self.access_epoch = self.access_epoch.saturating_add(1).max(1);
        self.access_epoch
    }

    fn enforce_manifest_budget(&mut self) {
        while self.manifests.len() > MAX_RESIDENT_MANIFEST_COUNT {
            let Some(victim) = oldest_key(&self.manifest_recency) else {
                break;
            };
            self.manifests.remove(&victim);
            self.manifest_recency.remove(&victim);
            self.manifest_eviction_count = self.manifest_eviction_count.saturating_add(1);
        }
    }

    fn enforce_artifact_budget(&mut self) {
        while self.artifacts.len() > MAX_RESIDENT_ARTIFACT_IDENTITY_COUNT
            || (self.artifacts.len() > 1
                && self.resident_artifact_byte_count > MAX_RESIDENT_ARTIFACT_BYTE_COUNT)
        {
            let Some(victim) = oldest_key(&self.artifact_recency) else {
                break;
            };
            if let Some(Some(artifact)) = self.artifacts.remove(&victim) {
                self.resident_artifact_byte_count = self
                    .resident_artifact_byte_count
                    .saturating_sub(artifact_payload_byte_count(&artifact));
            }
            self.artifact_recency.remove(&victim);
            self.artifact_eviction_count = self.artifact_eviction_count.saturating_add(1);
        }
    }

    fn enforce_glyph_bitmap_budget(&mut self) {
        while self.glyph_bitmaps.len() > MAX_RESIDENT_GLYPH_BITMAP_COUNT
            || (self.glyph_bitmaps.len() > 1
                && self.resident_glyph_bitmap_byte_count > MAX_RESIDENT_GLYPH_BITMAP_BYTE_COUNT)
        {
            let Some(victim) = oldest_key(&self.glyph_bitmap_recency) else {
                break;
            };
            if let Some(bitmap) = self.glyph_bitmaps.remove(&victim) {
                self.resident_glyph_bitmap_byte_count = self
                    .resident_glyph_bitmap_byte_count
                    .saturating_sub(bitmap.len());
            }
            self.glyph_bitmap_recency.remove(&victim);
            self.glyph_bitmap_eviction_count = self.glyph_bitmap_eviction_count.saturating_add(1);
        }
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

fn artifact_payload_byte_count(artifact: &SdfOfflineArtifact) -> usize {
    artifact
        .pages()
        .iter()
        .map(|page| page.pixels.len())
        .fold(0_usize, usize::saturating_add)
}

fn oldest_key<K>(recency: &HashMap<K, u64>) -> Option<K>
where
    K: Clone + Ord + std::hash::Hash + Eq,
{
    recency
        .iter()
        .min_by(|(left_key, left_epoch), (right_key, right_epoch)| {
            left_epoch
                .cmp(right_epoch)
                .then_with(|| left_key.cmp(right_key))
        })
        .map(|(key, _)| key.clone())
}

fn oldest_idle_access_count<K>(current_epoch: u64, recency: &HashMap<K, u64>) -> u64 {
    recency
        .values()
        .copied()
        .min()
        .map(|epoch| current_epoch.saturating_sub(epoch))
        .unwrap_or(0)
}
