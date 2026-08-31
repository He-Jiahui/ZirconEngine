use super::*;

use std::hint::black_box;
use std::time::Instant;

use crate::ui::template::compile_binding_program;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::template::{UiBindingRef, UiTemplateNode};

const TARGET_BINDING_COUNT: usize = 16;

#[test]
#[ignore = "release-only compiled binding ownership lookup performance evidence"]
fn compiled_binding_ownership_lookup_p95_beats_program_scan() {
    const ROOT_ASSET: &str = "res://ui/views/large.zui";
    const TARGET_ASSET: &str = "res://ui/widgets/target.zui";
    const OTHER_ASSET: &str = "res://ui/widgets/other.zui";
    const BINDING_COUNT: usize = 4_096;
    const LOOKUPS_PER_SAMPLE: usize = 128;
    const SAMPLE_PAIRS: usize = 21;

    let bindings = (0..BINDING_COUNT)
        .map(|index| binding(&format!("binding/{index:04}")))
        .collect::<Vec<_>>();
    let binding_source_asset_ids = (0..BINDING_COUNT)
        .map(|index| {
            if index < TARGET_BINDING_COUNT {
                TARGET_ASSET.to_string()
            } else {
                OTHER_ASSET.to_string()
            }
        })
        .collect::<Vec<_>>();
    let root = UiTemplateNode {
        component: Some("LargeBindingOwner".to_string()),
        source_asset_id: Some(ROOT_ASSET.to_string()),
        binding_source_asset_ids,
        bindings,
        ..UiTemplateNode::default()
    };
    let program = compile_binding_program(&root, ROOT_ASSET).unwrap();
    let mut index = UiAssetSurfaceIndex::new();
    index.record_binding_program(tree_id("runtime.ui.binding-owner-perf"), &program);

    let _ = sample_lookup(&index, &program, TARGET_ASSET, LOOKUPS_PER_SAMPLE, true);
    let _ = sample_lookup(&index, &program, TARGET_ASSET, LOOKUPS_PER_SAMPLE, false);

    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples_us.push(sample_lookup(
                &index,
                &program,
                TARGET_ASSET,
                LOOKUPS_PER_SAMPLE,
                true,
            ));
            optimized_samples_us.push(sample_lookup(
                &index,
                &program,
                TARGET_ASSET,
                LOOKUPS_PER_SAMPLE,
                false,
            ));
        } else {
            optimized_samples_us.push(sample_lookup(
                &index,
                &program,
                TARGET_ASSET,
                LOOKUPS_PER_SAMPLE,
                false,
            ));
            legacy_samples_us.push(sample_lookup(
                &index,
                &program,
                TARGET_ASSET,
                LOOKUPS_PER_SAMPLE,
                true,
            ));
        }
    }

    let legacy_p95_us = nearest_rank_p95(&legacy_samples_us);
    let optimized_p95_us = nearest_rank_p95(&optimized_samples_us);
    let improvement_percent = if legacy_p95_us == 0 {
        0
    } else {
        legacy_p95_us
            .saturating_sub(optimized_p95_us)
            .saturating_mul(100)
            / legacy_p95_us
    };
    assert!(
        optimized_p95_us.saturating_mul(100) <= legacy_p95_us.saturating_mul(50),
        "compiled binding ownership P95 {optimized_p95_us}us must improve scan P95 {legacy_p95_us}us by at least 50%"
    );
    println!(
        "PERF-RUNTIME74-BINDING-RELOAD-OWNERSHIP sample_pairs={SAMPLE_PAIRS} binding_count={BINDING_COUNT} target_binding_count={TARGET_BINDING_COUNT} lookups_per_sample={LOOKUPS_PER_SAMPLE} pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 legacy_binding_scans_per_sample={} optimized_binding_scans_per_sample=0 legacy_samples_us={} optimized_samples_us={} legacy_p95_us={legacy_p95_us} optimized_p95_us={optimized_p95_us} improvement_percent={improvement_percent} improvement_threshold_percent=50",
        BINDING_COUNT * LOOKUPS_PER_SAMPLE,
        joined_samples(&legacy_samples_us),
        joined_samples(&optimized_samples_us),
    );
}

fn sample_lookup(
    index: &UiAssetSurfaceIndex,
    program: &zircon_runtime_interface::ui::template::UiCompiledBindingProgram,
    asset_id: &str,
    lookup_count: usize,
    legacy: bool,
) -> u128 {
    let started = Instant::now();
    let mut matched = 0usize;
    for _ in 0..lookup_count {
        let program = black_box(program);
        let index = black_box(index);
        let queried_asset = black_box(asset_id);
        let count = if legacy {
            program
                .iter_bindings()
                .filter(|binding| program.binding_asset_id(binding.handle) == Some(queried_asset))
                .count()
        } else {
            index.bindings_for_asset(queried_asset).count()
        };
        matched += black_box(count);
    }
    black_box(matched);
    assert_eq!(matched, TARGET_BINDING_COUNT * lookup_count);
    started.elapsed().as_micros().max(1)
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}

fn joined_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn binding(id: &str) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event: UiEventKind::Click,
        component_event: None,
        mode: Default::default(),
        route: None,
        action: None,
        targets: Vec::new(),
    }
}
