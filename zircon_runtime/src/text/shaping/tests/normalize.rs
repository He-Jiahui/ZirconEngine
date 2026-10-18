//! 检查源字节身份、合法本地区间及规范等价输入的区分。
//!
//! 夹取异常区间和真实后端成形不在这些局部断言的范围内。

use super::ShapingTextView;

/// 分解序列按原字节返回，字节长度亦保持。
#[test]
fn source_preserving_view_keeps_decomposed_source_bytes_unchanged() {
    let source = "a\u{0304}\u{0301}b";
    let view = ShapingTextView::source_preserving(source);

    assert_eq!(view.shaping_text(), source);
    assert_eq!(view.shaping_text().len(), source.len());
}

/// 合法非空区间与末尾空区间保持原本的源偏移。
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

/// 规范等价的两种表示仍各自保留不同字节与自身全区间。
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
