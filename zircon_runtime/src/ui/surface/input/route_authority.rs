use zircon_runtime_interface::ui::dispatch::{UiInputDispatchResult, UiInputRoutePolicy};

use crate::ui::dispatch::route_stage_names_for_policy;

pub(super) const UI_INPUT_ROUTE_AUTHORITY_ANCHOR: &str = "runtime_09_m1_1_ui_input_route_authority";
pub(super) const UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX: &str = "route_authority=";

pub(super) fn annotate_authoritative_input_dispatch(result: &mut UiInputDispatchResult) {
    let policy = result.diagnostics.route_policy;
    replace_route_authority_note(
        &mut result.diagnostics.notes,
        route_authority_note_for_policy(policy),
    );
}

fn replace_route_authority_note(notes: &mut Vec<String>, authoritative_note: &str) {
    if let Some(last) = notes.last_mut() {
        if last.starts_with(UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX) {
            last.clear();
            last.push_str(authoritative_note);
            return;
        }
    }

    notes.retain(|note| !note.starts_with(UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX));
    notes.push(authoritative_note.to_string());
}

const fn route_authority_note_for_policy(policy: UiInputRoutePolicy) -> &'static str {
    match policy {
        UiInputRoutePolicy::Unrouted => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=unrouted;stages="
        }
        UiInputRoutePolicy::PreviewTunnel => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=preview_tunnel;stages=popup_stack>preview_tunnel"
        }
        UiInputRoutePolicy::Bubble => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=bubble;stages=popup_stack>preview_tunnel>direct_target>bubble_path"
        }
        UiInputRoutePolicy::Direct => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=direct;stages=direct_target"
        }
        UiInputRoutePolicy::FocusPath => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=focus_path;stages=popup_stack>focus_path"
        }
        UiInputRoutePolicy::PointerCapture => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=pointer_capture;stages=pointer_capture"
        }
        UiInputRoutePolicy::DefaultAction => {
            "route_authority=runtime_09_m1_1_ui_input_route_authority;policy=default_action;stages=popup_stack>default_action"
        }
    }
}

pub(super) fn route_authority_stage_names_for_policy(
    policy: UiInputRoutePolicy,
) -> Vec<&'static str> {
    route_stage_names_for_policy(policy)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_COUNT: usize = 17;
    const ITERATIONS: usize = 32_768;

    fn legacy_replace_route_authority_note(notes: &mut Vec<String>, authoritative_note: &str) {
        notes.retain(|note| !note.starts_with(UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX));
        notes.push(authoritative_note.to_string());
    }

    fn percentile_95(mut samples: Vec<u128>) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100) - 1]
    }

    #[test]
    fn optimization_batch_hi_runtime593_replaces_terminal_authority_note_in_place() {
        let bubble = route_authority_note_for_policy(UiInputRoutePolicy::Bubble);
        let direct = route_authority_note_for_policy(UiInputRoutePolicy::Direct);
        let mut notes = vec!["input_source=pointer".to_string(), bubble.to_string()];
        let original_allocation = notes[1].as_ptr();

        replace_route_authority_note(&mut notes, direct);

        assert_eq!(notes, ["input_source=pointer", direct]);
        assert_eq!(notes[1].as_ptr(), original_allocation);
    }

    #[test]
    fn optimization_batch_hi_runtime593_preserves_non_authority_note_order() {
        let direct = route_authority_note_for_policy(UiInputRoutePolicy::Direct);
        let mut notes = vec![
            format!("{UI_INPUT_ROUTE_AUTHORITY_NOTE_PREFIX}stale"),
            "first".to_string(),
            "second".to_string(),
        ];

        replace_route_authority_note(&mut notes, direct);

        assert_eq!(notes, ["first", "second", direct]);
    }

    #[test]
    fn optimization_batch_hi_runtime593_terminal_authority_source_contract() {
        let production = include_str!("route_authority.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();

        assert!(production.contains("if let Some(last) = notes.last_mut()"));
        assert!(production.contains("last.clear();"));
    }

    #[test]
    #[ignore = "Windows-native release performance evidence"]
    fn optimization_batch_hi_runtime593_terminal_authority_bench() {
        let bubble = route_authority_note_for_policy(UiInputRoutePolicy::Bubble);
        let direct = route_authority_note_for_policy(UiInputRoutePolicy::Direct);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);

        for sample in 0..SAMPLE_COUNT {
            let measure_legacy = || {
                let mut notes = vec!["input_source=pointer".to_string(), bubble.to_string()];
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    legacy_replace_route_authority_note(black_box(&mut notes), black_box(direct));
                }
                black_box(notes);
                started.elapsed().as_nanos()
            };
            let measure_optimized = || {
                let mut notes = vec!["input_source=pointer".to_string(), bubble.to_string()];
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    replace_route_authority_note(black_box(&mut notes), black_box(direct));
                }
                black_box(notes);
                started.elapsed().as_nanos()
            };
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p95 = percentile_95(legacy_samples);
        let optimized_p95 = percentile_95(optimized_samples);
        println!(
            "RUNTIME593_TERMINAL_ROUTE_AUTHORITY_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} replacement_allocations_per_iteration=1->0",
            legacy_p95, optimized_p95, SAMPLE_COUNT, ITERATIONS,
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(35),
            "optimized P95 must be at most 35% of legacy P95"
        );
    }
}
