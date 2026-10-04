use std::path::Path;

use super::FontFaceMetadata;

#[test]
fn face_metadata_projects_glyph_ids_with_coverage_in_one_build() {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf"),
    )
    .unwrap();
    let metadata = FontFaceMetadata::from_sfnt_bytes(&bytes, 0);

    for codepoint in ['A', 'e', '\u{00e9}'] {
        assert!(metadata.coverage().contains(codepoint));
        assert!(metadata.glyph_id(codepoint).is_some());
    }
    assert_eq!(metadata.glyph_id('\u{10ffff}'), None);
}

#[test]
fn face_metadata_reuses_the_sorted_glyph_map_for_coverage() {
    let source = include_str!("../face_metadata.rs");
    let copied_codepoints = ["glyph_map", "codepoints()"].join(".");

    assert!(
        source.contains("let coverage = glyph_map.coverage();"),
        "face metadata must build coverage from its already sorted glyph map"
    );
    assert!(
        !source.contains(&copied_codepoints),
        "face metadata must not copy and re-sort codepoints after glyph-map construction"
    );
}
