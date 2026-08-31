use std::hint::black_box;
use std::time::Instant;

use super::{
    push_paper_primitive_commands, reserve_paper_command_capacity, FrameRect, HostPaintCommand,
    TemplatePaneNodeData, MAX_PAPER_COMMANDS,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 32_768;

#[test]
fn optimization_batch_20260826em_editor128_capacity_preserves_five_layer_paper() {
    let node = TemplatePaneNodeData {
        component_role: "paper".to_owned(),
        elevation: 4.0,
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 8.0,
        y: 12.0,
        width: 80.0,
        height: 48.0,
    };
    let mut commands = Vec::new();

    assert!(push_paper_primitive_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        10,
        1.0,
    ));

    assert_eq!(commands.len(), MAX_PAPER_COMMANDS);
    assert!(commands.capacity() >= MAX_PAPER_COMMANDS);
}

#[test]
fn optimization_batch_20260826em_editor128_paper_reserves_maximum_command_count() {
    let source = include_str!("../commands.rs");
    let builder_start = source.find("fn push_paper_primitive_commands").unwrap();
    let builder_end = source[builder_start..]
        .find("fn reserve_paper_command_capacity")
        .map(|offset| builder_start + offset)
        .unwrap();
    let builder_source = &source[builder_start..builder_end];

    assert!(source.contains("const MAX_PAPER_COMMANDS: usize = 5;"));
    assert!(builder_source.contains("reserve_paper_command_capacity(commands);"));
    assert!(source.contains("commands.reserve(MAX_PAPER_COMMANDS);"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826em_editor128_paper_command_capacity_bench() {
    let commands = command_fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&commands, false));
            optimized_samples.push(measure(&commands, true));
        } else {
            optimized_samples.push(measure(&commands, true));
            legacy_samples.push(measure(&commands, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR128_PAPER_COMMAND_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} commands_per_build={MAX_PAPER_COMMANDS} \
legacy_allocations_per_empty_build=2 optimized_allocations_per_empty_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reserved paper command build P95 {optimized_p95_ns}ns must be at most 70% of growth-driven build P95 {legacy_p95_ns}ns"
    );
}

fn command_fixture() -> Vec<HostPaintCommand> {
    (0..MAX_PAPER_COMMANDS)
        .map(|index| {
            HostPaintCommand::quad(
                FrameRect {
                    x: index as f32,
                    y: 0.0,
                    width: 8.0,
                    height: 8.0,
                },
                None,
                index as i32,
                Some([32, 48, 64, 255]),
                None,
                0.0,
                0.0,
                1.0,
            )
        })
        .collect()
}

fn measure(commands: &[HostPaintCommand], reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = Vec::new();
        if reserve {
            reserve_paper_command_capacity(&mut output);
        }
        for command in commands {
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
