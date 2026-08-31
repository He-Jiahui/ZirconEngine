use std::collections::BTreeMap;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::types::{ExtensionBindingEventKind, ExtensionBindingSpec};

const WORKBENCH_EXTENSION_VIEW_ID: &str = "WorkbenchExtension";

pub(super) fn insert_workbench_extension_bindings(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    specs: &[ExtensionBindingSpec],
) {
    for spec in specs {
        insert_event(
            bindings,
            WORKBENCH_EXTENSION_VIEW_ID,
            spec.control_id,
            editor_event_kind(spec.event_kind),
            EditorUiBindingPayload::menu_action(spec.action_id),
        );
    }
}

fn editor_event_kind(event_kind: ExtensionBindingEventKind) -> EditorUiEventKind {
    match event_kind {
        ExtensionBindingEventKind::Click => EditorUiEventKind::Click,
        ExtensionBindingEventKind::Change => EditorUiEventKind::Change,
        ExtensionBindingEventKind::Submit => EditorUiEventKind::Submit,
    }
}

fn insert_event(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    event_kind: EditorUiEventKind,
    payload: EditorUiBindingPayload,
) {
    bindings.insert(
        binding_key(view_id, control_id),
        EditorUiBinding::new(view_id, control_id, event_kind, payload),
    );
}

fn binding_key(view_id: &str, control_id: &str) -> String {
    let mut key = String::with_capacity(view_id.len() + 1 + control_id.len());
    key.push_str(view_id);
    key.push('/');
    key.push_str(control_id);
    key
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::binding_key;

    #[test]
    fn optimization_batch_20260831gs_editor574_binding_key_preserves_segments() {
        assert_eq!(
            binding_key("WorkbenchExtension", "AnimationGraphCompile"),
            "WorkbenchExtension/AnimationGraphCompile"
        );
        assert_eq!(binding_key("", ""), "/");
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260831gs_editor574_binding_key_single_buffer_benchmark() {
        const SAMPLE_PAIRS: usize = 21;
        const ITERATIONS: usize = 500_000;
        const VIEW_ID: &str = "WorkbenchExtension";
        const CONTROL_ID: &str = "DiagnosticsObservabilityCaptureFrame";
        let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut checksum = 0usize;
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                let (elapsed, value) = measure(ITERATIONS, VIEW_ID, CONTROL_ID, legacy_binding_key);
                legacy_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) = measure(ITERATIONS, VIEW_ID, CONTROL_ID, binding_key);
                optimized_ns.push(elapsed);
                checksum ^= value;
            } else {
                let (elapsed, value) = measure(ITERATIONS, VIEW_ID, CONTROL_ID, binding_key);
                optimized_ns.push(elapsed);
                checksum ^= value;
                let (elapsed, value) = measure(ITERATIONS, VIEW_ID, CONTROL_ID, legacy_binding_key);
                legacy_ns.push(elapsed);
                checksum ^= value;
            }
        }
        let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
        let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "single-buffer binding key P95 must be at least 15% below format: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "EDITOR574_BINDING_KEY_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} checksum={checksum} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
            join_samples(&legacy_ns),
            join_samples(&optimized_ns),
        );

        fn measure(
            iterations: usize,
            view_id: &str,
            control_id: &str,
            operation: fn(&str, &str) -> String,
        ) -> (u128, usize) {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..iterations {
                checksum = checksum
                    .wrapping_add(operation(black_box(view_id), black_box(control_id)).len());
            }
            (started.elapsed().as_nanos(), black_box(checksum))
        }

        fn legacy_binding_key(view_id: &str, control_id: &str) -> String {
            format!("{view_id}/{control_id}")
        }

        fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
            let mut ordered = samples.to_vec();
            ordered.sort_unstable();
            let rank = (ordered.len() * percentile).div_ceil(100).max(1);
            ordered[rank - 1]
        }

        fn join_samples(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }
    }
}
