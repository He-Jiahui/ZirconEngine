use std::time::Instant;

use super::*;

use crate::core::runtime::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
};

const ACTIONS: usize = 1_000;
const TEXT_SCALES: [usize; 3] = [1, 100, 10_000];
const MAX_EDIT_STATE_PROPERTIES: usize = 10;
const MAX_BINDING_UPDATES_PER_ACTION: usize = 20;

#[derive(Clone, Copy, Debug)]
enum AccessibilityTextScaleAction {
    SetValue,
    ReplaceSelectedText,
    SetTextSelection,
}

impl AccessibilityTextScaleAction {
    const fn label(self) -> &'static str {
        match self {
            Self::SetValue => "set_value",
            Self::ReplaceSelectedText => "replace_selected_text",
            Self::SetTextSelection => "set_text_selection",
        }
    }

    const fn changes_text(self) -> bool {
        matches!(self, Self::SetValue | Self::ReplaceSelectedText)
    }
}

fn profile_counter_total(
    profile: &crate::core::runtime::diagnostics::profiling::ProfileSnapshot,
    name: &str,
) -> f64 {
    profile
        .counters
        .iter()
        .filter(|counter| counter.stream == "runtime" && counter.name == name)
        .map(|counter| counter.value)
        .sum()
}

fn profile_span_count(
    profile: &crate::core::runtime::diagnostics::profiling::ProfileSnapshot,
    category: &str,
    name: &str,
) -> usize {
    profile
        .spans
        .iter()
        .filter(|span| span.stream == "runtime" && span.category == category && span.name == name)
        .count()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

fn reset_replace_selection(surface: &mut UiSurface, text_len: usize) {
    let metadata = surface
        .tree
        .node_mut(id(2))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("scale fixture must retain text input metadata");
    metadata.attributes.insert(
        "caret_offset".to_string(),
        toml::Value::Integer(text_len as i64),
    );
    metadata
        .attributes
        .insert("selection_anchor".to_string(), toml::Value::Integer(0));
    metadata.attributes.insert(
        "selection_focus".to_string(),
        toml::Value::Integer(text_len as i64),
    );
}

#[test]
#[ignore = "PERF-MVP-258 scale evidence; run through the managed Windows validator"]
fn accessibility_text_action_atomic_transaction_scale_evidence() {
    let _capture_guard = test_capture_lock();

    for text_len in TEXT_SCALES {
        let source_text = "a".repeat(text_len);
        let value_b = "b".repeat(text_len);
        let value_c = "c".repeat(text_len);
        for action in [
            AccessibilityTextScaleAction::SetValue,
            AccessibilityTextScaleAction::ReplaceSelectedText,
            AccessibilityTextScaleAction::SetTextSelection,
        ] {
            let mut surface = root_surface();
            insert_text_input(
                &mut surface,
                &format!(
                    "text = \"{source_text}\"\ncaret_offset = 0\nselection_anchor = 0\nselection_focus = 0"
                ),
            );
            surface.rebuild();
            surface.clear_dirty_flags();

            let mut config = ProfileCaptureConfig::default();
            config.session_id = format!("accessibility-text-{}-{}", text_len, action.label());
            config.max_frames = 1;
            config.max_spans = 131_072;
            config.max_counters = 32_768;
            config.include_perfetto = false;
            assert!(start_capture(config).active);

            let mut durations_ns = Vec::with_capacity(ACTIONS);
            let mut changed_property_writes = 0usize;
            let mut binding_reports = 0usize;
            let mut binding_updates = 0usize;
            let mut dirty_commits = 0usize;
            let mut text_layout_revision_increments = 0usize;

            for index in 0..ACTIONS {
                if matches!(action, AccessibilityTextScaleAction::ReplaceSelectedText) {
                    // Fixture-only reset; it is outside the measured accessibility action.
                    reset_replace_selection(&mut surface, text_len);
                }
                surface.clear_dirty_flags();
                let revision_before = text_layout_revision(&surface);
                let started = Instant::now();
                let result = match action {
                    AccessibilityTextScaleAction::SetValue => dispatch_set_value(
                        &mut surface,
                        if index % 2 == 0 { &value_b } else { &value_c },
                    ),
                    AccessibilityTextScaleAction::ReplaceSelectedText => {
                        dispatch_replace_selected_text(
                            &mut surface,
                            if index % 2 == 0 { &value_b } else { &value_c },
                        )
                    }
                    AccessibilityTextScaleAction::SetTextSelection => dispatch_set_text_selection(
                        &mut surface,
                        if index % 2 == 0 {
                            UiA11yTextSelection {
                                caret: text_len,
                                anchor: 0,
                                focus: text_len,
                            }
                        } else {
                            UiA11yTextSelection::collapsed(0)
                        },
                    ),
                };
                durations_ns.push(started.elapsed().as_nanos().max(1));
                assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);

                let changed = result
                    .diagnostics
                    .notes
                    .iter()
                    .filter(|note| note.starts_with("accessibility_text_state_changed:"))
                    .count();
                assert!(
                    changed <= MAX_EDIT_STATE_PROPERTIES,
                    "one typed transaction may expose at most {MAX_EDIT_STATE_PROPERTIES} fields"
                );
                assert!(result.binding_reports.len() <= 1);
                changed_property_writes += changed;
                binding_reports += result.binding_reports.len();
                binding_updates += result
                    .binding_reports
                    .iter()
                    .map(|report| report.updates.len())
                    .sum::<usize>();
                if surface.dirty_flags().any() {
                    dirty_commits += 1;
                }
                text_layout_revision_increments +=
                    text_layout_revision(&surface).saturating_sub(revision_before) as usize;
            }

            let profile = snapshot();
            assert!(!reset_capture().active);
            let prepare_transactions =
                profile_span_count(&profile, "ui_text.edit", "property_prepare");
            let commit_transactions =
                profile_span_count(&profile, "ui_text.edit", "property_commit");
            let value_clones =
                profile_counter_total(&profile, "ui_text.edit.property_value_clones");
            let copied_bytes =
                profile_counter_total(&profile, "ui_text.edit.property_value_clone_bytes");
            let projected_bytes =
                profile_counter_total(&profile, "ui_text.edit.property_projected_bytes");
            let state_materializations =
                profile_counter_total(&profile, "ui_text.edit.state_materializations");
            let p95_ns = nearest_rank_p95(&mut durations_ns);
            let retention = profile
                .recorder_retention
                .first()
                .expect("scale capture must retain recorder capacity evidence");

            println!(
                "PERF-MVP-258-ATOMIC-TEXT-SCALE chars={text_len} action={} actions={ACTIONS} mutation_transactions={commit_transactions} transaction_prepare={prepare_transactions} changed_property_writes={changed_property_writes} binding_reports={binding_reports} binding_updates={binding_updates} dirty_commits={dirty_commits} text_layout_revision_increments={text_layout_revision_increments} state_materializations={state_materializations} property_value_clones={value_clones} property_value_clone_bytes={copied_bytes} property_projected_bytes={projected_bytes} cpu_p95_ns={p95_ns} span_overwritten={} counter_overwritten={} replace_selection_reset_outside_action={}",
                action.label(),
                retention.spans.overwritten,
                retention.counters.overwritten,
                matches!(action, AccessibilityTextScaleAction::ReplaceSelectedText),
            );

            let expected_text_invalidations = if action.changes_text() { ACTIONS } else { 0 };
            let expected_copy_bytes = (ACTIONS * text_len) as f64;
            assert_eq!(retention.spans.overwritten, 0);
            assert_eq!(retention.counters.overwritten, 0);
            assert_eq!(prepare_transactions, ACTIONS);
            assert_eq!(commit_transactions, ACTIONS);
            assert_eq!(binding_reports, ACTIONS);
            assert!(changed_property_writes <= ACTIONS * MAX_EDIT_STATE_PROPERTIES);
            assert!(binding_updates <= ACTIONS * MAX_BINDING_UPDATES_PER_ACTION);
            assert_eq!(dirty_commits, ACTIONS);
            assert_eq!(text_layout_revision_increments, expected_text_invalidations);
            assert_eq!(value_clones, ACTIONS as f64);
            assert_eq!(copied_bytes, expected_copy_bytes);
            assert_eq!(projected_bytes, expected_copy_bytes);
        }
    }
}
