use std::hint::black_box;
use std::time::Instant;

use super::{
    menu_items_for_layout, menu_preset_item_capacity, HostMenuPointerLayout, MenuItemSpec,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const PRESETS_PER_BUILD: usize = 256;
const FIXED_MENU_ITEMS: usize = 3;

#[test]
fn optimization_batch_20260826er_editor133_capacity_preserves_preset_menu_actions() {
    let layout = HostMenuPointerLayout {
        resolved_preset_name: "editing".to_string(),
        preset_names: (0..PRESETS_PER_BUILD)
            .map(|index| format!("preset-{index}"))
            .collect(),
        ..HostMenuPointerLayout::default()
    };

    let items = menu_items_for_layout(&layout, 5).into_owned();

    assert_eq!(items.len(), PRESETS_PER_BUILD + FIXED_MENU_ITEMS);
    assert!(items.capacity() >= PRESETS_PER_BUILD + FIXED_MENU_ITEMS);
    assert_eq!(
        items[0].action_id.as_deref(),
        Some("workbench.layout.preset.save.editing")
    );
    assert_eq!(items[1].action_id.as_deref(), Some("window.layout.reset"));
    assert_eq!(
        items[PRESETS_PER_BUILD + 1].action_id.as_deref(),
        Some("workbench.layout.preset.load.preset-255")
    );
    assert_eq!(
        items.last().and_then(|item| item.action_id.as_deref()),
        Some("window.debug_observatory.open")
    );
    assert_eq!(menu_preset_item_capacity(0), FIXED_MENU_ITEMS);
    assert_eq!(
        menu_preset_item_capacity(PRESETS_PER_BUILD),
        PRESETS_PER_BUILD + FIXED_MENU_ITEMS
    );
}

#[test]
fn optimization_batch_20260826er_editor133_preset_menu_reserves_exact_output_count() {
    let source = include_str!("../menu_items_for_layout.rs");
    assert!(
        source.contains("Vec::with_capacity(menu_preset_item_capacity(layout.preset_names.len()))")
    );
    assert!(source.contains("preset_count.saturating_add(3)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826er_editor133_menu_preset_capacity_bench() {
    let item = MenuItemSpec {
        action_id: None,
        enabled: true,
        children: Vec::new(),
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&item, false));
            optimized_samples.push(measure(&item, true));
        } else {
            optimized_samples.push(measure(&item, true));
            legacy_samples.push(measure(&item, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR133_MENU_PRESET_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} presets_per_build={PRESETS_PER_BUILD} \
items_per_build={} legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        PRESETS_PER_BUILD + FIXED_MENU_ITEMS,
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(item: &MenuItemSpec, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut items = if reserve {
            Vec::with_capacity(PRESETS_PER_BUILD + FIXED_MENU_ITEMS)
        } else {
            Vec::new()
        };
        for _ in 0..PRESETS_PER_BUILD + FIXED_MENU_ITEMS {
            items.push(black_box(item.clone()));
        }
        checksum ^= black_box(items.len() ^ items.capacity());
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
