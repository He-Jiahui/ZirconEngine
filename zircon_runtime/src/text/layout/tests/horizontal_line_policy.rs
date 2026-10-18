//! 使用人工行与度量侧表检查组合包络策略的局部算术和拒绝路径。
//!
//! glyph 不带真实字体身份；本文件不执行字体匹配、成形后端或渲染。

use crate::core::framework::text::TextDirection;
use crate::text::{
    HorizontalGlyphMetricSpan, HorizontalLineRawMetrics, ShapedGlyph, ShapedHardLine, TextRange,
    TextStyle,
};

use super::resolve_horizontal_plain_line_policy;

/// 用两组人工度量和完整侧表构造行框；不以真实字体或渲染结果证明回退字体行为。
#[test]
fn plain_fallback_faces_share_the_composite_alphabetic_baseline_without_offsets() {
    let primary = HorizontalLineRawMetrics::new(12.0, 4.0, 2.0).expect("valid metrics");
    let fallback = HorizontalLineRawMetrics::new(15.0, 6.0, 0.0).expect("valid metrics");
    let line = test_line(2);
    let policy = resolve_horizontal_plain_line_policy(
        &TextStyle {
            font_size: 16.0,
            line_height: 18.0,
            ..TextStyle::default()
        },
        &line,
        HorizontalLineRawMetrics::new(15.0, 6.0, 2.0).expect("valid envelope"),
        &[
            HorizontalGlyphMetricSpan {
                line_index: 0,
                glyph_start: 0,
                glyph_end: 1,
                metrics: primary,
            },
            HorizontalGlyphMetricSpan {
                line_index: 0,
                glyph_start: 1,
                glyph_end: 2,
                metrics: fallback,
            },
        ],
    )
    .expect("complete selected-face provenance");

    assert_eq!(policy.metrics.width, 20.0);
    // BUG: [CR-ZCA-R112-0001] 此夹具完整覆盖两个 glyph，15/6/2 的组合度量算出行高23、基线16，宽度20断言先通过；
    // 若执行，下面期望15的断言将失败，后一断言不可达；这是期望与当前策略失配的静态证明，未执行测试。
    assert_eq!(policy.metrics.baseline, 15.0);
    assert_eq!(policy.metrics.line_height, 23.0);
}

/// 两个 glyph 仅给出首个的侧表，验证局部包络不完整时策略返回 `None`。
#[test]
fn partial_selected_face_provenance_cannot_replace_existing_line_metrics() {
    let metrics = HorizontalLineRawMetrics::new(12.0, 4.0, 2.0).expect("valid metrics");
    let line = test_line(2);

    assert!(resolve_horizontal_plain_line_policy(
        &TextStyle::default(),
        &line,
        metrics,
        &[HorizontalGlyphMetricSpan {
            line_index: 0,
            glyph_start: 0,
            glyph_end: 1,
            metrics,
        }],
    )
    .is_none());
}

/// 构造无字体身份的拥有型行夹具。
///
/// # 参数与返回
/// `glyph_count` 决定生成的 glyph 数量及人工索引区间；返回固定宽度为20的行。
///
/// # 约束
/// 本文件只以二调用；区间是夹具的人工索引，不表示经过解析的真实文本来源。
/// 返回值拥有新建的 glyph 向量，不借用外部字体或文本。
fn test_line(glyph_count: usize) -> ShapedHardLine {
    ShapedHardLine {
        line_index: 0,
        source_range: TextRange {
            start: 0,
            end: glyph_count,
        },
        visual_range: TextRange {
            start: 0,
            end: glyph_count,
        },
        measured_width: 20.0,
        baseline: 0.0,
        line_height: 0.0,
        glyphs: (0..glyph_count)
            .map(|index| ShapedGlyph {
                glyph_id: 1,
                font_id: None,
                font_instance_id: None,
                source_range: TextRange {
                    start: index,
                    end: index + 1,
                },
                visual_range: TextRange {
                    start: index,
                    end: index + 1,
                },
                advance: 10.0,
                x: index as f32 * 10.0,
                y: 0.0,
                offset_x: 0.0,
                offset_y: 0.0,
                direction: TextDirection::LeftToRight,
                bidi_level: 0,
                cluster_flags: Default::default(),
                rotation: Default::default(),
                script: Default::default(),
            })
            .collect(),
    }
}
