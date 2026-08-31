use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiStateFlags,
    surface::{UiRenderCommandKind, UiResolvedStyle},
};

use super::super::shared::CollectionRowKind;
use super::{
    RowRenderState, UiFrame, UiNodeId, UiRenderCommand, UiTemplateNodeMetadata,
    tree_row_command_capacity, tree_row_commands,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 1_024;
const BENCH_DEPTH: usize = 256;
const COMMANDS_PER_BUILD: usize = BENCH_DEPTH + 6;

#[test]
fn optimization_batch_20260826en_runtime183_capacity_preserves_maximum_tree_row() {
    let mut metadata = UiTemplateNodeMetadata {
        component: "TreeRow".to_owned(),
        ..UiTemplateNodeMetadata::default()
    };
    metadata
        .attributes
        .insert("tree_depth".to_owned(), Value::Integer(4));
    metadata
        .attributes
        .insert("label".to_owned(), Value::String("Player".to_owned()));
    metadata
        .attributes
        .insert("selected".to_owned(), Value::Boolean(true));
    let state = RowRenderState::resolve(
        CollectionRowKind::Tree,
        &metadata,
        &UiStateFlags {
            visible: true,
            enabled: true,
            ..UiStateFlags::default()
        },
        None,
    );

    let commands = tree_row_commands(
        UiNodeId::new(7),
        &metadata,
        &state,
        UiFrame::new(4.0, 8.0, 320.0, 28.0),
        None,
        10,
        1.0,
    );

    assert_eq!(commands.len(), tree_row_command_capacity(4));
    assert!(commands.capacity() >= tree_row_command_capacity(4));
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.kind == UiRenderCommandKind::Quad)
            .count(),
        5
    );
}

#[test]
fn optimization_batch_20260826en_runtime183_tree_row_reuses_depth_and_capacity_bound() {
    let source = include_str!("../tree.rs");
    let builder_start = source.find("fn tree_row_commands").unwrap();
    let builder_end = source[builder_start..]
        .find("fn background")
        .map(|offset| builder_start + offset)
        .unwrap();
    let builder_source = &source[builder_start..builder_end];
    let disclosure_start = source.find("fn disclosure_rect").unwrap();
    let disclosure_end = source[disclosure_start..]
        .find("fn action_rect")
        .map(|offset| disclosure_start + offset)
        .unwrap();
    let disclosure_source = &source[disclosure_start..disclosure_end];

    assert!(source.contains("const MAX_TREE_ROW_BASE_COMMANDS: usize = 6;"));
    assert!(builder_source.contains("let depth = depth(metadata);"));
    assert!(builder_source.contains("Vec::with_capacity(tree_row_command_capacity(depth))"));
    assert!(builder_source.contains("for level in 0..depth"));
    assert!(builder_source.contains("disclosure_rect(metadata, frame, &visual, depth)"));
    assert!(!disclosure_source.contains("depth(metadata)"));
    assert!(source.contains("depth.saturating_add(MAX_TREE_ROW_BASE_COMMANDS)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826en_runtime183_tree_row_command_capacity_bench() {
    let command = command_fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&command, false));
            optimized_samples.push(measure(&command, true));
        } else {
            optimized_samples.push(measure(&command, true));
            legacy_samples.push(measure(&command, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME183_TREE_ROW_COMMAND_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} depth={BENCH_DEPTH} \
commands_per_build={COMMANDS_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reserved tree-row command build P95 {optimized_p95_ns}ns must be at most 70% of growth-driven build P95 {legacy_p95_ns}ns"
    );
}

fn command_fixture() -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(9),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(0.0, 0.0, 8.0, 8.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}

fn measure(command: &UiRenderCommand, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            Vec::with_capacity(tree_row_command_capacity(BENCH_DEPTH))
        } else {
            Vec::new()
        };
        for _ in 0..COMMANDS_PER_BUILD {
            output.push(black_box(command.clone()));
        }
        checksum ^= black_box(output.len() ^ output.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
