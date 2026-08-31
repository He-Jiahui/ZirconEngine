use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiComponentKeyboardAction, UiValue},
    event_ui::UiNodeId,
    tree::UiTreeError,
    widget::UiWidgetBehavior,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};

use super::{UiDefaultKeyboardActionReport, widget_behavior};

impl UiSurface {
    pub(crate) fn apply_default_keyboard_component_action(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<UiDefaultKeyboardActionReport, UiTreeError> {
        let Some(behavior) = self.default_keyboard_behavior(node_id)? else {
            return Ok(UiDefaultKeyboardActionReport::default());
        };

        match behavior {
            UiWidgetBehavior::Button | UiWidgetBehavior::MenuItem => {
                let mut binding_reports = Vec::new();
                let event = UiComponentEvent::Commit {
                    property: "activated".to_string(),
                    value: UiValue::Bool(true),
                };
                let component_events = self.component_event_reports_for_bindings(
                    node_id,
                    UiEventKind::Click,
                    event,
                    false,
                )?;
                let component_events = if behavior == UiWidgetBehavior::MenuItem {
                    self.with_default_menu_item_popup_close_reports(
                        node_id,
                        component_events,
                        &mut binding_reports,
                    )?
                } else {
                    component_events
                };
                Ok(UiDefaultKeyboardActionReport {
                    handled: !component_events.is_empty(),
                    component_events,
                    binding_reports,
                })
            }
            UiWidgetBehavior::Toggle => {
                let Some(next_checked) = self.default_toggle_next_checked(node_id)? else {
                    return Ok(UiDefaultKeyboardActionReport::default());
                };
                let property = self.default_toggle_checked_property(node_id)?;
                let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
                    node_id,
                    property.clone(),
                    UiValue::Bool(next_checked),
                ))?;
                if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
                    return Ok(UiDefaultKeyboardActionReport::default());
                }
                let binding_reports = vec![report.binding];
                let event = UiComponentEvent::ValueChanged {
                    property,
                    value: UiValue::Bool(next_checked),
                };
                let component_events = self.component_event_reports_for_bindings(
                    node_id,
                    UiEventKind::Change,
                    event,
                    true,
                )?;
                Ok(UiDefaultKeyboardActionReport {
                    handled: true,
                    component_events,
                    binding_reports,
                })
            }
            UiWidgetBehavior::Radio => self.apply_default_radio_keyboard_action(node_id),
            UiWidgetBehavior::Disclosure => {
                let Some(next_expanded) = self.default_expanded_next(node_id)? else {
                    return Ok(UiDefaultKeyboardActionReport::default());
                };
                let property = self.default_open_property(node_id, "expanded")?;
                let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
                    node_id,
                    property,
                    UiValue::Bool(next_expanded),
                ))?;
                if !matches!(report.status, UiPropertyMutationStatus::Accepted) {
                    return Ok(UiDefaultKeyboardActionReport::default());
                }
                let binding_reports = vec![report.binding];
                let event = UiComponentEvent::ToggleExpanded {
                    expanded: next_expanded,
                };
                let component_events = self.component_event_reports_for_bindings(
                    node_id,
                    UiEventKind::Toggle,
                    event,
                    true,
                )?;
                Ok(UiDefaultKeyboardActionReport {
                    handled: true,
                    component_events,
                    binding_reports,
                })
            }
            UiWidgetBehavior::Popup => self.apply_default_popup_keyboard_action(node_id),
            UiWidgetBehavior::Auto
            | UiWidgetBehavior::Passive
            | UiWidgetBehavior::RadioGroup
            | UiWidgetBehavior::Range
            | UiWidgetBehavior::Scrollbar
            | UiWidgetBehavior::ScrollbarThumb
            | UiWidgetBehavior::TextInput => Ok(UiDefaultKeyboardActionReport::default()),
        }
    }

    pub(crate) fn apply_default_semantic_keyboard_component_action(
        &mut self,
        node_id: UiNodeId,
        action: UiComponentKeyboardAction,
    ) -> Result<UiDefaultKeyboardActionReport, UiTreeError> {
        let Some(behavior) = self.default_keyboard_behavior(node_id)? else {
            return Ok(UiDefaultKeyboardActionReport::default());
        };
        let action = semantic_keyboard_action_for_behavior(action, behavior);
        let event = UiComponentEvent::KeyboardAction { action };
        let mut component_events = None;
        for event_kind in semantic_keyboard_event_kinds(action) {
            let batch = self.component_event_reports_for_bindings(
                node_id,
                *event_kind,
                event.clone(),
                true,
            )?;
            append_or_adopt_batch(&mut component_events, batch);
        }
        let component_events = component_events.unwrap_or_default();
        Ok(UiDefaultKeyboardActionReport {
            handled: !component_events.is_empty(),
            component_events,
            binding_reports: Vec::new(),
        })
    }

    pub(crate) fn apply_default_semantic_keyboard_component_text(
        &mut self,
        node_id: UiNodeId,
        text: &str,
    ) -> Result<UiDefaultKeyboardActionReport, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(UiDefaultKeyboardActionReport::default());
        };
        if !self.widget_interaction_enabled(node_id, node, metadata) {
            return Ok(UiDefaultKeyboardActionReport::default());
        }

        let event = UiComponentEvent::KeyboardText {
            text: text.to_string(),
        };
        let component_events =
            self.component_event_reports_for_bindings(node_id, UiEventKind::Change, event, true)?;
        Ok(UiDefaultKeyboardActionReport {
            handled: !component_events.is_empty(),
            component_events,
            binding_reports: Vec::new(),
        })
    }

    fn default_keyboard_behavior(
        &self,
        node_id: UiNodeId,
    ) -> Result<Option<UiWidgetBehavior>, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let Some(metadata) = node.template_metadata.as_ref() else {
            return Ok(None);
        };
        if !self.widget_interaction_enabled(node_id, node, metadata) {
            return Ok(None);
        }
        Ok(Some(widget_behavior(metadata)))
    }
}

fn semantic_keyboard_action_for_behavior(
    action: UiComponentKeyboardAction,
    behavior: UiWidgetBehavior,
) -> UiComponentKeyboardAction {
    if !matches!(
        behavior,
        UiWidgetBehavior::Range | UiWidgetBehavior::Scrollbar | UiWidgetBehavior::ScrollbarThumb
    ) {
        return action;
    }

    match action {
        UiComponentKeyboardAction::Next => UiComponentKeyboardAction::Increment,
        UiComponentKeyboardAction::Previous => UiComponentKeyboardAction::Decrement,
        _ => action,
    }
}

fn semantic_keyboard_event_kinds(action: UiComponentKeyboardAction) -> &'static [UiEventKind] {
    match action {
        UiComponentKeyboardAction::Activate | UiComponentKeyboardAction::Cancel => &[
            UiEventKind::Click,
            UiEventKind::Change,
            UiEventKind::Toggle,
            UiEventKind::Submit,
        ],
        UiComponentKeyboardAction::Next
        | UiComponentKeyboardAction::Previous
        | UiComponentKeyboardAction::First
        | UiComponentKeyboardAction::Last
        | UiComponentKeyboardAction::Increment
        | UiComponentKeyboardAction::Decrement
        | UiComponentKeyboardAction::LargeIncrement
        | UiComponentKeyboardAction::LargeDecrement
        | UiComponentKeyboardAction::BeginEdit => &[UiEventKind::Change],
    }
}

fn append_or_adopt_batch<T>(output: &mut Option<Vec<T>>, batch: Vec<T>) {
    if batch.is_empty() {
        return;
    }
    if let Some(output) = output {
        output.extend(batch);
    } else {
        *output = Some(batch);
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::append_or_adopt_batch;

    #[test]
    fn optimization_batch_dk_keyboard_event_batch_adoption_preserves_order() {
        let mut output = None;

        append_or_adopt_batch(&mut output, Vec::<u32>::new());
        append_or_adopt_batch(&mut output, vec![1, 2]);
        append_or_adopt_batch(&mut output, vec![3, 4]);

        assert_eq!(output, Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn optimization_batch_dk_semantic_keyboard_adopts_first_batch_source() {
        let source = include_str!("keyboard.rs");
        let function = source
            .split("pub(crate) fn apply_default_semantic_keyboard_component_action")
            .nth(1)
            .expect("semantic keyboard action")
            .split("pub(crate) fn apply_default_semantic_keyboard_component_text")
            .next()
            .expect("semantic action body");

        assert!(function.contains("let mut component_events = None;"));
        assert!(function.contains("append_or_adopt_batch"));
        assert!(!function.contains("let mut component_events = Vec::new();"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dk_adopt_first_keyboard_event_batch_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const MERGES_PER_SAMPLE: usize = 32_768;
        const REPORTS_PER_BATCH: usize = 32;

        let template = (0..REPORTS_PER_BATCH as u64).collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_batch_adoption(&template, MERGES_PER_SAMPLE, true));
                optimized_samples.push(measure_batch_adoption(&template, MERGES_PER_SAMPLE, false));
            } else {
                optimized_samples.push(measure_batch_adoption(&template, MERGES_PER_SAMPLE, false));
                legacy_samples.push(measure_batch_adoption(&template, MERGES_PER_SAMPLE, true));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME419_ADOPT_FIRST_KEYBOARD_EVENT_BATCH_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "adopted keyboard event batch p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_batch_adoption(template: &[u64], merges: usize, legacy: bool) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..merges {
            let batch = black_box(template).to_vec();
            let output = if legacy {
                let mut output = Vec::new();
                output.extend(batch);
                output
            } else {
                let mut output = None;
                append_or_adopt_batch(&mut output, batch);
                output.expect("non-empty batch is adopted")
            };
            checksum = checksum.wrapping_add(output.len() as u64);
            black_box(output);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = samples
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        samples[index]
    }
}
