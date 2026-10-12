use super::{icon_source_is_vector, source_is_svg, vector_cache_target, RasterTargetSize};

#[test]
fn svg_source_detection_accepts_resource_suffixes_without_classifying_bitmaps() {
    assert!(source_is_svg("res://icons/save.SVG#theme=dark"));
    assert!(source_is_svg("asset://icons/save.svg?generation=7"));
    assert!(!source_is_svg("asset://icons/save.png"));
    assert!(!source_is_svg("asset://icons/svg-preview.png"));
}

#[test]
fn semantic_icons_are_vector_but_explicit_bitmap_icons_remain_exact() {
    assert!(icon_source_is_vector("folder-open-outline"));
    assert!(icon_source_is_vector("toolbar/save.svg?theme=dark"));
    assert!(!icon_source_is_vector("thumbnails/save.PNG#g=7"));
    assert!(!icon_source_is_vector(""));
}

#[test]
fn small_vector_and_bitmap_targets_preserve_the_exact_device_pixel_extent() {
    let target = RasterTargetSize::new(17, 19);

    assert_eq!(vector_cache_target(target, false), target);
    assert_eq!(vector_cache_target(target, true), target);
}

#[test]
fn non_square_vector_targets_preserve_exact_device_pixel_geometry() {
    let target = RasterTargetSize::new(41, 43);

    assert_eq!(vector_cache_target(target, true), target);
}
