use super::*;

#[test]
fn direct_presenter_keeps_capture_until_its_viewport_is_resolved() {
    let products = Arc::new(ViewportProductRegistry::default());
    let direct_viewport = RenderViewportHandle::new(7);
    let fallback_viewport = RenderViewportHandle::new(8);
    assert!(products.requires_async_capture(direct_viewport));

    let provider = WgpuViewportProductProvider::new(Arc::clone(&products));
    assert!(products.requires_async_capture(direct_viewport));
    products.mark_direct_viewport_for_test(direct_viewport);
    assert!(!products.requires_async_capture(direct_viewport));
    assert!(products.requires_async_capture(fallback_viewport));
    drop(provider);

    assert!(products.requires_async_capture(direct_viewport));
}

#[test]
fn direct_consumers_are_reference_counted_per_presenter() {
    let products = Arc::new(ViewportProductRegistry::default());
    let viewport = RenderViewportHandle::new(7);
    let first = WgpuViewportProductProvider::new(Arc::clone(&products));
    let second = WgpuViewportProductProvider::new(Arc::clone(&products));

    first.confirm_viewport_for_test(viewport);
    second.confirm_viewport_for_test(viewport);
    assert!(!products.requires_async_capture(viewport));
    drop(first);
    assert!(!products.requires_async_capture(viewport));
    drop(second);
    assert!(products.requires_async_capture(viewport));
}

#[test]
fn product_registry_exports_independent_gpu_snapshots() {
    let source = include_str!("../viewport_product_registry.rs");

    assert!(!source.contains("copy_texture_for_external_image"));
    assert!(!source.contains("WgpuUiSurfaceContext"));
    assert!(source.contains("copy: WgpuUiExternalImageCopyReceipt"));
    assert!(source.contains("copy.submission()"));
    assert!(source
        .contains("validate_viewport_product_publication(copy.generation(), product_submission)"));
    assert!(source.contains("FrameProductPublicationFailed"));
    assert!(source.contains("image: WgpuUiExternalImage"));
    assert!(!source.contains("texture: texture,"));
    assert!(source.contains("products.by_viewport.clear()"));
    assert!(source.contains("products.by_resource_key.clear()"));
}

#[test]
fn resource_keys_keep_a_bounded_generation_ring() {
    let mut resource_keys = VecDeque::new();

    for generation in 1..=MAX_RETAINED_VIEWPORT_PRODUCT_GENERATIONS {
        assert!(
            retain_resource_key(&mut resource_keys, format!("viewport:7:{generation}"),).is_none()
        );
    }

    assert_eq!(
        retain_resource_key(&mut resource_keys, "viewport:7:4".to_string()),
        Some("viewport:7:1".to_string())
    );
    assert_eq!(
        resource_keys.into_iter().collect::<Vec<_>>(),
        vec!["viewport:7:2", "viewport:7:3", "viewport:7:4"]
    );
}
