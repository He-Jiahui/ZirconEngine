use super::*;

#[test]
fn ui_surface_text_attrs_preserve_requested_family_and_weight() {
    let attrs = text_attrs(Some("Zircon Sans"), 500, UiSurfaceTextStyle::Regular);

    assert_eq!(attrs.family, Family::Name("Zircon Sans"));
    assert_eq!(attrs.weight, Weight(500));

    let strong_attrs = text_attrs(Some("Zircon Sans"), 500, UiSurfaceTextStyle::Strong);
    assert_eq!(strong_attrs.family, Family::Name("Zircon Sans"));
    assert_eq!(strong_attrs.weight, Weight::BOLD);

    let emphasis_attrs = text_attrs(None, 450, UiSurfaceTextStyle::Emphasis);
    assert_eq!(emphasis_attrs.weight, Weight(450));
    assert_eq!(emphasis_attrs.style, Style::Italic);
}

#[test]
fn text_batch_cache_key_allows_a_versioned_damage_projection() {
    let versioned = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 9);
    let damaged = UiSurfaceDrawList::with_generation(
        (64, 32),
        Some(zr_rhi::UiSurfaceRect::new(0.0, 0.0, 8.0, 8.0)),
        Vec::new(),
        9,
    );
    let legacy = UiSurfaceDrawList::new((64, 32), None, Vec::new());

    assert!(text_batch_cache_key(&versioned, (64, 32)).is_some());
    assert!(text_batch_cache_key(&damaged, (64, 32)).is_some());
    assert_eq!(text_batch_cache_key(&legacy, (64, 32)), None);
}

#[test]
fn text_batch_cache_key_ignores_target_only_resize() {
    let mut draw_list = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 9);
    let original = text_batch_cache_key(&draw_list, draw_list.projection_size());

    draw_list.retarget_surface_size_preserving_projection((32, 16));

    assert_eq!(
        text_batch_cache_key(&draw_list, draw_list.projection_size()),
        original
    );
}

#[test]
fn text_preparation_skips_content_that_cannot_produce_visible_glyphs() {
    assert!(!text_has_visible_content(""));
    assert!(!text_has_visible_content(" \t\r\n"));
    assert!(text_has_visible_content("Zircon"));
}

#[test]
fn text_metrics_preserve_fractional_physical_sizes() {
    let metrics = text_metrics(13.333_333, 16.666_666);

    assert_eq!(metrics.font_size.to_bits(), 13.333_333_f32.to_bits());
    assert_eq!(metrics.line_height.to_bits(), 16.666_666_f32.to_bits());
}

#[test]
fn text_color_mode_matches_the_surface_transfer_function() {
    assert_eq!(
        text_color_mode(wgpu::TextureFormat::Bgra8UnormSrgb),
        glyphon::ColorMode::Accurate
    );
    assert_eq!(
        text_color_mode(wgpu::TextureFormat::Bgra8Unorm),
        glyphon::ColorMode::Web
    );
}

#[test]
fn text_prepare_failure_does_not_publish_the_generation_cache_key() {
    let cache_key = Some(TextBatchCacheKey {
        generation: 7,
        projection_size: (320, 240),
    });

    assert_eq!(committed_text_batch_cache_key(cache_key, 0), cache_key);
    assert_eq!(committed_text_batch_cache_key(cache_key, 1), None);
}
