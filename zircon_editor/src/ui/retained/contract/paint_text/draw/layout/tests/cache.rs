use super::*;

fn cache_properties(width_bits: u32) -> PaintTextLayoutCacheProperties {
    PaintTextLayoutCacheProperties {
        rect_width_bits: width_bits,
        rect_height_bits: 20.0_f32.to_bits(),
        font_size_bits: 13.0_f32.to_bits(),
        line_height_bits: 16.0_f32.to_bits(),
        font_collection_generation: 1,
        font_request: HostTextFontRequest {
            face: HostTextFontFace::Ui,
            family: "system-ui".to_owned(),
            weight: 400,
        },
        smoothing: HostTextSmoothing::Grayscale,
        word_wrap: false,
        display_only: false,
    }
}

fn layout(text: &str) -> Arc<PaintTextLayout> {
    Arc::new(PaintTextLayout {
        display_text: text.to_string(),
        glyphs: Vec::new(),
        artifact_raster_faces: Vec::new(),
        measured_lines: Vec::new(),
    })
}

#[test]
fn borrowed_text_lookup_reuses_the_inserted_layout() {
    let mut cache = IndexMap::new();
    let properties = cache_properties(0);
    let expected = layout("Preview");
    cache.insert(
        PaintTextLayoutCacheKey {
            text: "Preview".to_string(),
            properties: properties.clone(),
        },
        Arc::clone(&expected),
    );

    let actual = cache
        .get(&PaintTextLayoutCacheLookup {
            text: "Preview",
            properties: &properties,
        })
        .expect("cached layout");
    assert!(Arc::ptr_eq(actual, &expected));
}

#[test]
fn eviction_removes_one_layout_instead_of_clearing_the_cache() {
    let mut cache = IndexMap::new();
    for index in 0..2 {
        cache.insert(
            PaintTextLayoutCacheKey {
                text: format!("Layout {index}"),
                properties: cache_properties(index),
            },
            layout("Layout"),
        );
    }
    let _ = cache.shift_remove_index(0);
    assert_eq!(cache.len(), 1);
}

#[test]
fn exact_font_request_and_collection_generation_are_cache_identity() {
    let base = cache_properties(0);
    let mut different_generation = base.clone();
    different_generation.font_collection_generation += 1;
    let mut different_family = base.clone();
    different_family.font_request.family = "monospace".to_owned();
    let mut different_weight = base.clone();
    different_weight.font_request.weight += 1;

    assert_ne!(base, different_generation);
    assert_ne!(base, different_family);
    assert_ne!(base, different_weight);
}

#[test]
fn hits_refresh_recency_and_eviction_preserves_recent_layouts() {
    let mut cache = IndexMap::new();
    let properties = cache_properties(0);
    for text in ["old", "recent", "new"] {
        cache.insert(
            PaintTextLayoutCacheKey {
                text: text.into(),
                properties: properties.clone(),
            },
            layout(text),
        );
    }
    let lookup = |text| PaintTextLayoutCacheLookup {
        text,
        properties: &properties,
    };
    assert!(touch_layout(&mut cache, &lookup("old")).is_some());
    assert_eq!(cache.shift_remove_index(0).unwrap().0.text, "recent");
    cache.insert(
        PaintTextLayoutCacheKey {
            text: "newest".into(),
            properties: properties.clone(),
        },
        layout("newest"),
    );
    assert_eq!(cache.shift_remove_index(0).unwrap().0.text, "new");
    assert!(cache.get(&lookup("old")).is_some());
}

#[test]
fn scrolling_reuses_origin_relative_layout_without_rebuilding() {
    let build = |rect: &FrameRect| {
        assert_eq!((rect.x, rect.y), (0.0, 0.0));
        PaintTextLayout {
            display_text: "origin-relative-cache-regression".into(),
            glyphs: Vec::new(),
            artifact_raster_faces: Vec::new(),
            measured_lines: Vec::new(),
        }
    };
    let first = FrameRect {
        x: 4.25,
        y: 10.0,
        width: 101.0,
        height: 21.0,
    };
    let second = FrameRect {
        x: 300.75,
        y: -300.0,
        ..first.clone()
    };
    let a = cached_paint_text_layout(
        &first,
        "origin-relative-cache-regression",
        13.0,
        16.0,
        HostTextFontFace::Ui,
        HostTextSmoothing::Grayscale,
        HostTextLayoutPolicy::SingleLineEllipsis,
        false,
        build,
    );
    let b = cached_paint_text_layout(
        &second,
        "origin-relative-cache-regression",
        13.0,
        16.0,
        HostTextFontFace::Ui,
        HostTextSmoothing::Grayscale,
        HostTextLayoutPolicy::SingleLineEllipsis,
        false,
        |_| panic!("scrolling must hit"),
    );
    assert!(Arc::ptr_eq(&a, &b));
}
