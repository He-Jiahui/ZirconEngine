use std::collections::{BTreeMap, BTreeSet};

use crate::ui::retained_host as host_contract;
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

#[derive(Clone, Debug)]
struct ProjectedOption {
    raw: String,
    id: String,
    label: String,
    flags: Vec<String>,
}

impl ProjectedOption {
    fn has_flag(&self, expected: &str) -> bool {
        self.flags
            .iter()
            .any(|flag| flag.eq_ignore_ascii_case(expected))
    }

    fn flag_value(&self, expected_key: &str) -> Option<&str> {
        self.flags.iter().find_map(|flag| {
            let (key, value) = flag.split_once('=')?;
            key.trim()
                .eq_ignore_ascii_case(expected_key)
                .then(|| value.trim())
                .filter(|value| !value.is_empty())
        })
    }

    fn matches_id(&self, expected: &str) -> bool {
        let expected = expected.trim();
        !expected.is_empty()
            && [self.id.as_str(), self.label.as_str(), self.raw.as_str()]
                .into_iter()
                .any(|value| value == expected)
    }
}

fn structured_option(raw: &str) -> ProjectedOption {
    let mut parts = raw.splitn(2, '|');
    let id = parts.next().unwrap_or_default().trim();
    let flags = parts
        .next()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut option = ProjectedOption {
        raw: raw.to_string(),
        id: id.to_string(),
        label: id.to_string(),
        flags,
    };
    let label = option
        .flag_value("label")
        .or_else(|| option.flag_value("text"))
        .map(str::to_string);
    if let Some(label) = label {
        option.label = label;
    }
    option
}

fn option_matches_query(option: &ProjectedOption, query: Option<&str>) -> bool {
    let Some(query) = query else {
        return false;
    };
    option.id.to_ascii_lowercase().contains(query)
        || option.label.to_ascii_lowercase().contains(query)
        || option.raw.to_ascii_lowercase().contains(query)
}

fn option_matches_set(option: &ProjectedOption, values: &BTreeSet<String>) -> bool {
    values.iter().any(|value| option.matches_id(value))
}

fn option_id_set(value: Option<&toml::Value>) -> BTreeSet<String> {
    value
        .and_then(value_as_options)
        .unwrap_or_default()
        .into_iter()
        .collect()
}

fn selected_option_ids(attributes: &BTreeMap<String, toml::Value>) -> BTreeSet<String> {
    ["value", "selected_options", "selectedOptions"]
        .into_iter()
        .filter_map(|key| attributes.get(key))
        .flat_map(|value| selected_option_ids_from_value(value).into_iter())
        .collect()
}

fn selected_option_ids_from_value(value: &toml::Value) -> BTreeSet<String> {
    match UiValue::from_toml(value) {
        UiValue::String(value) | UiValue::Enum(value) => BTreeSet::from([value]),
        UiValue::Flags(values) => values.into_iter().collect(),
        UiValue::Array(values) => values
            .into_iter()
            .map(|value| value.display_text())
            .filter(|value| !value.is_empty())
            .collect(),
        value => {
            let text = value.display_text();
            if text.is_empty() {
                BTreeSet::new()
            } else {
                BTreeSet::from([text])
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
