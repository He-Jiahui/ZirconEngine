use super::*;
use crate::ui::{style::UiRgbaColor, surface::UiTextRange};

fn resolved_boxes(count: usize) -> Vec<UiResolvedTextBox> {
    (0..count)
        .map(|index| UiResolvedTextBox {
            range: UiTextRange {
                start: index,
                end: index + 1,
            },
            frame: UiFrame::new(index as f32, 0.0, 1.0, 16.0),
            background_color: Some(UiRgbaColor::from_u8(0x12, 0x34, 0x56, 0x78)),
            border_color: Some(UiRgbaColor::from_u8(0x87, 0x65, 0x43, 0x21)),
            border_width: 1.0,
        })
        .collect()
}

fn text_box_background_decorations_collecting(
    boxes: &[UiResolvedTextBox],
) -> Vec<UiTextPaintDecoration> {
    boxes
        .iter()
        .filter_map(|text_box| {
            text_box.background_color.map(|color| {
                UiTextPaintDecoration::table_cell_background(
                    text_box.range,
                    text_box.frame,
                    rgba_hex(color),
                )
            })
        })
        .collect()
}

fn text_box_border_decorations_collecting(
    boxes: &[UiResolvedTextBox],
) -> Vec<UiTextPaintDecoration> {
    boxes
        .iter()
        .filter_map(|text_box| {
            text_box.border_color.map(|color| {
                UiTextPaintDecoration::table_cell_border(
                    text_box.range,
                    text_box.frame,
                    rgba_hex(color),
                    text_box.border_width,
                )
            })
        })
        .collect()
}

fn collect_text_box_decorations(boxes: &[UiResolvedTextBox]) -> Vec<UiTextPaintDecoration> {
    let mut decorations = text_box_background_decorations_collecting(boxes);
    decorations.extend(text_box_border_decorations_collecting(boxes));
    decorations
}

fn append_text_box_decorations(boxes: &[UiResolvedTextBox]) -> Vec<UiTextPaintDecoration> {
    let mut decorations = Vec::with_capacity(boxes.len().saturating_mul(2));
    append_text_box_background_decorations(boxes, &mut decorations);
    append_text_box_border_decorations(boxes, &mut decorations);
    decorations
}

#[test]
fn runtime_interface03_batch74_75_direct_box_decoration_append_preserves_order_and_values() {
    let boxes = resolved_boxes(257);
    assert_eq!(
        append_text_box_decorations(&boxes),
        collect_text_box_decorations(&boxes),
    );
}

#[test]
#[ignore = "release-only direct text-box decoration append benchmark"]
fn runtime_interface03_batch74_75_direct_text_box_decoration_append_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const BOX_COUNT: usize = 4_096;
    const BUILD_COUNT: usize = 64;
    const SAMPLE_COUNT: usize = 11;
    let boxes = resolved_boxes(BOX_COUNT);
    let mut collecting_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut append_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_collecting = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(collect_text_box_decorations(black_box(&boxes)));
            }
            started.elapsed().as_nanos()
        };
        let measure_append = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(append_text_box_decorations(black_box(&boxes)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            collecting_samples.push(measure_collecting());
            append_samples.push(measure_append());
        } else {
            append_samples.push(measure_append());
            collecting_samples.push(measure_collecting());
        }
    }

    collecting_samples.sort_unstable();
    append_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_DIRECT_TEXT_BOX_DECORATION_APPEND_BENCH_V1 boxes={BOX_COUNT} builds={BUILD_COUNT} samples={SAMPLE_COUNT} collecting_p95_ns={} append_p95_ns={}",
        collecting_samples[p95], append_samples[p95],
    );
    assert!(
        append_samples[p95].saturating_mul(10) <= collecting_samples[p95].saturating_mul(9),
        "direct text-box decoration append must improve P95 by at least 10%: collecting={}ns append={}ns",
        collecting_samples[p95],
        append_samples[p95],
    );
}
