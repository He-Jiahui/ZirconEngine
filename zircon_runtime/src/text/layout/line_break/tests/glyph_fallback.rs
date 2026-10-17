//! 字素兜底测试辅助入口的三个合成夹具。
//! 这些断言经过默认提供者重新测量，不直接调用生产的已有行进距离判定。
//! 不据此证明生产数值边界、连接字符许可计算或完整字素序列覆盖。

use super::super::super::measure::measure_line_width;
use super::should_fallback_to_glyph_wrap;
use crate::text::TextStyle;

#[test]
fn overwide_plain_chunk_requests_glyph_wrap_fallback() {
    let style = TextStyle::default();
    let max_width = measure_line_width("a", &style)
        .into_result()
        .expect("measure test line width")
        + 0.1;

    assert!(should_fallback_to_glyph_wrap(
        true, "abcd", max_width, &style
    ));
}

// 此处直接给定关闭许可，断言不覆盖连接字符检测函数。
#[test]
fn overwide_glue_chunk_does_not_request_glyph_wrap_fallback() {
    let style = TextStyle::default();
    let max_width = measure_line_width("a", &style)
        .into_result()
        .expect("measure test line width")
        + 0.1;

    assert!(!should_fallback_to_glyph_wrap(
        false,
        "a\u{2060}b",
        max_width,
        &style
    ));
}

// 此夹具未单独断言测量超宽，不能作为超宽前提已被隔离验证的证据。
#[test]
fn single_grapheme_chunk_does_not_request_glyph_wrap_fallback() {
    let style = TextStyle::default();

    assert!(!should_fallback_to_glyph_wrap(true, "W", 1.0, &style));
}
