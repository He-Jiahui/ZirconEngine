use std::collections::BTreeMap;

pub(in super::super) fn projected_dialog_value_text(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> Option<String> {
    if !matches!(component_role, "dialog" | "confirm-dialog" | "alert-dialog") {
        return None;
    }
    first_non_empty_attribute(attributes, &["message", "description", "body"]).map(str::to_owned)
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
#[path = "tests/message.rs"]
mod tests;
