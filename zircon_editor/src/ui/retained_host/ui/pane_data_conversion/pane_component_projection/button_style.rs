use std::{borrow::Cow, collections::BTreeMap};

pub(super) fn button_style_values_with_aliases<'a>(
    attributes: &'a BTreeMap<String, toml::Value>,
    component_role: &str,
) -> Cow<'a, BTreeMap<String, toml::Value>> {
    let progress_aliases = is_progress_component_role(component_role);
    let progress_state_override = progress_aliases && attribute_is_true(attributes, "disabled");
    let (progress_track_source, progress_fill_source) = if progress_aliases {
        (
            progress_track_color_source(attributes),
            progress_fill_color_source(attributes),
        )
    } else {
        (None, None)
    };
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
#[path = "tests/button_style_performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "button_style/tests/deferred_progress_tests.rs"]
mod deferred_progress_tests;
