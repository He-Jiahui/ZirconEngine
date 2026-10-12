use super::super::super::RasterTargetSize;
use super::image_pixels_cache_key;

#[test]
fn image_cache_key_wire_format_is_stable() {
    let sized = image_pixels_cache_key(
        "icon:save",
        RasterTargetSize::new(16, 24),
        Some([1, 2, 3, 4]),
    );
    let intrinsic = image_pixels_cache_key("image:preview", None, None);

    assert_eq!(sized, "icon:save:16x24:tint:01020304");
    assert_eq!(sized.len(), sized.capacity());
    assert_eq!(intrinsic, "image:preview:intrinsic:tint:none");
    assert_eq!(intrinsic.len(), intrinsic.capacity());
}

#[test]
fn raster_cache_key_separates_size_and_tint_without_a_candidate_path() {
    let small = image_pixels_cache_key(
        "icon:save",
        RasterTargetSize::new(16, 16),
        Some([1, 2, 3, 4]),
    );
    let large = image_pixels_cache_key(
        "icon:save",
        RasterTargetSize::new(24, 24),
        Some([1, 2, 3, 4]),
    );
    let recolored = image_pixels_cache_key(
        "icon:save",
        RasterTargetSize::new(16, 16),
        Some([4, 3, 2, 1]),
    );

    assert_ne!(small, large);
    assert_ne!(small, recolored);
    assert!(!small.contains(".svg"));
    assert!(!small.contains("generation"));
}
