use unicode_vo::{Orientation, char_orientation};

use crate::text::{ShapedGlyphRotation, VerticalMode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct VerticalGlyphMetrics {
    pub(super) rotation: ShapedGlyphRotation,
    pub(super) advance: f32,
    pub(super) offset_x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::text::shaping) enum VerticalShapeOrientation {
    Upright,
    Sideways,
    TransformOrRotate,
}

pub(super) fn vertical_glyph_metrics(
    mode: VerticalMode,
    cluster_text: &str,
    horizontal_advance: f32,
    font_size: f32,
    native_vertical_advance: Option<f32>,
) -> VerticalGlyphMetrics {
    let horizontal_advance = horizontal_advance.max(0.0);
    if horizontal_advance == 0.0 || cluster_text.chars().all(char::is_control) {
        return VerticalGlyphMetrics {
            rotation: ShapedGlyphRotation::None,
            advance: 0.0,
            offset_x: 0.0,
        };
    }

    let rotation = vertical_glyph_rotation(mode, cluster_text);
    vertical_glyph_metrics_for_rotation(
        cluster_text,
        rotation,
        horizontal_advance,
        font_size,
        native_vertical_advance,
    )
}

pub(super) fn vertical_glyph_metrics_for_rotation(
    cluster_text: &str,
    rotation: ShapedGlyphRotation,
    horizontal_advance: f32,
    font_size: f32,
    native_vertical_advance: Option<f32>,
) -> VerticalGlyphMetrics {
    let horizontal_advance = horizontal_advance.max(0.0);
    if horizontal_advance == 0.0 || cluster_text.chars().all(char::is_control) {
        return VerticalGlyphMetrics {
            rotation: ShapedGlyphRotation::None,
            advance: 0.0,
            offset_x: 0.0,
        };
    }
    if !matches!(rotation, ShapedGlyphRotation::None) {
        return VerticalGlyphMetrics {
            rotation,
            advance: horizontal_advance,
            offset_x: 0.0,
        };
    }

    let advance = native_vertical_advance
        .filter(|advance| advance.is_finite() && *advance > 0.0)
        .unwrap_or_else(|| font_size.max(1.0));
    VerticalGlyphMetrics {
        rotation: ShapedGlyphRotation::None,
        advance,
        offset_x: (advance - horizontal_advance) * 0.5,
    }
}

pub(super) fn vertical_glyph_rotation(
    mode: VerticalMode,
    cluster_text: &str,
) -> ShapedGlyphRotation {
    if cluster_text.is_empty() || cluster_text.chars().all(char::is_control) {
        return ShapedGlyphRotation::None;
    }
    match vertical_shape_orientation(mode, cluster_text) {
        VerticalShapeOrientation::Upright => ShapedGlyphRotation::None,
        VerticalShapeOrientation::Sideways | VerticalShapeOrientation::TransformOrRotate => {
            ShapedGlyphRotation::Cw90
        }
    }
}

pub(in crate::text::shaping) fn vertical_shape_orientation(
    mode: VerticalMode,
    cluster_text: &str,
) -> VerticalShapeOrientation {
    if cluster_text.is_empty() || cluster_text.chars().all(char::is_control) {
        return VerticalShapeOrientation::Upright;
    }
    match mode {
        VerticalMode::Upright => VerticalShapeOrientation::Upright,
        VerticalMode::Sideways => VerticalShapeOrientation::Sideways,
        VerticalMode::Mixed => {
            let mut transform_or_rotate = false;
            for character in cluster_text.chars() {
                match char_orientation(character) {
                    Orientation::Upright | Orientation::TransformedOrUpright => {
                        return VerticalShapeOrientation::Upright;
                    }
                    Orientation::TransformedOrRotated => transform_or_rotate = true,
                    Orientation::Rotated => {}
                }
            }
            if transform_or_rotate {
                VerticalShapeOrientation::TransformOrRotate
            } else {
                VerticalShapeOrientation::Sideways
            }
        }
    }
}

pub(super) const fn transform_or_rotate_rotation(
    vertical_substituted: bool,
) -> ShapedGlyphRotation {
    if vertical_substituted {
        ShapedGlyphRotation::None
    } else {
        ShapedGlyphRotation::Cw90
    }
}

#[cfg(test)]
mod tests {
    use super::{
        VerticalShapeOrientation, transform_or_rotate_rotation, vertical_glyph_metrics,
        vertical_shape_orientation,
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
}
