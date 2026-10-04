use super::*;
use crate::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiLayoutMetrics},
    surface::{UiRenderCommandKind, UiRenderList, UiResolvedStyle},
};

fn render_list(command_count: usize) -> UiRenderList {
    UiRenderList {
        commands: (0..command_count).map(command).collect(),
    }
}

fn command(index: usize) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(index as u64 + 1),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(index as f32, 0.0, 1.0, 1.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}

fn legacy_flat_elements(list: &UiRenderList, metrics: UiLayoutMetrics) -> Vec<UiPaintElement> {
    let mut elements = Vec::new();
    let mut next_paint_order = 0;
    for command in &list.commands {
        let mut command_elements =
            command.to_paint_elements_with_metrics(next_paint_order, metrics);
        next_paint_order += command_elements.len() as u64;
        elements.append(&mut command_elements);
    }
    elements
}

fn legacy_frame_elements(
    list: &UiRenderFrameList,
    metrics: UiLayoutMetrics,
) -> Vec<UiPaintElement> {
    let mut elements = Vec::new();
    let mut next_paint_order = 0;
    for command in &list.commands {
        let mut command_elements =
            command.to_paint_elements_with_metrics(next_paint_order, metrics);
        next_paint_order += command_elements.len() as u64;
        elements.append(&mut command_elements);
    }
    elements
}

#[test]
fn runtime_interface03_batch4_direct_append_lists_match_per_command_collection() {
    let metrics = UiLayoutMetrics::default();
    let list = render_list(64);
    assert_eq!(
        list.to_paint_elements_with_metrics(metrics),
        legacy_flat_elements(&list, metrics),
    );

    let extract = UiRenderExtract {
        tree_id: UiTreeId::new("direct.append"),
        list,
        raster_scale: 1.0,
    };
    let frame = UiRenderFrameExtract::from_extract(&extract);
    assert_eq!(
        frame.list.to_paint_elements_with_metrics(metrics),
        legacy_frame_elements(&frame.list, metrics),
    );
}

#[test]
#[ignore = "release-only flat render-list direct append benchmark"]
fn runtime_interface03_batch4_render_list_direct_append_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const COMMAND_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 11;
    let list = render_list(COMMAND_COUNT);
    let metrics = UiLayoutMetrics::default();
    let mut temporary_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut direct_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_temporary = || {
            let started = Instant::now();
            black_box(legacy_flat_elements(black_box(&list), black_box(metrics)));
            started.elapsed().as_nanos()
        };
        let measure_direct = || {
            let started = Instant::now();
            black_box(black_box(&list).to_paint_elements_with_metrics(black_box(metrics)));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            temporary_samples.push(measure_temporary());
            direct_samples.push(measure_direct());
        } else {
            direct_samples.push(measure_direct());
            temporary_samples.push(measure_temporary());
        }
    }

    temporary_samples.sort_unstable();
    direct_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_RENDER_LIST_DIRECT_APPEND_BENCH_V1 commands={COMMAND_COUNT} samples={SAMPLE_COUNT} temporary_p50_ns={} direct_p50_ns={} temporary_p95_ns={} direct_p95_ns={}",
        temporary_samples[p50], direct_samples[p50], temporary_samples[p95], direct_samples[p95],
    );
    assert!(
        direct_samples[p95].saturating_mul(10) <= temporary_samples[p95].saturating_mul(9),
        "flat render-list direct append must improve P95 by at least 10%: temporary={}ns direct={}ns",
        temporary_samples[p95],
        direct_samples[p95],
    );
}

#[test]
#[ignore = "release-only persistent render-frame list direct append benchmark"]
fn runtime_interface03_batch4_render_frame_list_direct_append_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const COMMAND_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 11;
    let extract = UiRenderExtract {
        tree_id: UiTreeId::new("frame.direct.append"),
        list: render_list(COMMAND_COUNT),
        raster_scale: 1.0,
    };
    let frame = UiRenderFrameExtract::from_extract(&extract);
    let metrics = UiLayoutMetrics::default();
    let mut temporary_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut direct_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_temporary = || {
            let started = Instant::now();
            black_box(legacy_frame_elements(
                black_box(&frame.list),
                black_box(metrics),
            ));
            started.elapsed().as_nanos()
        };
        let measure_direct = || {
            let started = Instant::now();
            black_box(black_box(&frame.list).to_paint_elements_with_metrics(black_box(metrics)));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            temporary_samples.push(measure_temporary());
            direct_samples.push(measure_direct());
        } else {
            direct_samples.push(measure_direct());
            temporary_samples.push(measure_temporary());
        }
    }

    temporary_samples.sort_unstable();
    direct_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_RENDER_FRAME_LIST_DIRECT_APPEND_BENCH_V1 commands={COMMAND_COUNT} samples={SAMPLE_COUNT} temporary_p50_ns={} direct_p50_ns={} temporary_p95_ns={} direct_p95_ns={}",
        temporary_samples[p50], direct_samples[p50], temporary_samples[p95], direct_samples[p95],
    );
    assert!(
        direct_samples[p95].saturating_mul(10) <= temporary_samples[p95].saturating_mul(9),
        "persistent render-frame list direct append must improve P95 by at least 10%: temporary={}ns direct={}ns",
        temporary_samples[p95],
        direct_samples[p95],
    );
}
