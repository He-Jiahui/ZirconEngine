use std::collections::{BTreeMap, HashSet};

use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::option_spec::{parse_retained_option, RetainedOptionSpec};
use zircon_runtime_interface::ui::component::UiValue;

use super::pane_value_conversion::value_as_options;

pub(in crate::ui::retained_host::ui) fn structured_options_for_node(
    options: &[String],
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneOptionData> {
    let selected = selected_option_ids(attributes);
    let disabled = option_id_set(attributes.get("disabled_options"));
    let special = option_id_set(attributes.get("special_options"));
    let focused = option_id_set(attributes.get("focused_options"));
    let hovered = option_id_set(attributes.get("hovered_options"));
    let pressed = option_id_set(attributes.get("pressed_options"));
    let loading = option_id_set(attributes.get("loading_options"));
    let focused_index = option_index(attributes.get("focused_index"));
    let hovered_option_id = option_id(attributes.get("hovered_option_id"));
    let query = attributes
        .get("query")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);

    options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let option = structured_option(option);
            let matched = option_matches_query(&option, query.as_deref());
            let selected = option.has_flag("selected")
                || option.has_flag("checked")
                || option_matches_set(&option, &selected);
            let disabled = option.has_flag("disabled") || option_matches_set(&option, &disabled);
            let special = option.has_flag("special") || option_matches_set(&option, &special);
            let focused = option.has_flag("focused")
                || option_matches_set(&option, &focused)
                || focused_index == Some(index);
            let hovered = option.has_flag("hovered")
                || option_matches_set(&option, &hovered)
                || hovered_option_id
                    .as_deref()
                    .is_some_and(|id| option.matches_id(id));
            let pressed = option.has_flag("pressed") || option_matches_set(&option, &pressed);
            let loading = option.has_flag("loading") || option_matches_set(&option, &loading);
            host_contract::TemplatePaneOptionData {
                matched,
                id: option.id.into(),
                label: option.label.into(),
                selected,
                disabled,
                special,
                focused,
                hovered,
                pressed,
                loading,
                ..host_contract::TemplatePaneOptionData::default()
            }
        })
        .collect()
}

type ProjectedOption = RetainedOptionSpec;

fn structured_option(raw: &str) -> ProjectedOption {
    parse_retained_option(raw)
}

fn option_matches_query(option: &ProjectedOption, query: Option<&str>) -> bool {
    let Some(query) = query else {
        return false;
    };
    contains_ascii_case_insensitive(&option.id, query)
        || contains_ascii_case_insensitive(&option.label, query)
        || contains_ascii_case_insensitive(&option.raw, query)
}

fn option_matches_set(option: &ProjectedOption, values: &HashSet<String>) -> bool {
    values.contains(option.id.as_str())
        || values.contains(option.label.as_str())
        || values.contains(option.raw.as_str())
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    !needle.is_empty()
        && haystack
            .as_bytes()
            .windows(needle.len())
            .any(|candidate| candidate.eq_ignore_ascii_case(needle.as_bytes()))
}

fn option_id_set(value: Option<&toml::Value>) -> HashSet<String> {
    value
        .and_then(value_as_options)
        .unwrap_or_default()
        .into_iter()
        .filter_map(normalized_option_id)
        .collect()
}

fn selected_option_ids(attributes: &BTreeMap<String, toml::Value>) -> HashSet<String> {
    ["value", "selected_options", "selectedOptions"]
        .into_iter()
        .filter_map(|key| attributes.get(key))
        .flat_map(|value| selected_option_ids_from_value(value).into_iter())
        .filter_map(normalized_option_id)
        .collect()
}

fn normalized_option_id(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn selected_option_ids_from_value(value: &toml::Value) -> HashSet<String> {
    match UiValue::from_toml(value) {
        UiValue::String(value) | UiValue::Enum(value) => HashSet::from([value]),
        UiValue::Flags(values) => values.into_iter().collect(),
        UiValue::Array(values) => values
            .into_iter()
            .map(|value| value.display_text())
            .filter(|value| !value.is_empty())
            .collect(),
        value => {
            let text = value.display_text();
            if text.is_empty() {
                HashSet::new()
            } else {
                HashSet::from([text])
            }
        }
    }
}

fn option_index(value: Option<&toml::Value>) -> Option<usize> {
    value
        .and_then(toml::Value::as_integer)
        .and_then(|value| usize::try_from(value).ok())
}

fn option_id(value: Option<&toml::Value>) -> Option<String> {
    value
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const OPTION_ID_COUNT: usize = 8_192;
    const MEMBERSHIP_LOOKUP_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn option_ids() -> Vec<String> {
        (0..OPTION_ID_COUNT)
            .map(|index| format!("editor.pane.option.generated.{index:05}"))
            .collect()
    }

    fn membership_lookups(option_ids: &[String]) -> Vec<String> {
        (0..MEMBERSHIP_LOOKUP_COUNT)
            .map(|index| option_ids[(index * 4_099) % option_ids.len()].clone())
            .collect()
    }

    fn ordered_match_count(option_ids: &[String], lookups: &[String]) -> usize {
        let values = option_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        lookups
            .iter()
            .filter(|option_id| values.contains(option_id.as_str()))
            .count()
    }

    fn hash_match_count(option_ids: &[String], lookups: &[String]) -> usize {
        let values = option_ids
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        lookups
            .iter()
            .filter(|option_id| values.contains(option_id.as_str()))
            .count()
    }

    #[test]
    fn option_query_matching_is_ascii_case_insensitive_without_normalized_row_strings() {
        assert!(contains_ascii_case_insensitive("Open Project", "open pro"));
        assert!(!contains_ascii_case_insensitive("Open Project", "save"));
    }

    #[test]
    fn option_set_matching_checks_id_label_and_raw_keys() {
        let option = structured_option("file.open|label=Open Project,focused");

        assert!(option_matches_set(
            &option,
            &HashSet::from(["Open Project".to_string()])
        ));
        assert!(!option_matches_set(
            &option,
            &HashSet::from(["file.save".to_string()])
        ));
    }

    #[test]
    fn optimization_batch_20260826z_editor01_pane_option_hash_sets_preserve_state_and_input_order()
    {
        let attributes = BTreeMap::from([
            (
                "selected_options".to_string(),
                toml::Value::Array(vec![toml::Value::String("option.a".to_string())]),
            ),
            (
                "disabled_options".to_string(),
                toml::Value::Array(vec![toml::Value::String("option.b".to_string())]),
            ),
        ]);
        let options = vec![
            "option.b|label=Beta".to_string(),
            "option.a|label=Alpha".to_string(),
        ];

        let projected = structured_options_for_node(&options, &attributes);
        assert_eq!(
            projected
                .iter()
                .map(|option| option.id.as_ref())
                .collect::<Vec<&str>>(),
            vec!["option.b", "option.a"]
        );
        assert!(projected[0].disabled);
        assert!(!projected[0].selected);
        assert!(projected[1].selected);
        assert!(!projected[1].disabled);
    }

    #[test]
    fn optimization_batch_20260826z_editor01_pane_option_projection_uses_hash_membership() {
        let source = include_str!("pane_option_projection.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
        assert!(production.contains("values: &HashSet<String>"));
        assert!(production.contains("-> HashSet<String>"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826z_editor01_pane_option_hash_membership_performance_evidence() {
        let option_ids = option_ids();
        let lookups = membership_lookups(&option_ids);
        assert_eq!(
            ordered_match_count(&option_ids, &lookups),
            hash_match_count(&option_ids, &lookups)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_match_count(
                    black_box(&option_ids),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_match_count(
                    black_box(&option_ids),
                    black_box(&lookups),
                ));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_match_count(
                    black_box(&option_ids),
                    black_box(&lookups),
                ));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_match_count(
                    black_box(&option_ids),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "EDITOR01_PANE_OPTION_HASH_MEMBERSHIP_BENCH_V1 option_ids={OPTION_ID_COUNT} \
             lookups={MEMBERSHIP_LOOKUP_COUNT} ordered_lookup_class=log_n \
             hash_lookup_class=average_constant ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-membership P95 {:?} exceeded 60% of ordered-membership P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
