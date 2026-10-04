use std::path::Path;

use super::*;

#[test]
fn text_font_face_match_cache_evicts_only_its_oldest_entry_at_capacity() {
    let mut cache = FaceMatchCache::default();
    let keys = (0..=MAX_FACE_MATCH_CACHE_ENTRIES)
        .map(|index| FontMatchCacheKey::from(&FontQuery::single_family(format!("Family {index}"))))
        .collect::<Vec<_>>();

    for key in keys.iter().take(MAX_FACE_MATCH_CACHE_ENTRIES) {
        cache.insert(key.clone(), None);
    }
    cache.insert(keys[MAX_FACE_MATCH_CACHE_ENTRIES].clone(), None);

    assert_eq!(cache.len(), MAX_FACE_MATCH_CACHE_ENTRIES);
    assert!(!cache.contains(&keys[0]));
    assert!(keys[1..].iter().all(|key| cache.contains(key)));
}

#[test]
fn font_asset_face_index_is_retained_as_an_owner_borrow() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/fonts/ZirconDefaultComposite-subset.ttc");
    let owner = "res://fonts/retained-owner-index.font.toml";
    let mut database = FontDatabase::default();
    let registered = database
        .replace_font_source(owner, source, Some("Retained Owner Face"), 1)
        .expect("owner face should register");

    let faces: &[FontFaceId] = database.font_asset_faces(owner);

    assert_eq!(faces, registered.faces.as_slice());
}
