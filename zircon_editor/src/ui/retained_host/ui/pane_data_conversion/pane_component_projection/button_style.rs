use std::{borrow::Cow, collections::BTreeMap};

pub(super) fn button_style_values_with_aliases<'a>(
    attributes: &'a BTreeMap<String, toml::Value>,
) -> Cow<'a, BTreeMap<String, toml::Value>> {
    let needs_alias = [
        ("focus_border_color", "border_color"),
        ("thumb_outline_color", "border_color"),
        ("disabled_opacity", "opacity"),
    ]
    .into_iter()
    .any(|(source, target)| attributes.contains_key(source) && !attributes.contains_key(target));
    if !needs_alias {
        return Cow::Borrowed(attributes);
    }

    let mut values = attributes.clone();
    alias_toml_value_key(&mut values, "focus_border_color", "border_color");
    alias_toml_value_key(&mut values, "thumb_outline_color", "border_color");
    alias_toml_value_key(&mut values, "disabled_opacity", "opacity");
    Cow::Owned(values)
}

fn alias_toml_value_key(values: &mut BTreeMap<String, toml::Value>, source: &str, target: &str) {
    if values.contains_key(target) {
        return;
    }
    if let Some(value) = values.get(source).cloned() {
        values.insert(target.to_string(), value);
    }
}

#[cfg(test)]
mod performance_tests {
    use std::{borrow::Cow, collections::BTreeMap};

    use super::button_style_values_with_aliases;

    #[test]
    fn button_style_alias_projection_borrows_when_no_alias_is_needed() {
        let attributes = BTreeMap::from([(
            "button_variant".to_string(),
            toml::Value::String("filled".to_string()),
        )]);

        assert!(matches!(
            button_style_values_with_aliases(&attributes),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn button_style_alias_projection_owns_only_when_an_alias_is_inserted() {
        let attributes = BTreeMap::from([(
            "focus_border_color".to_string(),
            toml::Value::String("#123456".to_string()),
        )]);

        let values = button_style_values_with_aliases(&attributes);

        assert!(matches!(&values, Cow::Owned(_)));
        assert_eq!(
            values.get("border_color").and_then(toml::Value::as_str),
            Some("#123456")
        );
        assert!(!attributes.contains_key("border_color"));
    }
}
