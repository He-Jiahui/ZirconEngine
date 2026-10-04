use super::{
    transform_or_rotate_rotation, vertical_glyph_metrics, vertical_shape_orientation,
    VerticalShapeOrientation,
};
use crate::text::{ShapedGlyphRotation, VerticalMode};

#[test]
fn text_vertical_cjk_upright_uses_synthesized_em_advance() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "本", 18.0, 20.0, None);

    assert_eq!(metrics.rotation, ShapedGlyphRotation::None);
    assert_eq!(metrics.advance, 20.0);
    assert_eq!(metrics.offset_x, 1.0);
}

#[test]
fn text_vertical_latin_sideways_preserves_horizontal_advance() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "A", 11.0, 20.0, Some(32.0));

    assert_eq!(metrics.rotation, ShapedGlyphRotation::Cw90);
    assert_eq!(metrics.advance, 11.0);
    assert_eq!(metrics.offset_x, 0.0);
}

#[test]
fn text_vertical_punctuation_is_upright_and_centered() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "。", 8.0, 20.0, None);

    assert_eq!(metrics.rotation, ShapedGlyphRotation::None);
    assert_eq!(metrics.advance, 20.0);
    assert_eq!(metrics.offset_x, 6.0);
}

#[test]
fn text_vertical_modes_override_unicode_mixed_orientation() {
    let upright = vertical_glyph_metrics(VerticalMode::Upright, "A", 11.0, 20.0, None);
    let sideways = vertical_glyph_metrics(VerticalMode::Sideways, "本", 18.0, 20.0, Some(32.0));

    assert_eq!(upright.rotation, ShapedGlyphRotation::None);
    assert_eq!(upright.advance, 20.0);
    assert_eq!(sideways.rotation, ShapedGlyphRotation::Cw90);
    assert_eq!(sideways.advance, 18.0);
}

#[test]
fn text_vertical_upright_prefers_native_vmtx_advance() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "本", 18.0, 20.0, Some(24.5));

    assert_eq!(metrics.rotation, ShapedGlyphRotation::None);
    assert_eq!(metrics.advance, 24.5);
    assert_eq!(metrics.offset_x, 3.25);
}

#[test]
fn text_vertical_upright_offset_stays_finite_for_extreme_horizontal_advance() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "本", f32::MAX, 20.0, None);

    assert!(metrics.offset_x.is_finite());
    assert_eq!(metrics.offset_x, -f32::MAX * 0.5);
}

#[test]
fn transformed_or_rotated_prefers_vertical_substitution_before_rotation() {
    assert_eq!(
        vertical_shape_orientation(VerticalMode::Mixed, "（"),
        VerticalShapeOrientation::TransformOrRotate
    );
    assert_eq!(
        transform_or_rotate_rotation(true),
        ShapedGlyphRotation::None
    );
    assert_eq!(
        transform_or_rotate_rotation(false),
        ShapedGlyphRotation::Cw90
    );
}
