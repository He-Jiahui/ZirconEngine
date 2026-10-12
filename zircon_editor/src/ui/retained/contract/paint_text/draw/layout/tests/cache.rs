use super::*;

fn cache_properties(rect_x_bits: u32) -> PaintTextLayoutCacheProperties {
    PaintTextLayoutCacheProperties {
        rect_x_bits,
        rect_y_bits: 0,
        rect_width_bits: 100.0_f32.to_bits(),
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
    let _ = cache.swap_remove_index(0);
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
