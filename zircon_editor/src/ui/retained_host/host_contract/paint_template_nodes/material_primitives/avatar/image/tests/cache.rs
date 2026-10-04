use super::{AvatarMaskCache, AvatarMaskCacheKey, MAX_AVATAR_MASK_CACHE_ENTRIES};
use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::HostPaintImagePixels;

fn image(resource_key: &str) -> HostPaintImagePixels {
    HostPaintImagePixels {
        resource_key: resource_key.to_string(),
        width: 1,
        height: 1,
        rgba: vec![255; 4].into(),
        atlas: None,
    }
}

#[test]
fn cache_evicts_the_least_recently_used_mask_variant() {
    let mut cache = AvatarMaskCache::default();
    for index in 0..MAX_AVATAR_MASK_CACHE_ENTRIES {
        let image = image(format!("avatar-{index:03}").as_str());
        cache.insert(AvatarMaskCacheKey::new(&image, 1.0), image);
    }
    let oldest = AvatarMaskCacheKey::new(&image("avatar-000"), 1.0);
    assert!(cache.get(&oldest).is_some());

    let newest = image("avatar-new");
    let newest_key = AvatarMaskCacheKey::new(&newest, 1.0);
    cache.insert(newest_key.clone(), newest);

    assert_eq!(cache.entries.len(), MAX_AVATAR_MASK_CACHE_ENTRIES);
    assert!(cache.get(&oldest).is_some());
    assert!(cache
        .get(&AvatarMaskCacheKey::new(&image("avatar-001"), 1.0))
        .is_none());
    assert!(cache.get(&newest_key).is_some());
}
