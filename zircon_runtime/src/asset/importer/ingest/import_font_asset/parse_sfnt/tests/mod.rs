mod fixtures;

use ttf2woff2::{encode, BrotliQuality};
use ttf_parser::Face;

use super::*;
use crate::asset::assets::{decode_font_source, DecodedFontSource};

use fixtures::{fira_regular, patch_os2_weight, ttc_from_fonts, variable_font};

fn decoded(bytes: Vec<u8>) -> DecodedFontSource {
    decode_font_source(bytes).expect("font fixture should decode")
}

#[test]
fn text_font_cmap_coverage_bitset_matches_face() {
    let bytes = fira_regular();
    let source = decoded(bytes.clone());
    let metadata = parse_font_metadata(&source).unwrap();
    let face = Face::parse(&bytes, 0).unwrap();
    let coverage = &metadata.faces[0].cmap;

    assert!(coverage.codepoint_count > 0);
    for ch in ['A', 'a', '0'] {
        assert_eq!(
            coverage.contains_codepoint(ch as u32),
            face.glyph_index(ch).is_some()
        );
    }
}

#[test]
fn text_font_static_face_reports_no_variable_axes() {
    let metadata = parse_font_metadata(&decoded(fira_regular())).unwrap();

    assert_eq!(metadata.face_count, 1);
    assert!(metadata.faces[0].variation_axes.is_empty());
    assert!(metadata.faces[0].named_instances.is_empty());
}

#[test]
fn text_font_parse_ttf_extracts_os2_name_metadata() {
    let original = fira_regular();
    let expected = Face::parse(&original, 0).unwrap();
    let metadata = parse_font_metadata(&decoded(original.clone())).unwrap();
    let face = &metadata.faces[0];

    assert_eq!(metadata.source_format, FontAssetSourceFormat::Sfnt);
    assert_eq!(face.face_index, 0);
    assert!(face
        .family
        .as_deref()
        .is_some_and(|family| family.contains("Fira")));
    assert_eq!(face.weight, 400);
    assert_eq!(face.width_class, 5);
    assert_eq!(face.style, FontAssetFaceStyle::Normal);
    assert_eq!(face.metrics.units_per_em, expected.units_per_em());
    assert_eq!(face.metrics.ascender, expected.ascender());
    assert_eq!(face.metrics.descender, expected.descender());
    assert_eq!(face.metrics.line_gap, expected.line_gap());
    assert_eq!(
        face.metrics.underline.map(|metrics| metrics.position),
        expected.underline_metrics().map(|metrics| metrics.position)
    );
    assert_eq!(
        face.metrics.strikeout.map(|metrics| metrics.thickness),
        expected
            .strikeout_metrics()
            .map(|metrics| metrics.thickness)
    );
}

#[test]
fn text_font_variable_axes_roundtrip() {
    let source = decoded(variable_font());
    let metadata = parse_font_metadata(&source).unwrap();
    let face = &metadata.faces[0];

    assert_eq!(face.variation_axes.len(), 1);
    let axis = &face.variation_axes[0];
    assert_eq!(axis.tag, "wght");
    assert_eq!((axis.min, axis.default, axis.max), (100.0, 400.0, 900.0));
    assert_eq!(face.named_instances.len(), 1);
    assert_eq!(face.named_instances[0].coordinates.len(), 1);
    assert_eq!(face.named_instances[0].coordinates[0].tag, "wght");
    assert_eq!(face.named_instances[0].coordinates[0].value, 650.0);
}

#[test]
fn text_font_parse_ttc_enumerates_faces() {
    let regular = fira_regular();
    let mut second_face = regular.clone();
    patch_os2_weight(&mut second_face, 700);
    let collection = ttc_from_fonts(&[regular.as_slice(), second_face.as_slice()]);

    let metadata = parse_font_metadata(&decoded(collection)).unwrap();

    assert_eq!(
        metadata.source_format,
        FontAssetSourceFormat::TrueTypeCollection
    );
    assert_eq!(metadata.face_count, 2);
    assert_eq!(metadata.faces[0].face_index, 0);
    assert_eq!(metadata.faces[1].face_index, 1);
    assert_eq!(metadata.faces[0].weight, 400);
    assert_eq!(metadata.faces[1].weight, 700);
}

#[test]
fn text_font_woff2_decodes_to_sfnt() {
    let original = fira_regular();
    let woff2 = encode(&original, BrotliQuality::default()).unwrap();
    assert!(woff2.starts_with(b"wOF2"));

    let source = decoded(woff2);
    assert_eq!(source.source_format(), FontAssetSourceFormat::Woff2);
    assert!(!source.bytes().starts_with(b"wOF2"));

    let metadata = parse_font_metadata(&source).unwrap();
    assert_eq!(metadata.source_format, FontAssetSourceFormat::Woff2);
    assert_eq!(metadata.face_count, 1);
    assert!(metadata.faces[0]
        .family
        .as_deref()
        .is_some_and(|family| family.contains("Fira")));
    assert!(Face::parse(source.bytes(), 0).is_ok());
}

#[test]
fn text_font_malformed_woff2_preserves_decode_failure() {
    let error = decode_font_source(b"wOF2invalid".to_vec()).unwrap_err();

    assert!(error
        .to_string()
        .contains("WOFF2 font source decode failed"));
    assert!(std::error::Error::source(&error).is_some());
}
