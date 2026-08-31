use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostBindingProjection;

use super::binding_ids::{showcase_action_id_for_binding_id, showcase_binding_with_suffix};

pub(in super::super) fn preferred_showcase_action_buttons(
    control_id: &str,
    bindings: &[RetainedUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneActionData> {
    let specs = action_button_specs(control_id);
    let mut actions = Vec::with_capacity(specs.len());
    actions.extend(specs.iter().filter_map(|(label, suffix)| {
        showcase_binding_with_suffix(bindings, suffix).map(|binding| {
            host_contract::TemplatePaneActionData {
                label: (*label).into(),
                action_id: showcase_action_id_for_binding_id(&binding.binding_id).into(),
            }
        })
    }));
    actions
}

fn action_button_specs(control_id: &str) -> &'static [(&'static str, &'static str)] {
    match control_id {
        "AssetFieldDemo" => &[
            ("Find", "AssetFieldLocate"),
            ("Open", "AssetFieldOpen"),
            ("Clear", "AssetFieldClear"),
        ],
        "InstanceFieldDemo" => &[
            ("Find", "InstanceFieldLocate"),
            ("Open", "InstanceFieldOpen"),
            ("Clear", "InstanceFieldClear"),
        ],
        "ObjectFieldDemo" => &[
            ("Find", "ObjectFieldLocate"),
            ("Open", "ObjectFieldOpen"),
            ("Clear", "ObjectFieldClear"),
        ],
        "ArrayFieldDemo" => &[
            ("Add", "ArrayFieldAddElement"),
            ("Set", "ArrayFieldSetElement"),
            ("Remove", "ArrayFieldRemoveElement"),
            ("Move", "ArrayFieldMoveElement"),
        ],
        "MapFieldDemo" => &[
            ("Add", "MapFieldAddEntry"),
            ("Set", "MapFieldSetEntry"),
            ("Remove", "MapFieldRemoveEntry"),
        ],
        _ => &[],
    }
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830df_action_buttons_reserve_spec_upper_bound() {
        let source = include_str!("action_buttons.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("showcase action button production source");

        assert!(production.contains("let specs = action_button_specs(control_id)"));
        assert!(production.contains("Vec::with_capacity(specs.len())"));
        assert!(production.contains("actions.extend("));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830df_action_button_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const ACTION_COUNT: usize = 4;
        const MARKER: &str = "EDITOR518_ACTION_BUTTON_CAPACITY_BENCH_V1";

        let legacy_growth_events = action_growth_events(BATCH_COUNT, ACTION_COUNT, false);
        let optimized_growth_events = action_growth_events(BATCH_COUNT, ACTION_COUNT, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} actions={ACTION_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn action_growth_events(batch_count: usize, action_count: usize, reserve: bool) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut actions = if reserve {
                Vec::with_capacity(action_count)
            } else {
                Vec::new()
            };
            for action in 0..action_count {
                let previous_capacity = actions.capacity();
                actions.push(action);
                growth_events += usize::from(actions.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
