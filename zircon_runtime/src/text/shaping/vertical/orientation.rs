use unicode_vo::{char_orientation, Orientation};

use crate::text::layout_geometry::finite_geometry;
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
    let offset_x = (advance - horizontal_advance) * 0.5;
    let offset_x_exact = (f64::from(advance) - f64::from(horizontal_advance)) * 0.5;
    VerticalGlyphMetrics {
        rotation: ShapedGlyphRotation::None,
        advance,
        offset_x: offset_x
            .is_finite()
            .then_some(offset_x)
            .unwrap_or_else(|| finite_geometry(offset_x_exact)),
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
#[path = "tests/orientation.rs"]
mod tests;
