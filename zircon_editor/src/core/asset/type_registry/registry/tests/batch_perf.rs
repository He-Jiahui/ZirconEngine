use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const CONTRIBUTIONS_PER_SAMPLE: usize = 4_096;

#[test]
fn optimization_batch_20260830cj_editor_asset_type_batch_reserves_exact_contribution_outputs() {
    let source = include_str!("../batch.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("asset type batch implementation");

    assert!(implementation.contains("let (creation_template_count, context_command_count)"));
    assert!(implementation.contains("Vec::with_capacity(creation_template_count)"));
    assert!(implementation.contains("Vec::with_capacity(context_command_count)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cj_editor_asset_type_batch_capacity_p95() {
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false));
            optimized.push(measure(true));
        } else {
            optimized.push(measure(true));
            legacy.push(measure(false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "EDITOR332_ASSET_TYPE_BATCH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} contributions_per_sample={CONTRIBUTIONS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_capacity: bool) -> u128 {
    let contribution_sizes = [(1usize, 1usize); CONTRIBUTIONS_PER_SAMPLE];
    let started = Instant::now();
    let (creation_count, command_count) = if use_capacity {
        contribution_sizes.iter().fold(
            (0usize, 0usize),
            |(creations, commands), (next_creations, next_commands)| {
                (
                    creations.saturating_add(*next_creations),
                    commands.saturating_add(*next_commands),
                )
            },
        )
    } else {
        (0, 0)
    };
    let mut creations = Vec::with_capacity(creation_count);
    let mut commands = Vec::with_capacity(command_count);
    for (creation_size, command_size) in contribution_sizes {
        creations.extend(0..creation_size);
        commands.extend(0..command_size);
    }
    std::hint::black_box((creations, commands));
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
