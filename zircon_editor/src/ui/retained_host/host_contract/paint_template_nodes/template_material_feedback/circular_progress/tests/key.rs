use super::*;

#[test]
fn circular_progress_cache_key_wire_format_is_stable() {
    let key = circular_progress_image_key(32, 0.5, [1, 2, 3, 4], [5, 6, 7, 8]);

    assert_eq!(
        key,
        "mui-circular-progress:32:42000000:3f000000:01020304:05060708"
    );
    assert_eq!(key.len(), key.capacity());
}

#[test]
fn circular_progress_key_includes_complete_color_and_progress_identity() {
    let baseline = circular_progress_image_key(32, 0.58, [10, 20, 30, 40], [50, 60, 70, 80]);

    assert_ne!(
        baseline,
        circular_progress_image_key(32, 0.58, [10, 21, 30, 40], [50, 60, 70, 80])
    );
    assert_ne!(
        baseline,
        circular_progress_image_key(32, 0.58, [10, 20, 30, 40], [50, 61, 70, 80])
    );
    assert_ne!(
        baseline,
        circular_progress_image_key(32, 0.59, [10, 20, 30, 40], [50, 60, 70, 80])
    );
    assert_ne!(
        circular_progress_image_key_for_target(32, 31.25, 0.58, [10, 20, 30, 40], [50, 60, 70, 80]),
        circular_progress_image_key_for_target(32, 31.5, 0.58, [10, 20, 30, 40], [50, 60, 70, 80])
    );
}

#[test]
fn circular_progress_entry_keys_the_resolved_raster_percent() {
    let production = include_str!("../entry.rs");

    assert!(production.contains("circular_progress_image_key(size, progress, track, fill)"));
}
