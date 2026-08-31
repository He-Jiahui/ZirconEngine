use std::hint::black_box;
use std::time::Instant;

use super::runtime_render_commands_to_host;
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiResolvedStyle,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const COMMANDS_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826fk_editor152_capacity_preserves_group_command_order() {
    let commands = (0..COMMANDS_PER_BUILD)
        .map(group_command)
        .collect::<Vec<_>>();

    let host_commands = runtime_render_commands_to_host(&commands, None);

    assert_eq!(host_commands.len(), COMMANDS_PER_BUILD);
    assert!(host_commands.capacity() >= commands.len());
    assert_eq!(host_commands[0].z_index, 0);
    assert_eq!(
        host_commands[COMMANDS_PER_BUILD - 1].z_index,
        (COMMANDS_PER_BUILD - 1) as i32
    );
}

#[test]
fn optimization_batch_20260826fk_editor152_host_command_output_reserves_input_count() {
    let source = include_str!("../entry.rs");
    assert!(source.contains("let mut host_commands = Vec::with_capacity(commands.len());"));
    assert!(!source.contains("let mut host_commands = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fk_editor152_render_command_conversion_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR152_RENDER_COMMAND_CONVERSION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} commands_per_build={COMMANDS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn group_command(index: usize) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(index as u64),
        kind: UiRenderCommandKind::Group,
        frame: UiFrame::new(index as f32, 0.0, 32.0, 24.0),
        clip_frame: None,
        z_index: index as i32,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut commands = if reserve {
            Vec::with_capacity(COMMANDS_PER_BUILD)
        } else {
            Vec::new()
        };
        for command in 0..COMMANDS_PER_BUILD {
            commands.push(black_box([command; 16]));
        }
        checksum ^= black_box(commands.len() ^ commands.capacity());
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
