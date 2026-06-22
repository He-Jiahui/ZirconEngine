use std::collections::BTreeMap;

pub(super) fn button_style_values_with_aliases(
    attributes: &BTreeMap<String, toml::Value>,
) -> BTreeMap<String, toml::Value> {
    let mut values = attributes.clone();
    alias_toml_value_key(&mut values, "focus_border_color", "border_color");
    alias_toml_value_key(&mut values, "thumb_outline_color", "border_color");
    alias_toml_value_key(&mut values, "disabled_opacity", "opacity");
    values
}

fn alias_toml_value_key(values: &mut BTreeMap<String, toml::Value>, source: &str, target: &str) {
    if values.contains_key(target) {
        return;
    }
    if let Some(value) = values.get(source).cloned() {
        values.insert(target.to_string(), value);
    }
}
