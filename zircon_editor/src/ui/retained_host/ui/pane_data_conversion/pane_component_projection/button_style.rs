use std::{borrow::Cow, collections::BTreeMap};

pub(super) fn button_style_values_with_aliases<'a>(
    attributes: &'a BTreeMap<String, toml::Value>,
    component_role: &str,
) -> Cow<'a, BTreeMap<String, toml::Value>> {
    let progress_aliases = is_progress_component_role(component_role);
    let progress_state_override = progress_aliases && attribute_is_true(attributes, "disabled");
    let progress_track_source = progress_track_color_source(attributes);
    let progress_fill_source = progress_fill_color_source(attributes);
    let needs_alias = [
        ("focus_border_color", "border_color"),
        ("thumb_outline_color", "border_color"),
        ("disabled_opacity", "opacity"),
    ]
    .into_iter()
    .any(|(source, target)| attributes.contains_key(source) && !attributes.contains_key(target))
        || progress_aliases
            && [
                (progress_track_source, "background_color"),
                (progress_fill_source, "foreground_color"),
            ]
            .into_iter()
            .any(|(source, target)| {
                source.is_some_and(|source| {
                    attributes.contains_key(source)
                        && (progress_state_override || !attributes.contains_key(target))
                })
            });
    if !needs_alias {
        return Cow::Borrowed(attributes);
    }

    let mut values = attributes.clone();
    alias_toml_value_key(&mut values, "focus_border_color", "border_color");
    alias_toml_value_key(&mut values, "thumb_outline_color", "border_color");
    alias_toml_value_key(&mut values, "disabled_opacity", "opacity");
    if progress_aliases {
        if let Some(source) = progress_track_source {
            project_progress_color(
                &mut values,
                source,
                "background_color",
                progress_state_override,
            );
        }
        if let Some(source) = progress_fill_source {
            project_progress_color(
                &mut values,
                source,
                "foreground_color",
                progress_state_override,
            );
        }
    }
    Cow::Owned(values)
}

fn is_progress_component_role(component_role: &str) -> bool {
    matches!(
        component_role,
        "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner"
    )
}

fn progress_track_color_source(attributes: &BTreeMap<String, toml::Value>) -> Option<&'static str> {
    if attribute_is_true(attributes, "disabled") {
        return attributes
            .contains_key("disabled_track_color")
            .then_some("disabled_track_color");
    }
    attributes
        .contains_key("track_color")
        .then_some("track_color")
}

fn progress_fill_color_source(attributes: &BTreeMap<String, toml::Value>) -> Option<&'static str> {
    if attribute_is_true(attributes, "disabled") {
        return attributes
            .contains_key("disabled_fill_color")
            .then_some("disabled_fill_color");
    }

    let semantic_source = match attributes
        .get("validation_level")
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
    {
        "warning" => Some("warning_color"),
        "error" | "danger" => Some("error_color"),
        _ => None,
    };
    semantic_source
        .filter(|source| attributes.contains_key(*source))
        .or_else(|| {
            attributes
                .contains_key("fill_color")
                .then_some("fill_color")
        })
}

fn attribute_is_true(attributes: &BTreeMap<String, toml::Value>, name: &str) -> bool {
    attributes
        .get(name)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn project_progress_color(
    values: &mut BTreeMap<String, toml::Value>,
    source: &str,
    target: &str,
    state_override: bool,
) {
    if state_override {
        if let Some(value) = values.get(source).cloned() {
            values.insert(target.to_string(), value);
        }
    } else {
        alias_toml_value_key(values, source, target);
    }
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
            button_style_values_with_aliases(&attributes, "button"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn button_style_alias_projection_owns_only_when_an_alias_is_inserted() {
        let attributes = BTreeMap::from([(
            "focus_border_color".to_string(),
            toml::Value::String("#123456".to_string()),
        )]);

        let values = button_style_values_with_aliases(&attributes, "button");

        assert!(matches!(&values, Cow::Owned(_)));
        assert_eq!(
            values.get("border_color").and_then(toml::Value::as_str),
            Some("#123456")
        );
        assert!(!attributes.contains_key("border_color"));
    }

    #[test]
    fn progress_style_aliases_project_track_and_fill_into_painter_channels() {
        let attributes = BTreeMap::from([
            (
                "track_color".to_string(),
                toml::Value::String("#151a1d".to_string()),
            ),
            (
                "fill_color".to_string(),
                toml::Value::String("#28b8c5".to_string()),
            ),
        ]);

        let values = button_style_values_with_aliases(&attributes, "progress");

        assert!(matches!(&values, Cow::Owned(_)));
        assert_eq!(
            values.get("background_color").and_then(toml::Value::as_str),
            Some("#151a1d")
        );
        assert_eq!(
            values.get("foreground_color").and_then(toml::Value::as_str),
            Some("#28b8c5")
        );
        assert!(!attributes.contains_key("background_color"));
        assert!(!attributes.contains_key("foreground_color"));
    }

    #[test]
    fn progress_style_aliases_select_semantic_and_disabled_state_colors() {
        let warning = BTreeMap::from([
            (
                "fill_color".to_string(),
                toml::Value::String("#28b8c5".to_string()),
            ),
            (
                "warning_color".to_string(),
                toml::Value::String("#d99b2b".to_string()),
            ),
            (
                "validation_level".to_string(),
                toml::Value::String("warning".to_string()),
            ),
        ]);
        let disabled = BTreeMap::from([
            ("disabled".to_string(), toml::Value::Boolean(true)),
            (
                "background_color".to_string(),
                toml::Value::String("#010203".to_string()),
            ),
            (
                "foreground_color".to_string(),
                toml::Value::String("#040506".to_string()),
            ),
            (
                "track_color".to_string(),
                toml::Value::String("#151a1d".to_string()),
            ),
            (
                "disabled_track_color".to_string(),
                toml::Value::String("#20262a".to_string()),
            ),
            (
                "fill_color".to_string(),
                toml::Value::String("#28b8c5".to_string()),
            ),
            (
                "disabled_fill_color".to_string(),
                toml::Value::String("#667078".to_string()),
            ),
        ]);

        let warning_values = button_style_values_with_aliases(&warning, "progress");
        let disabled_values = button_style_values_with_aliases(&disabled, "progress");

        assert_eq!(
            warning_values
                .get("foreground_color")
                .and_then(toml::Value::as_str),
            Some("#d99b2b")
        );
        assert_eq!(
            disabled_values
                .get("background_color")
                .and_then(toml::Value::as_str),
            Some("#20262a")
        );
        assert_eq!(
            disabled_values
                .get("foreground_color")
                .and_then(toml::Value::as_str),
            Some("#667078")
        );
    }

    #[test]
    fn progress_style_aliases_route_error_and_danger_to_the_error_color() {
        for validation_level in ["error", "danger"] {
            let attributes = BTreeMap::from([
                (
                    "fill_color".to_string(),
                    toml::Value::String("#28b8c5".to_string()),
                ),
                (
                    "error_color".to_string(),
                    toml::Value::String("#d74b4b".to_string()),
                ),
                (
                    "validation_level".to_string(),
                    toml::Value::String(validation_level.to_string()),
                ),
            ]);

            let values = button_style_values_with_aliases(&attributes, "progress");

            assert_eq!(
                values.get("foreground_color").and_then(toml::Value::as_str),
                Some("#d74b4b"),
                "{validation_level} must consume the Progress error channel"
            );
        }
    }

    #[test]
    fn disabled_progress_without_state_colors_defers_to_the_painter_palette() {
        let attributes = BTreeMap::from([
            ("disabled".to_string(), toml::Value::Boolean(true)),
            (
                "track_color".to_string(),
                toml::Value::String("#151a1d".to_string()),
            ),
            (
                "fill_color".to_string(),
                toml::Value::String("#28b8c5".to_string()),
            ),
        ]);

        let values = button_style_values_with_aliases(&attributes, "progress");

        assert!(matches!(&values, Cow::Borrowed(_)));
        assert!(!values.contains_key("background_color"));
        assert!(!values.contains_key("foreground_color"));
    }

    #[test]
    fn progress_style_aliases_do_not_leak_into_non_progress_components() {
        let attributes = BTreeMap::from([
            (
                "track_color".to_string(),
                toml::Value::String("#151a1d".to_string()),
            ),
            (
                "fill_color".to_string(),
                toml::Value::String("#28b8c5".to_string()),
            ),
        ]);

        assert!(matches!(
            button_style_values_with_aliases(&attributes, "slider"),
            Cow::Borrowed(_)
        ));
    }
}
