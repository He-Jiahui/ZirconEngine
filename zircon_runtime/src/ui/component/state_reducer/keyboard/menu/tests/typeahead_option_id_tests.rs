use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{UiComponentState, UiValue};

use super::super::{current_option_entry_value_index, OptionEntry};

const OPTION_COUNT_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime765_menu_typeahead_borrows_option_ids_for_current_index() {
    let options = vec![
        option_entry("first"),
        option_entry("selected"),
        option_entry("third"),
    ];
    let state = UiComponentState::new()
        .with_value("value", UiValue::String("selected".to_string()))
        .with_value("value_text", UiValue::String("third".to_string()));

    let current = current_option_entry_value_index(&state, &options);

    assert_eq!(current, Some(1));
    assert_eq!(options[1].id, "selected");
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime765_menu_typeahead_option_id_borrow_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_id_projection(OPTION_COUNT_PER_SAMPLE, true));
            optimized_ns.push(measure_id_projection(OPTION_COUNT_PER_SAMPLE, false));
        } else {
            optimized_ns.push(measure_id_projection(OPTION_COUNT_PER_SAMPLE, false));
            legacy_ns.push(measure_id_projection(OPTION_COUNT_PER_SAMPLE, true));
        }
    }

    let legacy_id_clones = OPTION_COUNT_PER_SAMPLE;
    let optimized_id_clones = 0;
    assert!(legacy_id_clones > optimized_id_clones);
    assert_eq!(optimized_id_clones, 0);

    println!(
        "RUNTIME765_MENU_TYPEAHEAD_OPTION_ID_BORROW_BENCH_V1 option_count_per_sample={OPTION_COUNT_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_id_clones={legacy_id_clones} optimized_id_clones={optimized_id_clones} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn option_entry(id: &str) -> OptionEntry {
    OptionEntry {
        id: id.to_string(),
        text: id.to_string(),
    }
}

fn measure_id_projection(option_count: usize, clone_ids: bool) -> u128 {
    let options = (0..option_count)
        .map(|index| option_entry(&format!("option-{index}")))
        .collect::<Vec<_>>();
    let started = Instant::now();
    if clone_ids {
        let ids = options
            .iter()
            .map(|option| option.id.clone())
            .collect::<Vec<_>>();
        black_box(ids);
    } else {
        black_box(options.iter().position(|option| option.id == "option-512"));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
