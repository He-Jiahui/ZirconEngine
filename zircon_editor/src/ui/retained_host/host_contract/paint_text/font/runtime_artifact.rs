use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

use fontdue::Font;
use zircon_runtime::core::framework::text::TextFontFaceHandle;
use zircon_runtime::ui::surface::UiTextGlyphArtifactRasterFace;

use super::{font_settings_for_collection_index, HostTextFont, HostTextFontSnapshot};
use crate::ui::retained_host::host_contract::paint_text::sync::lock_recovering_poison;

const RUNTIME_ARTIFACT_FONT_CACHE_CAPACITY: usize = 64;
const RUNTIME_ARTIFACT_FONT_FAMILY: &str = "Zircon Runtime Artifact";
const RUNTIME_ARTIFACT_FONT_WEIGHT: u16 = 400;
const RUNTIME_ARTIFACT_FONT_CACHE_DOMAIN: &[u8] = b"zircon-retained-runtime-artifact-font-v1";

/// Builds an editor raster snapshot from the exact runtime face selected by shaping.
///
/// The cache is intentionally bounded and keyed by the runtime source identity and generation.
/// `fontdue` cannot apply OpenType variation coordinates, so non-default instances fail closed and
/// leave the caller on its complete host-layout fallback rather than painting mismatched glyphs.
pub(in crate::ui::retained_host::host_contract) fn host_runtime_artifact_font_snapshot(
    runtime_face: &UiTextGlyphArtifactRasterFace,
) -> Option<HostTextFontSnapshot> {
    if runtime_face
        .variations()
        .is_some_and(|variations| !variations.0.is_empty())
    {
        return None;
    }

    let key = RuntimeArtifactFontKey {
        source_identity: runtime_face.source_identity(),
        font_generation: runtime_face.font_generation(),
        font_face: runtime_face.font_face(),
        font_instance: runtime_face.font_instance(),
        collection_index: runtime_face.collection_index(),
    };
    if let Some(font) = cached_runtime_artifact_font(key) {
        return Some(HostTextFontSnapshot { font });
    }

    let bytes = runtime_face.bytes();
    let font = Arc::new(HostTextFont {
        font: Some(Arc::new(
            Font::from_bytes(
                Arc::clone(&bytes),
                font_settings_for_collection_index(key.collection_index),
            )
            .ok()?,
        )),
        bytes,
        runtime_family: Arc::from(RUNTIME_ARTIFACT_FONT_FAMILY),
        weight: RUNTIME_ARTIFACT_FONT_WEIGHT,
        collection_index: key.collection_index,
        cache_key: runtime_artifact_font_cache_key(key),
    });

    Some(HostTextFontSnapshot {
        font: insert_runtime_artifact_font(key, font),
    })
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct RuntimeArtifactFontKey {
    source_identity: [u8; 16],
    font_generation: u64,
    font_face: TextFontFaceHandle,
    font_instance: Option<TextFontFaceHandle>,
    collection_index: u32,
}

fn cached_runtime_artifact_font(key: RuntimeArtifactFontKey) -> Option<Arc<HostTextFont>> {
    let cache = runtime_artifact_font_cache();
    let mut cache = lock_recovering_poison(cache);
    let index = cache.iter().position(|entry| entry.key == key)?;
    let entry = cache.remove(index)?;
    let font = Arc::clone(&entry.font);
    cache.push_front(entry);
    Some(font)
}

fn insert_runtime_artifact_font(
    key: RuntimeArtifactFontKey,
    font: Arc<HostTextFont>,
) -> Arc<HostTextFont> {
    let cache = runtime_artifact_font_cache();
    let mut cache = lock_recovering_poison(cache);
    if let Some(index) = cache.iter().position(|entry| entry.key == key) {
        let entry = cache.remove(index).expect("cache entry located above");
        let existing = Arc::clone(&entry.font);
        cache.push_front(entry);
        return existing;
    }

    cache.push_front(RuntimeArtifactFontCacheEntry {
        key,
        font: Arc::clone(&font),
    });
    cache.truncate(RUNTIME_ARTIFACT_FONT_CACHE_CAPACITY);
    font
}

struct RuntimeArtifactFontCacheEntry {
    key: RuntimeArtifactFontKey,
    font: Arc<HostTextFont>,
}

fn runtime_artifact_font_cache() -> &'static Mutex<VecDeque<RuntimeArtifactFontCacheEntry>> {
    static CACHE: OnceLock<Mutex<VecDeque<RuntimeArtifactFontCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn runtime_artifact_font_cache_key(key: RuntimeArtifactFontKey) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    RUNTIME_ARTIFACT_FONT_CACHE_DOMAIN.hash(&mut hasher);
    key.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_artifact_font_cache_key_keeps_generation_and_instance_identity() {
        let base = RuntimeArtifactFontKey {
            source_identity: [7; 16],
            font_generation: 12,
            font_face: TextFontFaceHandle::new(3, 12),
            font_instance: Some(TextFontFaceHandle::new(5, 12)),
            collection_index: 1,
        };
        let stale_generation = RuntimeArtifactFontKey {
            font_generation: 13,
            font_face: TextFontFaceHandle::new(3, 13),
            font_instance: Some(TextFontFaceHandle::new(5, 13)),
            ..base
        };
        let other_instance = RuntimeArtifactFontKey {
            font_instance: Some(TextFontFaceHandle::new(6, 12)),
            ..base
        };

        assert_ne!(
            runtime_artifact_font_cache_key(base),
            runtime_artifact_font_cache_key(stale_generation)
        );
        assert_ne!(
            runtime_artifact_font_cache_key(base),
            runtime_artifact_font_cache_key(other_instance)
        );
    }
}
