use super::ShapingTextView;

#[test]
fn source_preserving_view_keeps_decomposed_source_bytes_unchanged() {
    let source = "a\u{0304}\u{0301}b";
    let view = ShapingTextView::source_preserving(source);

    assert_eq!(view.shaping_text(), source);
    assert_eq!(view.shaping_text().len(), source.len());
}

#[test]
fn source_preserving_view_maps_shaping_offsets_to_original_source_offsets() {
    let source = "a\u{0304}\u{0301}b";
    let view = ShapingTextView::source_preserving(source);

    assert_eq!(view.source_range_for_shaping_range(1..5), 1..5);
    assert_eq!(
        view.source_range_for_shaping_range(source.len()..source.len()),
        source.len()..source.len()
    );
}

#[test]
fn source_preserving_view_keeps_canonical_equivalents_as_distinct_source_bytes() {
    let composed = "\u{00E9}";
    let decomposed = "e\u{0301}";
    let composed_view = ShapingTextView::source_preserving(composed);
    let decomposed_view = ShapingTextView::source_preserving(decomposed);

    assert_ne!(composed_view.shaping_text(), decomposed_view.shaping_text());
    assert_eq!(
        composed_view.source_range_for_shaping_range(0..composed.len()),
        0..composed.len()
    );
    assert_eq!(
        decomposed_view.source_range_for_shaping_range(0..decomposed.len()),
        0..decomposed.len()
    );
}
