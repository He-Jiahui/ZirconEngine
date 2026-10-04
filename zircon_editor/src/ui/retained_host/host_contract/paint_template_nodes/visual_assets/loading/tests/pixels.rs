use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use super::{
    icon_raster_resource_key, load_pixels_from_candidates, load_pixels_from_candidates_with_status,
    CandidatePixelsLoad,
};

#[test]
fn icon_raster_identity_can_preserve_the_content_addressed_pixel_key() {
    assert_eq!(
        icon_raster_resource_key("icon:save", "retained-image:16x16:abcd").as_deref(),
        Some("icon-raster:retained-image:16x16:abcd")
    );
    assert!(icon_raster_resource_key("image:preview", "retained-image:16x16:abcd").is_none());
}

#[test]
fn warm_pixel_cache_skips_candidate_path_construction() {
    static NEXT_KEY: AtomicU64 = AtomicU64::new(1);
    let base_key = format!(
        "test:warm-candidate-cache:{}",
        NEXT_KEY.fetch_add(1, Ordering::Relaxed)
    );
    let candidate_builds = AtomicUsize::new(0);

    assert!(load_pixels_from_candidates(
        || {
            candidate_builds.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        },
        &base_key,
        None,
        None,
        None,
    )
    .is_none());
    assert!(load_pixels_from_candidates(
        || {
            candidate_builds.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        },
        &base_key,
        None,
        None,
        None,
    )
    .is_none());

    assert_eq!(candidate_builds.load(Ordering::Relaxed), 1);
}

#[test]
fn cached_missing_candidate_is_distinct_from_a_deferred_load() {
    static NEXT_KEY: AtomicU64 = AtomicU64::new(1);
    let base_key = format!(
        "test:missing-candidate-status:{}",
        NEXT_KEY.fetch_add(1, Ordering::Relaxed)
    );

    assert!(matches!(
        load_pixels_from_candidates_with_status(Vec::new, &base_key, None, None, None),
        CandidatePixelsLoad::Missing
    ));
    assert!(matches!(
        load_pixels_from_candidates_with_status(
            || panic!("cached missing entry must skip candidate construction"),
            &base_key,
            None,
            None,
            None,
        ),
        CandidatePixelsLoad::Missing
    ));
}
