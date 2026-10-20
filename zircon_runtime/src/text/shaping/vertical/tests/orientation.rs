//! 方向分类与数值投影的固定样例契约，直接调用父模块的纯计算入口。
//!
//! 这里没有加载字体或执行竖排替换；样例不能外推为完整 Unicode 或任意浮点输入的证明。

use super::{
    transform_or_rotate_rotation, vertical_glyph_metrics, vertical_shape_orientation,
    VerticalShapeOrientation,
};
use crate::text::{ShapedGlyphRotation, VerticalMode};

/// 缺少原生竖排步进时，汉字样例使用字号合成步进并补偿横向居中。
#[test]
fn text_vertical_cjk_upright_uses_synthesized_em_advance() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "本", 18.0, 20.0, None);

    assert_eq!(metrics.rotation, ShapedGlyphRotation::None);
    assert_eq!(metrics.advance, 20.0);
    assert_eq!(metrics.offset_x, 1.0);
}

/// 拉丁样例旋转后沿用已有步进，提供的原生竖排步进不参与该分支。
#[test]
fn text_vertical_latin_sideways_preserves_horizontal_advance() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "A", 11.0, 20.0, Some(32.0));

    assert_eq!(metrics.rotation, ShapedGlyphRotation::Cw90);
    assert_eq!(metrics.advance, 11.0);
    assert_eq!(metrics.offset_x, 0.0);
}

/// 全角句号样例保持直立并居中；该样例不规定所有标点的分类。
#[test]
fn text_vertical_punctuation_is_upright_and_centered() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "。", 8.0, 20.0, None);

    assert_eq!(metrics.rotation, ShapedGlyphRotation::None);
    assert_eq!(metrics.advance, 20.0);
    assert_eq!(metrics.offset_x, 6.0);
}

/// 两个固定字符分别检验显式直立/侧向模式优先于 Mixed 的字符属性选择。
#[test]
fn text_vertical_modes_override_unicode_mixed_orientation() {
    let upright = vertical_glyph_metrics(VerticalMode::Upright, "A", 11.0, 20.0, None);
    let sideways = vertical_glyph_metrics(VerticalMode::Sideways, "本", 18.0, 20.0, Some(32.0));

    assert_eq!(upright.rotation, ShapedGlyphRotation::None);
    assert_eq!(upright.advance, 20.0);
    assert_eq!(sideways.rotation, ShapedGlyphRotation::Cw90);
    assert_eq!(sideways.advance, 18.0);
}

/// 预先提供的有效原生步进优先于字号合成值；测试本身不读取字体度量表。
#[test]
fn text_vertical_upright_prefers_native_vmtx_advance() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "本", 18.0, 20.0, Some(24.5));

    assert_eq!(metrics.rotation, ShapedGlyphRotation::None);
    assert_eq!(metrics.advance, 24.5);
    assert_eq!(metrics.offset_x, 3.25);
}

/// 有限极值的已有步进仍得到预期的有限居中偏移。
/// 此样例不证明宽精度恢复分支实际执行，也不覆盖其他异常数值组合。
#[test]
fn text_vertical_upright_offset_stays_finite_for_extreme_horizontal_advance() {
    let metrics = vertical_glyph_metrics(VerticalMode::Mixed, "本", f32::MAX, 20.0, None);

    assert!(metrics.offset_x.is_finite());
    assert_eq!(metrics.offset_x, -f32::MAX * 0.5);
}

/// 全角左括号给出 Tr 类别，再分别投影调用方预先给定的替换布尔结果。
/// 这里检验分类和旋转映射，不执行真实字体替换。
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
