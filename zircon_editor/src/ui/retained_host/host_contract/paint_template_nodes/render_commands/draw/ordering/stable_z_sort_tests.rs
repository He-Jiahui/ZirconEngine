use std::hint::black_box;
use std::time::Instant;

use super::super::super::super::super::data::FrameRect;
use super::super::super::command::HostPaintCommand;
use super::fallback_ordered_commands;

const SAMPLE_PAIRS: usize = 21;
const SORTS_PER_SAMPLE: usize = 1_024;
const COMMAND_COUNT: usize = 256;

#[test]
fn optimization_batch_20260826el_editor127_host_paint_sort_preserves_equal_z_order() {
    let commands = vec![command(2, 0), command(1, 1), command(1, 2), command(0, 3)];
    let ordered = fallback_ordered_commands(&commands);

    assert!(std::ptr::eq(ordered[0], &commands[3]));
    assert!(std::ptr::eq(ordered[1], &commands[1]));
    assert!(std::ptr::eq(ordered[2], &commands[2]));
    assert!(std::ptr::eq(ordered[3], &commands[0]));
}

#[test]
fn optimization_batch_20260826el_editor127_host_paint_sort_drops_redundant_indices() {
    let source = include_str!("../ordering.rs");
    let helper_start = source.find("fn fallback_ordered_commands").unwrap();
    let helper_end = source[helper_start..]
        .find("fn z_indices_are_ordered")
        .map(|offset| helper_start + offset)
        .unwrap();
    let helper_source = &source[helper_start..helper_end];

    assert!(!helper_source.contains("enumerate()"));
    assert!(!helper_source.contains("(index, command)"));
    assert!(helper_source.contains("commands.iter().collect::<Vec<_>>()"));
    assert!(helper_source.contains("sort_by_key(|command| command.z_index)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826el_editor127_host_paint_command_stable_z_sort_bench() {
    let commands = command_fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&commands));
            optimized_samples.push(measure_optimized(&commands));
        } else {
            optimized_samples.push(measure_optimized(&commands));
            legacy_samples.push(measure_legacy(&commands));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR127_HOST_PAINT_COMMAND_STABLE_Z_SORT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
sorts_per_sample={SORTS_PER_SAMPLE} commands_per_sort={COMMAND_COUNT} \
legacy_index_fields_per_command=1 optimized_index_fields_per_command=0 \
legacy_element_bytes={} optimized_element_bytes={} legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        std::mem::size_of::<(usize, &HostPaintCommand)>(),
        std::mem::size_of::<&HostPaintCommand>(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "stable host-paint z sort P95 {optimized_p95_ns}ns must be at most 70% of indexed tuple sort P95 {legacy_p95_ns}ns"
    );
}

fn command(z_index: i32, identity: usize) -> HostPaintCommand {
    HostPaintCommand::quad(
        FrameRect {
            x: identity as f32,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        None,
        z_index,
        Some([identity as u8, 0, 0, 255]),
        None,
        0.0,
        0.0,
        1.0,
    )
}

fn command_fixture() -> Vec<HostPaintCommand> {
    (0..COMMAND_COUNT)
        .map(|index| command(((index * 37) % 17) as i32, index))
        .collect()
}

fn legacy_ordered_commands(commands: &[HostPaintCommand]) -> Vec<&HostPaintCommand> {
    let mut ordered = {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_collect_order");
        commands.iter().enumerate().collect::<Vec<_>>()
    };
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_sort");
        ordered.sort_by_key(|(index, command)| (command.z_index, *index));
    }
    ordered.into_iter().map(|(_, command)| command).collect()
}

fn measure_legacy(commands: &[HostPaintCommand]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0i32;
    for _ in 0..SORTS_PER_SAMPLE {
        let ordered = black_box(legacy_ordered_commands(black_box(commands)));
        checksum ^= ordered[0].z_index ^ ordered[ordered.len() - 1].z_index;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(commands: &[HostPaintCommand]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0i32;
    for _ in 0..SORTS_PER_SAMPLE {
        let ordered = black_box(fallback_ordered_commands(black_box(commands)));
        checksum ^= ordered[0].z_index ^ ordered[ordered.len() - 1].z_index;
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
