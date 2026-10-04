use std::sync::Arc;

use super::*;
use crate::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommandKind, UiResolvedStyle},
};

fn leaf_nodes(count: usize) -> Vec<Arc<UiRenderFrameCommandNode>> {
    (0..count)
        .map(|_| {
            Arc::new(UiRenderFrameCommandNode::Segment(
                Vec::<UiRenderCommand>::new().into(),
            ))
        })
        .collect()
}

fn promote_directory_level_cloning(
    nodes: Vec<Arc<UiRenderFrameCommandNode>>,
) -> Vec<Arc<UiRenderFrameCommandNode>> {
    nodes
        .chunks(UI_RENDER_FRAME_DIRECTORY_FANOUT)
        .map(|children| {
            Arc::new(UiRenderFrameCommandNode::Directory(
                children.to_vec().into(),
            ))
        })
        .collect()
}

fn sample_commands(count: usize, text_bytes: usize) -> Vec<UiRenderCommand> {
    (0..count)
        .map(|index| UiRenderCommand {
            node_id: UiNodeId::new(index as u64 + 1),
            kind: UiRenderCommandKind::Text,
            frame: UiFrame::new(index as f32, 0.0, 1.0, 1.0),
            clip_frame: None,
            z_index: 0,
            style: UiResolvedStyle::default(),
            text_layout: None,
            text: Some("x".repeat(text_bytes)),
            image: None,
            opacity: 1.0,
        })
        .collect()
}

fn deserialize_frame_commands_cloning(encoded: &[u8]) -> UiRenderFrameCommands {
    let commands = serde_json::from_slice::<Vec<UiRenderCommand>>(encoded).unwrap();
    UiRenderFrameCommands::from_slice(&commands)
}

#[test]
fn runtime_interface03_batch70_71_moved_directory_preserves_parent_nodes() {
    let nodes = leaf_nodes(UI_RENDER_FRAME_DIRECTORY_FANOUT * 2 + 1);
    let expected = promote_directory_level_cloning(nodes.clone());
    let mut directory_node_count = 0;
    let actual = UiRenderFrameCommands::promote_directory_level(nodes, &mut directory_node_count);

    assert_eq!(actual, expected);
    assert_eq!(directory_node_count, expected.len());
}

#[test]
#[ignore = "release-only moved render-frame command directory benchmark"]
fn runtime_interface03_batch70_71_moved_directory_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const LEAF_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 11;
    let nodes = leaf_nodes(LEAF_COUNT);
    let mut cloning_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut moved_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let cloning_input = nodes.clone();
        let moved_input = nodes.clone();
        let measure_cloning = || {
            let started = Instant::now();
            black_box(promote_directory_level_cloning(black_box(cloning_input)));
            started.elapsed().as_nanos()
        };
        let measure_moved = || {
            let started = Instant::now();
            let mut directory_node_count = 0;
            black_box(UiRenderFrameCommands::promote_directory_level(
                black_box(moved_input),
                &mut directory_node_count,
            ));
            black_box(directory_node_count);
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            cloning_samples.push(measure_cloning());
            moved_samples.push(measure_moved());
        } else {
            moved_samples.push(measure_moved());
            cloning_samples.push(measure_cloning());
        }
    }

    cloning_samples.sort_unstable();
    moved_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_MOVED_FRAME_COMMAND_DIRECTORY_BENCH_V1 leaves={LEAF_COUNT} samples={SAMPLE_COUNT} cloning_p95_ns={} moved_p95_ns={}",
        cloning_samples[p95], moved_samples[p95],
    );
    assert!(
        moved_samples[p95].saturating_mul(5) <= cloning_samples[p95].saturating_mul(4),
        "moved frame-command directory levels must improve P95 by at least 20%: cloning={}ns moved={}ns",
        cloning_samples[p95],
        moved_samples[p95],
    );
}

#[test]
fn runtime_interface03_batch70_71_streamed_deserialization_preserves_commands() {
    let commands = sample_commands(130, 32);
    let encoded = serde_json::to_vec(&commands).unwrap();

    assert_eq!(
        serde_json::from_slice::<UiRenderFrameCommands>(&encoded).unwrap(),
        deserialize_frame_commands_cloning(&encoded),
    );
}

#[test]
#[ignore = "release-only streamed render-frame command deserialization benchmark"]
fn runtime_interface03_batch70_71_streamed_deserialization_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const COMMAND_COUNT: usize = 1_024;
    const TEXT_BYTES: usize = 1_024;
    const DECODE_COUNT: usize = 8;
    const SAMPLE_COUNT: usize = 11;
    let encoded = serde_json::to_vec(&sample_commands(COMMAND_COUNT, TEXT_BYTES)).unwrap();
    let mut cloning_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut streamed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_cloning = || {
            let started = Instant::now();
            for _ in 0..DECODE_COUNT {
                black_box(deserialize_frame_commands_cloning(black_box(&encoded)));
            }
            started.elapsed().as_nanos()
        };
        let measure_streamed = || {
            let started = Instant::now();
            for _ in 0..DECODE_COUNT {
                black_box(serde_json::from_slice::<UiRenderFrameCommands>(black_box(
                    &encoded,
                )))
                .unwrap();
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            cloning_samples.push(measure_cloning());
            streamed_samples.push(measure_streamed());
        } else {
            streamed_samples.push(measure_streamed());
            cloning_samples.push(measure_cloning());
        }
    }

    cloning_samples.sort_unstable();
    streamed_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STREAMED_FRAME_COMMAND_DESERIALIZE_BENCH_V1 commands={COMMAND_COUNT} text_bytes={TEXT_BYTES} decodes={DECODE_COUNT} samples={SAMPLE_COUNT} cloning_p95_ns={} streamed_p95_ns={}",
        cloning_samples[p95], streamed_samples[p95],
    );
    assert!(
        streamed_samples[p95].saturating_mul(10) <= cloning_samples[p95].saturating_mul(9),
        "streamed frame-command deserialization must improve P95 by at least 10%: cloning={}ns streamed={}ns",
        cloning_samples[p95],
        streamed_samples[p95],
    );
}
