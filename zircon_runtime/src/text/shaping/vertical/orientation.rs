//! 竖排簇方向与步进的纯数值投影，供直接成形和已有字形的竖排布局共用。
//!
//! 本层只借用调用期间的文本，返回值不携带原文、字体资源或竖排替换凭据。

use unicode_vo::{char_orientation, Orientation};

use crate::text::layout_geometry::finite_geometry;
use crate::text::{ShapedGlyphRotation, VerticalMode};

/// 一个簇的竖排布局结果；三个字段均按值复制，不延长输入文本或字体的生命周期。
/// 步进与偏移沿用调用方的布局单位，具体数值约束由各计算入口说明。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct VerticalGlyphMetrics {
    /// 最终绘制旋转；空簇、全控制字符或零步进的早返会清除此旋转。
    pub(super) rotation: ShapedGlyphRotation,
    /// 沿竖排推进方向占用的步进；侧向簇可直接沿用已有步进。
    pub(super) advance: f32,
    /// 加到已有横向偏移上的居中补偿，不包含后端原有的字形偏移。
    pub(super) offset_x: f32,
}

/// 字符属性和显式模式给出的成形类别，不代表字体已执行竖排替换。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::text::shaping) enum VerticalShapeOrientation {
    /// 使用直立的竖排成形方向。
    Upright,
    /// 使用横排成形，布局时顺时针旋转九十度。
    Sideways,
    /// 允许后端先尝试竖排替换；最终旋转还需调用方提供替换结果。
    TransformOrRotate,
}

/// 按模式和文本分类，再计算一个簇的竖排布局数值。
///
/// - `mode`：显式直立、侧向或按字符属性混排。
/// - `cluster_text`：调用期间借用的簇文本；空串和全控制字符不占步进。
/// - `horizontal_advance`：已有步进，先取非负值；零值不合成字号步进。
/// - `font_size`：缺少有效原生步进时的合成依据，与已有步进使用同一布局单位。
/// - `native_vertical_advance`：预先取得的可选原生步进，只在未旋转分支参与选择。
///
/// 返回独立的数值结果。此入口没有字体替换凭据，Tr 类别按旋转后备处理；
/// 已有替换结论的直接后端应使用 `vertical_glyph_metrics_for_rotation`。
pub(super) fn vertical_glyph_metrics(
    mode: VerticalMode,
    cluster_text: &str,
    horizontal_advance: f32,
    font_size: f32,
    native_vertical_advance: Option<f32>,
) -> VerticalGlyphMetrics {
    let horizontal_advance = horizontal_advance.max(0.0);
    // 空簇和控制簇不能因缺少原生度量而合成出可见的字号步进。
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

/// 沿用调用方已经确定的旋转，避免覆盖直接后端的竖排替换结论。
///
/// - `cluster_text`：仅用于空串/全控制字符判断，不在此重新分类方向。
/// - `rotation`：调用方给出的绘制旋转；零步进或非显示簇仍会清除它。
/// - `horizontal_advance`：用于保留或居中比较的已有步进；直接路径也可传入成形后的竖排步进。
/// - `font_size`：未旋转且原生步进不可用时，按至少一个布局单位合成步进。
/// - `native_vertical_advance`：只有有限且为正的值才用于未旋转簇。
///
/// 返回旋转、推进量和附加居中偏移；侧向分支保留已有步进并令附加偏移为零。
/// 本层只筛选可选原生步进，调用方仍负责其余数值的输入契约。
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
    // 正常值沿用 f32 的舍入结果；候选非有限时才从宽精度差值恢复横向偏移。
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

/// 从 `mode` 和调用期间借用的 `cluster_text` 得到无字体凭据的绘制旋转。
///
/// 空串/全控制字符返回不旋转；其余输入按成形类别映射，Tr 采用顺时针九十度后备。
/// 需要根据实际竖排替换决定 Tr 旋转时，参见 `transform_or_rotate_rotation`。
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

/// 为调用期间借用的 `cluster_text` 文本片段选择竖排成形类别，不读取字体或判断实际替换。
///
/// `mode` 的显式直立/侧向设置优先于混排分类，但空串/全控制字符始终返回直立。
/// Mixed 逐字符查询 `unicode_vo`：任一 Upright/Tu 优先；否则有 Tr 返回待替换类别，
/// 全部剩余字符均为 Rotated 时返回侧向。此处不校验输入恰好是一个 grapheme。
/// 返回的分类可供 itemization 合并相邻片段，不携带文本借用或 Unicode 版本凭据。
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
            // 必须扫描到能确定整段优先级为止，不能只采用首个字符或先遇到的 Tr。
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

/// 把调用方已经判定的 Tr 替换结果投影为绘制旋转。
///
/// `vertical_substituted` 为真时保持直立，否则返回顺时针九十度后备。
/// 此布尔值由直接后端的字形比较提供；本函数不执行替换，也不验证字体或特性开关。
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
