use std::{hint::black_box, time::Instant};

use crate::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiLayoutMetrics},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
};

use super::{UiRenderFrameCommands, UiRenderFrameList};

const COMMAND_COUNT: usize = 4_097;
const LOOKUP_COUNT: usize = 8;
const SAMPLE_COUNT: usize = 11;

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

fn frame_list() -> UiRenderFrameList {
    let commands = (0..COMMAND_COUNT).map(command).collect::<Vec<_>>();
    UiRenderFrameList {
        commands: UiRenderFrameCommands::from_slice(&commands),
    }
}

#[test]
fn runtime_interface03_batch20_render_frame_capacity_preserves_projection() {
    let list = frame_list();
    let metrics = UiLayoutMetrics::default();
    let unreserved = list.to_paint_elements_with_metrics_unreserved(metrics);
    let reserved = list.to_paint_elements_with_metrics(metrics);

    assert_eq!(reserved, unreserved);
    assert_eq!(reserved.len(), COMMAND_COUNT);
    assert_eq!(reserved.capacity(), COMMAND_COUNT);
    assert!(
        reserved.capacity().saturating_mul(3) <= unreserved.capacity().saturating_mul(2),
        "reserved projection must reduce retained capacity by at least 33%: unreserved={} reserved={}",
        unreserved.capacity(),
        reserved.capacity(),
    );
}

#[test]
#[ignore = "release-only render-frame paint-element capacity benchmark"]
fn runtime_interface03_batch20_render_frame_capacity_release_benchmark() {
    let list = frame_list();
    let metrics = UiLayoutMetrics::default();
    let unreserved_capacity = list
        .to_paint_elements_with_metrics_unreserved(metrics)
        .capacity();
    let reserved_capacity = list.to_paint_elements_with_metrics(metrics).capacity();
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_unreserved = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(list.to_paint_elements_with_metrics_unreserved(black_box(metrics)));
            }
            started.elapsed().as_nanos()
        };
        let measure_reserved = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(list.to_paint_elements_with_metrics(black_box(metrics)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            unreserved_samples.push(measure_unreserved());
            reserved_samples.push(measure_reserved());
        } else {
            reserved_samples.push(measure_reserved());
            unreserved_samples.push(measure_unreserved());
        }
    }

    unreserved_samples.sort_unstable();
    reserved_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_RENDER_FRAME_PAINT_CAPACITY_BENCH_V1 commands={COMMAND_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} unreserved_capacity={unreserved_capacity} reserved_capacity={reserved_capacity} unreserved_p95_ns={} reserved_p95_ns={}",
        unreserved_samples[p95], reserved_samples[p95],
    );
    assert_eq!(reserved_capacity, COMMAND_COUNT);
    assert!(
        reserved_capacity.saturating_mul(3) <= unreserved_capacity.saturating_mul(2),
        "reserved projection must reduce retained capacity by at least 33%: unreserved={unreserved_capacity} reserved={reserved_capacity}",
    );
}
