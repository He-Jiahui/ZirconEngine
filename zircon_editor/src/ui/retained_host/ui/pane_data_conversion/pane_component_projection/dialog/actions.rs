use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;

pub(in super::super) fn projected_dialog_actions(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneActionData> {
    match component_role {
        "dialog" => dialog_actions(attributes),
        "confirm-dialog" | "alert-dialog" => confirm_dialog_actions(attributes),
        _ => Vec::new(),
    }
}

fn dialog_actions(
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneActionData> {
    let Some(label) = first_non_empty_attribute(
        attributes,
        &[
            "action",
            "primary_action_text",
            "confirm_text",
            "close_text",
        ],
    ) else {
        return Vec::new();
    };
    let action_id = first_non_empty_attribute(
        attributes,
        &["dialog_action_id", "action_id", "commit_action_id"],
    )
    .unwrap_or_default();
    vec![host_contract::TemplatePaneActionData {
        label: label.into(),
        action_id: action_id.into(),
    }]
}

fn confirm_dialog_actions(
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneActionData> {
    let cancel_label =
        first_non_empty_attribute(attributes, &["cancel_text", "cancelText", "close_text"])
            .unwrap_or("Cancel");
    let confirm_label = first_non_empty_attribute(
        attributes,
        &[
            "confirm_text",
            "confirmText",
            "primary_action_text",
            "action",
        ],
    )
    .unwrap_or("Confirm");

    vec![
        host_contract::TemplatePaneActionData {
            label: cancel_label.into(),
            action_id: first_non_empty_attribute(
                attributes,
                &["cancel_action_id", "cancelActionId"],
            )
            .unwrap_or("cancel")
            .into(),
        },
        host_contract::TemplatePaneActionData {
            label: confirm_label.into(),
            action_id: first_non_empty_attribute(
                attributes,
                &["confirm_action_id", "confirmActionId", "dialog_action_id"],
            )
            .unwrap_or("confirm")
            .into(),
        },
    ]
}

fn first_non_empty_attribute<'a>(
    attributes: &'a BTreeMap<String, toml::Value>,
    names: &[&str],
) -> Option<&'a str> {
    names
        .iter()
        .filter_map(|name| attributes.get(*name))
        .filter_map(toml::Value::as_str)
        .find(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::{confirm_dialog_actions, dialog_actions, first_non_empty_attribute};

    #[test]
    fn optimization_batch_dp_dialog_actions_preserve_labels_and_defaults() {
        let attributes = BTreeMap::from([
            (
                "cancel_text".to_string(),
                toml::Value::String("Back".to_string()),
            ),
            (
                "confirm_text".to_string(),
                toml::Value::String("Apply".to_string()),
            ),
            (
                "confirm_action_id".to_string(),
                toml::Value::String("apply_changes".to_string()),
            ),
        ]);
        let actions = confirm_dialog_actions(&attributes);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].label, "Back");
        assert_eq!(actions[0].action_id, "cancel");
        assert_eq!(actions[1].label, "Apply");
        assert_eq!(actions[1].action_id, "apply_changes");

        let empty = BTreeMap::new();
        assert!(dialog_actions(&empty).is_empty());
        assert_eq!(
            first_non_empty_attribute(&attributes, &["missing", "cancel_text"]),
            Some("Back")
        );
    }

    #[test]
    fn optimization_batch_dp_dialog_actions_borrow_attribute_values() {
        let source = include_str!("actions.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("dialog actions production source");
        assert!(production.contains("toml::Value::as_str"));
        assert!(!production.contains("first_non_empty_string_attribute"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dp_dialog_attribute_borrow_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const LOOKUPS_PER_SAMPLE: usize = 8_192;
        let attributes = benchmark_attributes();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_attribute_lookups(
                    &attributes,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
                optimized_samples.push(measure_attribute_lookups(
                    &attributes,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
            } else {
                optimized_samples.push(measure_attribute_lookups(
                    &attributes,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
                legacy_samples.push(measure_attribute_lookups(
                    &attributes,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR352_DIALOG_ATTRIBUTE_BORROW_BENCH_V1 lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "dialog attribute borrow p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_attribute_lookups(
            attributes: &BTreeMap<String, toml::Value>,
            lookup_count: usize,
            optimized: bool,
        ) -> u128 {
            let started_at = Instant::now();
            let mut checksum = 0_usize;
            for _ in 0..lookup_count {
                let values = if optimized {
                    [
                        first_non_empty_attribute(attributes, &["cancel_text", "cancelText"])
                            .unwrap_or("Cancel")
                            .len(),
                        first_non_empty_attribute(attributes, &["confirm_text", "action"])
                            .unwrap_or("Confirm")
                            .len(),
                    ]
                } else {
                    [
                        super::super::super::attribute_values::first_non_empty_string_attribute(
                            attributes,
                            &["cancel_text", "cancelText"],
                        )
                        .unwrap_or_else(|| "Cancel".to_string())
                        .len(),
                        super::super::super::attribute_values::first_non_empty_string_attribute(
                            attributes,
                            &["confirm_text", "action"],
                        )
                        .unwrap_or_else(|| "Confirm".to_string())
                        .len(),
                    ]
                };
                checksum = checksum.wrapping_add(values[0] ^ values[1]);
            }
            black_box(checksum);
            started_at.elapsed().as_nanos()
        }

        fn benchmark_attributes() -> BTreeMap<String, toml::Value> {
            BTreeMap::from([
                (
                    "cancel_text".to_string(),
                    toml::Value::String("Cancel changes".to_string()),
                ),
                (
                    "confirm_text".to_string(),
                    toml::Value::String("Apply changes".to_string()),
                ),
                (
                    "action".to_string(),
                    toml::Value::String("fallback".to_string()),
                ),
            ])
        }

        fn p95(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
        }
    }
}
