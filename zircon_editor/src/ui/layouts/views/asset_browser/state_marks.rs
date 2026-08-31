use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn mark_toggle_state(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    active: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = false;
        assign_shared_string_if_changed(
            &mut node.surface_variant,
            if active { "inset" } else { "" },
        );
        assign_shared_string_if_changed(
            &mut node.text_tone,
            if active { "default" } else { "subtle" },
        );
    }
}

pub(super) fn mark_utility_tab_state(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    active: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = false;
        assign_shared_string_if_changed(&mut node.surface_variant, "");
        assign_shared_string_if_changed(
            &mut node.text_tone,
            if active { "default" } else { "subtle" },
        );
    }
}

fn assign_shared_string_if_changed(target: &mut SharedString, value: &str) {
    if target.as_str() != value {
        *target = value.into();
    }
}

pub(super) fn mark_panel_selected(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    selected: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = selected;
        node.focused = false;
    }
}

pub(super) fn mark_panel_group_selected(
    nodes: &mut [ViewTemplateNodeData],
    control_ids: &[&str],
    selected: bool,
) {
    for control_id in control_ids {
        mark_panel_selected(nodes, control_id, selected);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotent_state_marks_preserve_toggle_and_utility_values() {
        let mut nodes = vec![state_node("toggle"), state_node("utility")];

        mark_toggle_state(&mut nodes, "toggle", true);
        assert!(nodes[0].selected);
        assert!(!nodes[0].focused);
        assert_eq!(nodes[0].surface_variant.as_str(), "inset");
        assert_eq!(nodes[0].text_tone.as_str(), "default");

        mark_toggle_state(&mut nodes, "toggle", false);
        assert!(!nodes[0].selected);
        assert_eq!(nodes[0].surface_variant.as_str(), "");
        assert_eq!(nodes[0].text_tone.as_str(), "subtle");

        mark_utility_tab_state(&mut nodes, "utility", true);
        assert!(nodes[1].selected);
        assert!(!nodes[1].focused);
        assert_eq!(nodes[1].surface_variant.as_str(), "");
        assert_eq!(nodes[1].text_tone.as_str(), "default");
    }

    #[test]
    fn idempotent_state_marks_keep_shared_strings_when_unchanged() {
        let mut nodes = vec![state_node("toggle")];
        mark_toggle_state(&mut nodes, "toggle", true);
        let surface_pointer = nodes[0].surface_variant.as_str().as_ptr();
        let tone_pointer = nodes[0].text_tone.as_str().as_ptr();

        mark_toggle_state(&mut nodes, "toggle", true);

        assert_eq!(nodes[0].surface_variant.as_str().as_ptr(), surface_pointer);
        assert_eq!(nodes[0].text_tone.as_str().as_ptr(), tone_pointer);
    }

    #[test]
    fn idempotent_state_marks_use_conditional_shared_string_writes() {
        let source = include_str!("state_marks.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");

        assert!(implementation.contains("fn assign_shared_string_if_changed"));
        assert!(
            implementation
                .matches("assign_shared_string_if_changed(")
                .count()
                >= 5
        );
        assert!(!implementation.contains("node.surface_variant = if active"));
        assert!(!implementation.contains("node.text_tone = if active"));
    }

    #[test]
    #[ignore = "release performance benchmark"]
    fn idempotent_state_marks_release_benchmark() {
        const SAMPLES: usize = 11;
        const ITERATIONS: usize = 16_384;
        const RETIRED_SHARED_STRING_WRITES: usize = 2;
        const OPTIMIZED_SHARED_STRING_WRITES: usize = 0;

        let mut retired_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            let benchmark = |mark: fn(&mut [ViewTemplateNodeData], &str, bool)| {
                let mut nodes = vec![state_node("toggle")];
                mark(&mut nodes, "toggle", true);
                let started = std::time::Instant::now();
                for _ in 0..ITERATIONS {
                    mark(&mut nodes, "toggle", true);
                    std::hint::black_box(nodes[0].surface_variant.as_str().as_ptr());
                    std::hint::black_box(nodes[0].text_tone.as_str().as_ptr());
                }
                started.elapsed().as_nanos()
            };

            if sample % 2 == 0 {
                retired_samples.push(benchmark(retired_mark_toggle_state));
                optimized_samples.push(benchmark(mark_toggle_state));
            } else {
                optimized_samples.push(benchmark(mark_toggle_state));
                retired_samples.push(benchmark(retired_mark_toggle_state));
            }
        }

        let retired_p95_ns = percentile_95(&mut retired_samples);
        let optimized_p95_ns = percentile_95(&mut optimized_samples);
        let reduction_bps = retired_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(10_000)
            / retired_p95_ns.max(1);
        println!(
            "EDITOR57_IDEMPOTENT_STATE_MARKS_BENCH_V1 \
             retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             reduction_bps={reduction_bps} samples={SAMPLES} iterations={ITERATIONS} \
             repeated_shared_string_writes={RETIRED_SHARED_STRING_WRITES}->{OPTIMIZED_SHARED_STRING_WRITES}"
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(45),
            "optimized P95 must be at least 55% faster: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn retired_mark_toggle_state(
        nodes: &mut [ViewTemplateNodeData],
        control_id: &str,
        active: bool,
    ) {
        if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
            node.selected = active;
            node.focused = false;
            node.surface_variant = if active { "inset".into() } else { "".into() };
            node.text_tone = if active {
                "default".into()
            } else {
                "subtle".into()
            };
        }
    }

    fn state_node(control_id: &str) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            control_id: control_id.into(),
            ..ViewTemplateNodeData::default()
        }
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
        samples[index]
    }
}
