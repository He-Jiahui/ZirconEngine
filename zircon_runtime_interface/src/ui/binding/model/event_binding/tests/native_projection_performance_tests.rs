use std::{hint::black_box, time::Instant};

use super::UiEventBinding;
use crate::ui::binding::model::{UiBindingCall, UiBindingValue, UiEventKind, UiEventPath};

fn sample_binding() -> UiEventBinding {
    let values = (0..256)
        .map(|index| UiBindingValue::String(format!("value-{index:03}-{}", "x".repeat(96))))
        .collect();
    UiEventBinding::new(
        UiEventPath::new(
            "Editor.AssetBrowser.MainPanel",
            "Content.LongLivedSelectionList",
            UiEventKind::Change,
        ),
        UiBindingCall::new("ApplySelection").with_argument(UiBindingValue::Array(values)),
    )
}

fn native_binding_allocating(binding: &UiEventBinding) -> String {
    let prefix = binding.path.native_prefix();
    match &binding.action {
        Some(action) => format!("{prefix}({})", action.native_repr()),
        None => prefix,
    }
}

#[test]
fn single_buffer_event_binding_preserves_action_and_actionless_output() {
    let binding = sample_binding();
    assert_eq!(
        binding.native_binding(),
        native_binding_allocating(&binding)
    );

    let actionless =
        UiEventBinding::without_action(UiEventPath::new("Editor", "Canvas", UiEventKind::Click));
    assert_eq!(
        actionless.native_binding(),
        native_binding_allocating(&actionless),
    );
}

#[test]
#[ignore = "release-only single-buffer event binding projection benchmark"]
fn runtime_interface03_batch39_single_buffer_event_binding_release_benchmark() {
    const ITERATIONS: usize = 10_000;
    const SAMPLE_COUNT: usize = 11;
    let binding = sample_binding();
    let mut allocating_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut single_buffer_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_allocating = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(native_binding_allocating(black_box(&binding)));
            }
            started.elapsed().as_nanos()
        };
        let measure_single_buffer = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(binding.native_binding());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            allocating_samples.push(measure_allocating());
            single_buffer_samples.push(measure_single_buffer());
        } else {
            single_buffer_samples.push(measure_single_buffer());
            allocating_samples.push(measure_allocating());
        }
    }

    allocating_samples.sort_unstable();
    single_buffer_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SINGLE_BUFFER_EVENT_BINDING_BENCH_V1 iterations={ITERATIONS} samples={SAMPLE_COUNT} allocating_p95_ns={} single_buffer_p95_ns={}",
        allocating_samples[p95], single_buffer_samples[p95],
    );
    assert!(
        single_buffer_samples[p95].saturating_mul(5)
            <= allocating_samples[p95].saturating_mul(4),
        "single-buffer event binding projection must improve P95 by at least 20%: allocating={}ns single_buffer={}ns",
        allocating_samples[p95],
        single_buffer_samples[p95],
    );
}
