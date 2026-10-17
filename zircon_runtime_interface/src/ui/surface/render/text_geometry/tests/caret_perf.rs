//! 用固定单字节行夹具对照行选择与亲和性，不调用字体整形。
//! 显式忽略的计时用例排除夹具构造；忽略属性本身不强制构建模式。
//! 排序后使用末项观测样本，不能把测试名称当作统计或产品性能验收。

use std::{hint::black_box, time::Instant};

use crate::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedTextLayout, UiResolvedTextLine, UiTextCaret, UiTextCaretAffinity,
        UiTextDirection, UiTextRange,
    },
};

use super::{caret_line, linear_caret_line};

const LINE_COUNT: usize = 65_536;
const LOOKUP_COUNT: usize = 64;
const SAMPLE_COUNT: usize = 11;

/// 以行号同时构造源字节范围与纵向坐标；本文件只传入固定有限区间。
/// 返回拥有数据的单字节行；文本运行列表为空，不执行文本整形。
fn line(index: usize) -> UiResolvedTextLine {
    UiResolvedTextLine {
        text: "x".to_string(),
        frame: UiFrame::new(0.0, index as f32, 1.0, 1.0),
        placement_frame: UiFrame::new(0.0, index as f32, 1.0, 1.0),
        source_range: UiTextRange {
            start: index,
            end: index + 1,
        },
        visual_range: UiTextRange { start: 0, end: 1 },
        measured_width: 1.0,
        glyph_advances: vec![1.0],
        baseline: 1.0,
        direction: UiTextDirection::LeftToRight,
        runs: Vec::new(),
        ellipsized: false,
    }
}

/// 返回拥有指定数量合成行的布局，其余字段沿用缺省值。
fn layout(line_count: usize) -> UiResolvedTextLayout {
    UiResolvedTextLayout {
        lines: (0..line_count).map(line).collect(),
        ..UiResolvedTextLayout::default()
    }
}

fn caret(offset: usize, affinity: UiTextCaretAffinity) -> UiTextCaret {
    UiTextCaret { offset, affinity }
}

/// 对照八行中的二十组位置与亲和性，另核共享边界的具体行和空布局。
#[test]
fn runtime_interface03_batch11_binary_caret_line_lookup_preserves_affinity() {
    let layout = layout(8);
    for offset in 0..=9 {
        for affinity in [
            UiTextCaretAffinity::Upstream,
            UiTextCaretAffinity::Downstream,
        ] {
            let caret = caret(offset, affinity);
            let binary = caret_line(&layout, &caret).map(|line| line.source_range);
            let linear = linear_caret_line(&layout, &caret).map(|line| line.source_range);
            assert_eq!(binary, linear, "offset={offset} affinity={affinity:?}");
        }
    }

    assert_eq!(
        caret_line(&layout, &caret(4, UiTextCaretAffinity::Upstream))
            .expect("upstream boundary")
            .source_range,
        UiTextRange { start: 3, end: 4 },
    );
    assert_eq!(
        caret_line(&layout, &caret(4, UiTextCaretAffinity::Downstream))
            .expect("downstream boundary")
            .source_range,
        UiTextRange { start: 4, end: 5 },
    );
    assert!(caret_line(
        &UiResolvedTextLayout::default(),
        &caret(0, Default::default())
    )
    .is_none());
}

/// 夹具在计时外；每样本各做六十四次行选择，交替两路径先后顺序。
/// 只描述这个固定负载，未包含字体、映射构造或渲染提交。
#[test]
#[ignore = "release-only binary caret-line lookup benchmark"]
fn runtime_interface03_batch11_binary_caret_line_lookup_release_benchmark() {
    let layout = layout(LINE_COUNT);
    let caret = caret(LINE_COUNT - 2, UiTextCaretAffinity::Downstream);
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut binary_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(linear_caret_line(black_box(&layout), black_box(&caret)));
            }
            started.elapsed().as_nanos()
        };
        let measure_binary = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(caret_line(black_box(&layout), black_box(&caret)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            binary_samples.push(measure_binary());
        } else {
            binary_samples.push(measure_binary());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    binary_samples.sort_unstable();
    // 十一个样本排序后取末项，即这批样本的最大观测值。
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BINARY_CARET_LINE_LOOKUP_BENCH_V1 lines={LINE_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} binary_p95_ns={}",
        linear_samples[p95],
        binary_samples[p95],
    );
    assert!(
        binary_samples[p95].saturating_mul(10) <= linear_samples[p95],
        "binary caret lookup must improve P95 by at least 90%: linear={}ns binary={}ns",
        linear_samples[p95],
        binary_samples[p95],
    );
}
