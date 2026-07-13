use super::*;

use std::path::Path;

use crate::core::framework::render::{FontFaceId, VariationCoords};

const WGHT: u32 = u32::from_be_bytes(*b"wght");
const WDTH: u32 = u32::from_be_bytes(*b"wdth");

#[test]
fn text_font_instance_roundtrips_canonical_face_and_coordinates() {
    let mut registry = FontInstanceRegistry::default();
    let face = FontFaceId(7);
    let requested = VariationCoords(vec![(WGHT, 650.0), (WDTH, 90.0)]);

    let id = registry.resolve_or_insert(face, &requested).unwrap();
    let instance = registry.get(id).expect("registered font instance");

    assert_eq!(instance.face, face);
    assert_eq!(
        instance.variations,
        VariationCoords(vec![(WDTH, 90.0), (WGHT, 650.0)])
    );
}

#[test]
fn text_font_instance_identity_normalizes_order_duplicates_and_negative_zero() {
    let mut registry = FontInstanceRegistry::default();
    let face = FontFaceId(11);
    let noisy = VariationCoords(vec![(WGHT, 500.0), (WDTH, -0.0), (WGHT, 650.0)]);
    let canonical = VariationCoords(vec![(WDTH, 0.0), (WGHT, 650.0)]);

    assert_eq!(
        registry.resolve_or_insert(face, &noisy).unwrap(),
        registry.resolve_or_insert(face, &canonical).unwrap()
    );
    assert_eq!(registry.len(), 1);
}

#[test]
fn text_font_instance_rejects_non_finite_coordinate() {
    let mut registry = FontInstanceRegistry::default();
    let error = registry
        .resolve_or_insert(FontFaceId(13), &VariationCoords(vec![(WGHT, f32::NAN)]))
        .unwrap_err();

    assert_eq!(error, FontInstanceError::NonFiniteCoordinate { tag: WGHT });
    assert!(registry.is_empty());
}

#[test]
fn text_font_instance_identity_separates_faces_and_coordinates() {
    let mut registry = FontInstanceRegistry::default();
    let regular = registry
        .resolve_or_insert(FontFaceId(17), &VariationCoords(vec![(WGHT, 400.0)]))
        .unwrap();
    let bold = registry
        .resolve_or_insert(FontFaceId(17), &VariationCoords(vec![(WGHT, 700.0)]))
        .unwrap();
    let other_face = registry
        .resolve_or_insert(FontFaceId(18), &VariationCoords(vec![(WGHT, 400.0)]))
        .unwrap();

    assert_ne!(regular, bold);
    assert_ne!(regular, other_face);
}

#[test]
fn text_font_effective_variations_drop_axes_not_exposed_by_static_face() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let bytes = std::fs::read(source).expect("tracked static font fixture");
    let variations = variations_with_font_weight(
        &bytes,
        0,
        &VariationCoords(vec![(WDTH, 90.0), (WGHT, 650.0)]),
        700,
    );

    assert_eq!(variations, VariationCoords::default());
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_effective_variations_clamp_axes_and_drop_default_coordinates() {
    let bytes =
        std::fs::read(r"C:\Windows\Fonts\bahnschrift.ttf").expect("Windows variable-font fixture");
    let face = ttf_parser::Face::parse(&bytes, 0).expect("Bahnschrift face");
    let width = face
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    let weight = face
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wght"))
        .expect("Bahnschrift weight axis");
    let variations = variations_with_font_weight(
        &bytes,
        0,
        &VariationCoords(vec![(
            u32::from_be_bytes(width.tag.to_bytes()),
            width.min_value - 100.0,
        )]),
        weight.def_value as u16,
    );

    assert_eq!(
        variations,
        VariationCoords(vec![(
            u32::from_be_bytes(width.tag.to_bytes()),
            width.min_value,
        )])
    );
}
