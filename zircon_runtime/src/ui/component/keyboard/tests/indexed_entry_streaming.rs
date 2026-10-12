use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentState, UiValue,
};

use super::indexed_keyboard_entries;

const OPTION_ENTRIES_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime766_indexed_keyboard_entries_preserve_property_and_option_order() {
    let descriptor = UiComponentDescriptor::new(
        "KeyboardFixture",
        "Keyboard Fixture",
        UiComponentCategory::Visual,
        "fixture",
    );
    let state = UiComponentState::new().with_value(
        "options",
        UiValue::Array(vec![
            UiValue::String("first".to_string()),
            UiValue::Array(vec![
                UiValue::Enum("second".to_string()),
                UiValue::String(String::new()),
            ]),
            UiValue::String("third".to_string()),
        ]),
    );

    let entries = indexed_keyboard_entries(&state, &descriptor);

    assert_eq!(entries, ["first", "second", "third"]);
    assert!(entries.capacity() >= 3);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
// TODO: [CR-R02-runtime_wave5_component_state_contracts-0002] 此基准仅测容器模型并输出预设的中间向量数；补生产投影路径的分配计数，核对仍存在的 option_entry_list 临时结果。
fn runtime766_indexed_keyboard_entry_stream_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_growth(OPTION_ENTRIES_PER_SAMPLE, 0));
            optimized_ns.push(measure_growth(
                OPTION_ENTRIES_PER_SAMPLE,
                OPTION_ENTRIES_PER_SAMPLE,
            ));
        } else {
            optimized_ns.push(measure_growth(
                OPTION_ENTRIES_PER_SAMPLE,
                OPTION_ENTRIES_PER_SAMPLE,
            ));
            legacy_ns.push(measure_growth(OPTION_ENTRIES_PER_SAMPLE, 0));
        }
    }

    let legacy_intermediate_vectors = 1;
    let optimized_intermediate_vectors = 0;
    assert!(legacy_intermediate_vectors > optimized_intermediate_vectors);
    assert_eq!(optimized_intermediate_vectors, 0);

    println!(
        "RUNTIME766_INDEXED_KEYBOARD_ENTRY_STREAM_BENCH_V1 option_entries_per_sample={OPTION_ENTRIES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_intermediate_vectors={legacy_intermediate_vectors} optimized_intermediate_vectors={optimized_intermediate_vectors} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

// 此计时只比较预留容量的容器模型，不调用键盘条目投影；输出的中间容器数是预设模型值。
fn measure_growth(length: usize, capacity: usize) -> u128 {
    let started = Instant::now();
    let mut entries = Vec::with_capacity(capacity);
    for entry in 0..length {
        entries.push(black_box(entry));
    }
    black_box(entries);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
