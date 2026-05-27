use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_class, bool_attribute, bool_attribute_any, int_attribute_any, pascal_case,
    string_attribute_any, string_from_attributes_any,
};

const BREAKPOINTS: &[&str] = &["xs", "sm", "md", "lg", "xl"];

pub(super) fn append_layout_component_classes(
    node: &mut UiTemplateNode,
    component: &str,
    prefix: &str,
) -> bool {
    match component {
        "Box"
        | "ClickAwayListener"
        | "Portal"
        | "NoSsr"
        | "CssBaseline"
        | "InitColorSchemeScript"
        | "Fade"
        | "Grow"
        | "Slide"
        | "Zoom" => true,
        "Stack" => {
            append_stack_classes(node, prefix);
            true
        }
        "UseMediaQuery" => {
            append_use_media_query_classes(node, prefix);
            true
        }
        "Container" => {
            append_container_classes(node, prefix);
            true
        }
        "Grid" => {
            append_grid_classes(node, prefix);
            true
        }
        "Masonry" => {
            append_masonry_classes(node, prefix);
            true
        }
        "Collapse" => {
            append_collapse_classes(node, prefix);
            true
        }
        _ => false,
    }
}

pub(super) fn append_layout_slot_classes(
    child: &mut UiTemplateNode,
    owner_component: &str,
    owner_attributes: &BTreeMap<String, Value>,
    slot_name: &str,
) -> bool {
    match (owner_component, slot_name) {
        ("Collapse", "wrapper" | "wrapperInner") => {
            let orientation = collapse_orientation(owner_attributes);
            append_class(&mut child.classes, format!("MuiCollapse-{orientation}"));
            true
        }
        _ => false,
    }
}

pub(super) fn suppresses_generic_classes(component: &str) -> bool {
    matches!(
        component,
        "Box"
            | "ClickAwayListener"
            | "Collapse"
            | "Container"
            | "CssBaseline"
            | "Fade"
            | "Grid"
            | "Grow"
            | "InitColorSchemeScript"
            | "Masonry"
            | "NoSsr"
            | "Portal"
            | "Slide"
            | "Stack"
            | "UseMediaQuery"
            | "Zoom"
    )
}

fn append_container_classes(node: &mut UiTemplateNode, prefix: &str) {
    if let Some(max_width) = container_max_width(node) {
        append_class(
            &mut node.classes,
            format!("{prefix}-maxWidth{}", pascal_case(&max_width)),
        );
    }
    if bool_attribute(node, "fixed") {
        append_class(&mut node.classes, format!("{prefix}-fixed"));
    }
    if bool_attribute_any(node, &["disableGutters", "disable_gutters"]) {
        append_class(&mut node.classes, format!("{prefix}-disableGutters"));
    }
}

fn container_max_width(node: &UiTemplateNode) -> Option<String> {
    match node.attributes.get("maxWidth") {
        Some(Value::Boolean(false)) => None,
        Some(Value::String(value)) if value.trim() == "false" => None,
        Some(Value::String(value)) if !value.trim().is_empty() => Some(value.trim().to_string()),
        Some(Value::Integer(value)) => Some(value.to_string()),
        Some(Value::Float(value)) if value.is_finite() => Some(value.round().to_string()),
        Some(_) => Some("lg".to_string()),
        None => Some("lg".to_string()),
    }
}

fn append_grid_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute(node, "container") {
        append_class(&mut node.classes, format!("{prefix}-container"));
        append_responsive_value_classes(
            node,
            prefix,
            "spacing",
            &["spacing"],
            ResponsiveClassOptions::positive_numbers_only(),
        );
        append_responsive_value_classes(
            node,
            prefix,
            "columns",
            &["columns"],
            ResponsiveClassOptions::positive_numbers_only(),
        );
        append_responsive_value_classes(
            node,
            prefix,
            "rowSpacing",
            &["rowSpacing", "row_spacing"],
            ResponsiveClassOptions::positive_numbers_only(),
        );
        append_responsive_value_classes(
            node,
            prefix,
            "columnSpacing",
            &["columnSpacing", "column_spacing"],
            ResponsiveClassOptions::positive_numbers_only(),
        );

        append_responsive_value_classes_with_default(
            node,
            prefix,
            "direction",
            &["direction"],
            "row",
            ResponsiveClassOptions::default(),
        );
        append_responsive_value_classes_with_default(
            node,
            prefix,
            "wrap",
            &["wrap"],
            "wrap",
            ResponsiveClassOptions::default(),
        );
    }

    append_responsive_value_classes(
        node,
        prefix,
        "grid",
        &["size"],
        ResponsiveClassOptions::skip_false(),
    );
    append_responsive_value_classes(
        node,
        prefix,
        "offset",
        &["offset"],
        ResponsiveClassOptions::skip_false(),
    );
}

fn append_stack_classes(node: &mut UiTemplateNode, prefix: &str) {
    append_responsive_value_classes_with_default(
        node,
        prefix,
        "direction",
        &["direction"],
        "column",
        ResponsiveClassOptions::default(),
    );
    append_responsive_value_classes(
        node,
        prefix,
        "spacing",
        &["spacing"],
        ResponsiveClassOptions::default(),
    );
    if bool_attribute_any(node, &["useFlexGap", "use_flex_gap"]) {
        append_class(&mut node.classes, format!("{prefix}-useFlexGap"));
    }
}

fn append_use_media_query_classes(node: &mut UiTemplateNode, prefix: &str) {
    if bool_attribute_any(node, &["matches", "defaultMatches", "default_matches"]) {
        append_class(&mut node.classes, format!("{prefix}-match"));
    }
    if bool_attribute_any(node, &["noSsr", "no_ssr"]) {
        append_class(&mut node.classes, format!("{prefix}-noSsr"));
    }
}

fn append_masonry_classes(node: &mut UiTemplateNode, prefix: &str) {
    if masonry_config_attribute_present(node, &["columns"]) {
        append_class(&mut node.classes, format!("{prefix}-columnsConfigured"));
    }
    if masonry_config_attribute_present(node, &["spacing"]) {
        append_class(&mut node.classes, format!("{prefix}-spacingConfigured"));
    }
    if bool_attribute_any(node, &["sequential"]) {
        append_class(&mut node.classes, format!("{prefix}-sequential"));
    }
    if int_attribute_any(node, &["defaultColumns", "default_columns"]).is_some()
        && scalar_attribute_present(node, &["defaultHeight", "default_height"])
        && scalar_attribute_present(node, &["defaultSpacing", "default_spacing"])
    {
        append_class(&mut node.classes, format!("{prefix}-ssrDefaults"));
    }
}

fn append_collapse_classes(node: &mut UiTemplateNode, prefix: &str) {
    let orientation = string_attribute_any(node, &["orientation"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "vertical".to_string());
    append_class(&mut node.classes, format!("{prefix}-{orientation}"));

    let status = string_attribute_any(node, &["transition_status"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "entered".to_string());
    if status == "entered" {
        append_class(&mut node.classes, format!("{prefix}-entered"));
    }
    if status == "exited" && !bool_attribute(node, "in") && collapse_size_is_zero(&node.attributes)
    {
        append_class(&mut node.classes, format!("{prefix}-hidden"));
    }
}

fn masonry_config_attribute_present(node: &UiTemplateNode, names: &[&str]) -> bool {
    names
        .iter()
        .any(|name| masonry_config_value_present(node.attributes.get(*name)))
}

fn masonry_config_value_present(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(value)) => !value.trim().is_empty(),
        Some(Value::Integer(_)) => true,
        Some(Value::Float(value)) => value.is_finite(),
        Some(Value::Array(values)) => values
            .iter()
            .any(|value| masonry_config_value_present(Some(value))),
        Some(Value::Table(values)) => values
            .values()
            .any(|value| masonry_config_value_present(Some(value))),
        _ => false,
    }
}

fn scalar_attribute_present(node: &UiTemplateNode, names: &[&str]) -> bool {
    names.iter().any(|name| match node.attributes.get(*name) {
        Some(Value::String(value)) => !value.trim().is_empty(),
        Some(Value::Integer(_)) => true,
        Some(Value::Float(value)) => value.is_finite(),
        _ => false,
    })
}

fn collapse_size_is_zero(attributes: &BTreeMap<String, Value>) -> bool {
    string_from_attributes_any(attributes, &["collapsedSize", "collapsed_size"])
        .map(|value| matches!(value.as_str(), "0" | "0px" | "0.0" | "0.0px"))
        .unwrap_or(true)
}

fn collapse_orientation(attributes: &BTreeMap<String, Value>) -> String {
    string_from_attributes_any(attributes, &["orientation"])
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "vertical".to_string())
}

#[derive(Clone, Copy, Default)]
struct ResponsiveClassOptions {
    skip_false: bool,
    positive_numbers_only: bool,
}

impl ResponsiveClassOptions {
    fn skip_false() -> Self {
        Self {
            skip_false: true,
            positive_numbers_only: false,
        }
    }

    fn positive_numbers_only() -> Self {
        Self {
            skip_false: true,
            positive_numbers_only: true,
        }
    }
}

fn append_responsive_value_classes(
    node: &mut UiTemplateNode,
    prefix: &str,
    class_stem: &str,
    attribute_names: &[&str],
    options: ResponsiveClassOptions,
) {
    let Some(value) = attribute_names
        .iter()
        .find_map(|name| node.attributes.get(*name))
    else {
        return;
    };

    for (breakpoint, token) in responsive_class_tokens(value, options) {
        append_class(
            &mut node.classes,
            format!("{prefix}-{class_stem}-{breakpoint}-{token}"),
        );
    }
}

fn append_responsive_value_classes_with_default(
    node: &mut UiTemplateNode,
    prefix: &str,
    class_stem: &str,
    attribute_names: &[&str],
    default: &str,
    options: ResponsiveClassOptions,
) {
    let Some(value) = attribute_names
        .iter()
        .find_map(|name| node.attributes.get(*name))
    else {
        append_class(
            &mut node.classes,
            format!("{prefix}-{class_stem}-xs-{default}"),
        );
        return;
    };

    for (breakpoint, token) in responsive_class_tokens(value, options) {
        append_class(
            &mut node.classes,
            format!("{prefix}-{class_stem}-{breakpoint}-{token}"),
        );
    }
}

fn responsive_class_tokens(
    value: &Value,
    options: ResponsiveClassOptions,
) -> Vec<(String, String)> {
    match value {
        Value::Array(values) => values
            .iter()
            .zip(BREAKPOINTS.iter())
            .filter_map(|(value, breakpoint)| {
                class_token(value, options).map(|token| ((*breakpoint).to_string(), token))
            })
            .collect(),
        Value::Table(values) => BREAKPOINTS
            .iter()
            .filter_map(|breakpoint| {
                values
                    .get(*breakpoint)
                    .and_then(|value| class_token(value, options))
                    .map(|token| ((*breakpoint).to_string(), token))
            })
            .collect(),
        _ => class_token(value, options)
            .map(|token| vec![("xs".to_string(), token)])
            .unwrap_or_default(),
    }
}

fn class_token(value: &Value, options: ResponsiveClassOptions) -> Option<String> {
    match value {
        Value::String(value) => {
            let value = value.trim();
            if value.is_empty() || (options.skip_false && value == "false") {
                return None;
            }
            if options.positive_numbers_only && value.parse::<f64>().ok()? <= 0.0 {
                return None;
            }
            Some(value.to_string())
        }
        Value::Integer(value) => {
            if options.positive_numbers_only && *value <= 0 {
                return None;
            }
            Some(value.to_string())
        }
        Value::Float(value) if value.is_finite() => {
            if options.positive_numbers_only && *value <= 0.0 {
                return None;
            }
            Some(format_float_token(*value))
        }
        Value::Boolean(true) => Some("true".to_string()),
        Value::Boolean(false) if !options.skip_false => Some("false".to_string()),
        _ => None,
    }
}

fn format_float_token(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}
