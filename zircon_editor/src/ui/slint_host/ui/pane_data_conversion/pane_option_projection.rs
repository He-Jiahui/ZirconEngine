use std::collections::{BTreeMap, BTreeSet};

use crate::ui::slint_host as host_contract;
use zircon_runtime_interface::ui::component::UiValue;

use super::pane_value_conversion::value_as_options;

pub(super) fn structured_options_for_node(
    options: &[String],
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneOptionData> {
    let selected = selected_option_ids(attributes.get("value"));
    let disabled = option_id_set(attributes.get("disabled_options"));
    let special = option_id_set(attributes.get("special_options"));
    let focused = option_id_set(attributes.get("focused_options"));
    let hovered = option_id_set(attributes.get("hovered_options"));
    let pressed = option_id_set(attributes.get("pressed_options"));
    let query = attributes
        .get("query")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);

    options
        .iter()
        .map(|option| host_contract::TemplatePaneOptionData {
            matched: option_matches_query(option, query.as_deref()),
            id: option.as_str().into(),
            label: option.as_str().into(),
            selected: selected.contains(option),
            disabled: disabled.contains(option),
            special: special.contains(option),
            focused: focused.contains(option),
            hovered: hovered.contains(option),
            pressed: pressed.contains(option),
        })
        .collect()
}

fn option_matches_query(option: &str, query: Option<&str>) -> bool {
    query
        .map(|query| option.to_ascii_lowercase().contains(query))
        .unwrap_or(false)
}

fn option_id_set(value: Option<&toml::Value>) -> BTreeSet<String> {
    value
        .and_then(value_as_options)
        .unwrap_or_default()
        .into_iter()
        .collect()
}

fn selected_option_ids(value: Option<&toml::Value>) -> BTreeSet<String> {
    let Some(value) = value else {
        return BTreeSet::new();
    };

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
