use super::super::pane_value_conversion::{value_as_bool, value_as_f64, value_as_string};

const MUI_ALERT_DEFAULT_SEVERITY: &str = "success";

const MUI_Z_INDEX_MOBILE_STEPPER: i32 = 1000;
const MUI_Z_INDEX_FAB: i32 = 1050;
const MUI_Z_INDEX_APP_BAR: i32 = 1100;
const MUI_Z_INDEX_DRAWER: i32 = 1200;
const MUI_Z_INDEX_MODAL: i32 = 1300;
const MUI_Z_INDEX_SNACKBAR: i32 = 1400;
const MUI_Z_INDEX_TOOLTIP: i32 = 1500;

pub(super) fn projected_component_variant(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
) -> String {
    let mut variant = attributes
        .get("invisible")
        .and_then(value_as_bool)
        .filter(|invisible| *invisible)
        .map(|_| "invisible".to_string())
        .or_else(|| attributes.get("mui_variant").and_then(value_as_string))
        .or_else(|| {
            attributes
                .get("component_variant")
                .and_then(value_as_string)
        })
        .or_else(|| attributes.get("variant").and_then(value_as_string))
        .unwrap_or_default();

    if let Some(animation) = attributes.get("animation").and_then(value_as_string) {
        if !animation.is_empty() && !variant.split_whitespace().any(|part| part == animation) {
            if variant.is_empty() {
                variant = animation;
            } else {
                variant.push(' ');
                variant.push_str(&animation);
            }
        }
    }

    if component_role == "divider" {
        if let Some(orientation) = attributes.get("orientation").and_then(value_as_string) {
            append_variant_token(&mut variant, &orientation);
        }
        if attributes
            .get("flexItem")
            .or_else(|| attributes.get("flex_item"))
            .and_then(value_as_bool)
            .unwrap_or(false)
        {
            append_variant_token(&mut variant, "flexItem");
        }
        if divider_has_children(attributes) {
            append_variant_token(&mut variant, "withChildren");
        }
        if let Some(text_align) = attributes
            .get("textAlign")
            .or_else(|| attributes.get("text_align"))
            .and_then(value_as_string)
        {
            if matches!(text_align.as_str(), "left" | "right") {
                append_variant_token(
                    &mut variant,
                    &format!("textAlign{}", pascal_case(&text_align)),
                );
            }
        }
    }

    if component_role == "timeline-dot" {
        if let Some(color) = attributes.get("color").and_then(value_as_string) {
            append_variant_token(&mut variant, &color);
        }
    }

    if component_role == "badge" {
        append_badge_variant_tokens(attributes, &mut variant);
    }

    if component_role == "alert" {
        append_alert_variant_tokens(attributes, &mut variant);
    }

    if component_role == "chip" {
        append_chip_variant_tokens(attributes, &mut variant);
    }

    if component_role == "skeleton" {
        append_skeleton_variant_tokens(attributes, &mut variant);
    }

    variant
}

fn append_variant_token(variant: &mut String, token: &str) {
    if token.is_empty()
        || variant
            .split_whitespace()
            .any(|part| part.eq_ignore_ascii_case(token))
    {
        return;
    }
    if !variant.is_empty() {
        variant.push(' ');
    }
    variant.push_str(token);
}

fn divider_has_children(attributes: &std::collections::BTreeMap<String, toml::Value>) -> bool {
    attributes
        .get("text")
        .or_else(|| attributes.get("label"))
        .and_then(value_as_string)
        .is_some_and(|value| !value.is_empty())
}

fn pascal_case(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + characters.as_str()
}

fn append_badge_variant_tokens(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    variant: &mut String,
) {
    let badge_variant = badge_variant(attributes);
    append_variant_token(variant, &badge_variant);
    if badge_is_invisible(attributes, &badge_variant) {
        append_variant_token(variant, "invisible");
    }

    let color = attributes
        .get("color")
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string());
    append_variant_token(variant, &color);

    let overlap = attributes
        .get("overlap")
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "rectangular".to_string());
    append_variant_token(variant, &overlap);
    append_variant_token(variant, &format!("overlap{}", pascal_case(&overlap)));

    let (vertical, horizontal) = badge_anchor_origin(attributes);
    append_variant_token(variant, &vertical);
    append_variant_token(variant, &horizontal);
    append_variant_token(
        variant,
        &format!(
            "anchorOrigin{}{}",
            pascal_case(&vertical),
            pascal_case(&horizontal)
        ),
    );
    append_variant_token(
        variant,
        &format!(
            "anchorOrigin{}{}{}",
            pascal_case(&vertical),
            pascal_case(&horizontal),
            pascal_case(&overlap)
        ),
    );
}

fn badge_variant(attributes: &std::collections::BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "standard".to_string())
}

fn badge_is_invisible(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    variant: &str,
) -> bool {
    if attributes
        .get("invisible")
        .and_then(value_as_bool)
        .unwrap_or(false)
    {
        return true;
    }
    let content = attributes
        .get("badgeContent")
        .or_else(|| attributes.get("badge_content"))
        .or_else(|| attributes.get("value_text"));
    if variant != "dot" && !content.is_some_and(badge_content_present) {
        return true;
    }
    content.is_some_and(|value| {
        badge_content_is_numeric_zero(value)
            && !attributes
                .get("showZero")
                .or_else(|| attributes.get("show_zero"))
                .and_then(value_as_bool)
                .unwrap_or(false)
    })
}

fn badge_content_present(value: &toml::Value) -> bool {
    match value {
        toml::Value::String(value) => !value.trim().is_empty(),
        toml::Value::Array(values) => !values.is_empty(),
        toml::Value::Table(values) => !values.is_empty(),
        toml::Value::Integer(_)
        | toml::Value::Float(_)
        | toml::Value::Boolean(_)
        | toml::Value::Datetime(_) => true,
    }
}

fn badge_content_is_numeric_zero(value: &toml::Value) -> bool {
    match value {
        toml::Value::Integer(value) => *value == 0,
        toml::Value::Float(value) => *value == 0.0,
        _ => false,
    }
}

fn badge_anchor_origin(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
) -> (String, String) {
    let anchor_origin = attributes.get("anchorOrigin");
    let vertical = string_from_toml_map(anchor_origin, "vertical")
        .or_else(|| {
            attributes
                .get("anchor_origin_vertical")
                .and_then(value_as_string)
        })
        .unwrap_or_else(|| "top".to_string());
    let horizontal = string_from_toml_map(anchor_origin, "horizontal")
        .or_else(|| {
            attributes
                .get("anchor_origin_horizontal")
                .and_then(value_as_string)
        })
        .unwrap_or_else(|| "right".to_string());
    (vertical, horizontal)
}

fn append_alert_variant_tokens(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    variant: &mut String,
) {
    let alert_variant = attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "standard".to_string());
    append_variant_token(variant, &alert_variant);

    let severity = alert_color_severity(attributes);
    append_variant_token(variant, &severity);
    append_variant_token(variant, &format!("color{}", pascal_case(&severity)));
    if alert_has_visible_icon(attributes) {
        append_variant_token(variant, "hasIcon");
    }
    if alert_has_action(attributes) {
        append_variant_token(variant, "hasAction");
    }
    if alert_has_close_action(attributes) {
        append_variant_token(variant, "hasCloseAction");
    }
}

fn alert_has_visible_icon(attributes: &std::collections::BTreeMap<String, toml::Value>) -> bool {
    !matches!(attributes.get("icon"), Some(toml::Value::Boolean(false)))
        && !matches!(
            attributes.get("show_icon"),
            Some(toml::Value::Boolean(false))
        )
        && !matches!(
            attributes.get("showIcon"),
            Some(toml::Value::Boolean(false))
        )
}

fn alert_has_action(attributes: &std::collections::BTreeMap<String, toml::Value>) -> bool {
    has_non_empty_attribute(attributes, &["action"]) || alert_has_close_action(attributes)
}

fn alert_has_close_action(attributes: &std::collections::BTreeMap<String, toml::Value>) -> bool {
    has_non_empty_attribute(attributes, &["onClose", "on_close"])
}

fn append_chip_variant_tokens(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    variant: &mut String,
) {
    let chip_variant = attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "filled".to_string());
    append_variant_token(variant, &chip_variant);

    let size = attributes
        .get("size")
        .or_else(|| attributes.get("mui_size"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "medium".to_string());
    append_variant_token(variant, &size);
    append_variant_token(variant, &format!("size{}", pascal_case(&size)));

    let color = attributes
        .get("color")
        .or_else(|| attributes.get("mui_color"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string());
    append_variant_token(variant, &color);
    append_variant_token(variant, &format!("color{}", pascal_case(&color)));

    if attributes
        .get("clickable")
        .and_then(value_as_bool)
        .unwrap_or(false)
    {
        append_variant_token(variant, "clickable");
    }
    if chip_is_deletable(attributes) {
        append_variant_token(variant, "deletable");
        append_variant_token(variant, "hasDeleteIcon");
    }
    if attributes
        .get("deleteIcon")
        .or_else(|| attributes.get("delete_icon"))
        .and_then(value_as_string)
        .is_some_and(|value| !value.is_empty())
    {
        append_variant_token(variant, "hasDeleteIcon");
    }
    if attributes
        .get("icon")
        .and_then(value_as_string)
        .is_some_and(|value| !value.is_empty())
    {
        append_variant_token(variant, "hasIcon");
    }
    if attributes
        .get("avatar")
        .and_then(value_as_string)
        .is_some_and(|value| !value.is_empty())
    {
        append_variant_token(variant, "hasAvatar");
    }
    if attributes
        .get("focusVisible")
        .or_else(|| attributes.get("focus_visible"))
        .and_then(value_as_bool)
        .unwrap_or(false)
    {
        append_variant_token(variant, "focusVisible");
    }
}

fn chip_is_deletable(attributes: &std::collections::BTreeMap<String, toml::Value>) -> bool {
    attributes
        .get("deletable")
        .and_then(value_as_bool)
        .unwrap_or(false)
        || chip_has_non_empty_attribute(attributes, &["onDelete", "on_delete"])
}

fn chip_has_non_empty_attribute(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    names: &[&str],
) -> bool {
    names.iter().any(|name| {
        attributes.get(*name).is_some_and(|value| match value {
            toml::Value::String(value) => !value.trim().is_empty(),
            toml::Value::Boolean(value) => *value,
            toml::Value::Array(values) => !values.is_empty(),
            toml::Value::Table(values) => !values.is_empty(),
            toml::Value::Integer(_) | toml::Value::Float(_) | toml::Value::Datetime(_) => true,
        })
    })
}

fn append_skeleton_variant_tokens(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    variant: &mut String,
) {
    if !variant_has_any_token(variant, &["text", "rectangular", "rounded", "circular"]) {
        append_variant_token(variant, skeleton_shape_variant(attributes).as_str());
    }
    if let Some(animation) = skeleton_animation(attributes, variant) {
        append_variant_token(variant, &animation);
    }
    if skeleton_has_children(attributes) {
        append_variant_token(variant, "withChildren");
        if !has_non_empty_attribute(attributes, &["width"]) {
            append_variant_token(variant, "fitContent");
        }
        if !has_non_empty_attribute(attributes, &["height"]) {
            append_variant_token(variant, "heightAuto");
        }
    }
}

fn skeleton_shape_variant(attributes: &std::collections::BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "text".to_string())
}

fn skeleton_animation(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    variant: &str,
) -> Option<String> {
    if variant_has_any_token(variant, &["pulse", "wave"]) {
        return None;
    }
    match attributes.get("animation") {
        Some(toml::Value::Boolean(false)) => None,
        Some(value) => value_as_string(value).filter(|value| !value.is_empty() && value != "false"),
        None => Some("pulse".to_string()),
    }
}

fn skeleton_has_children(attributes: &std::collections::BTreeMap<String, toml::Value>) -> bool {
    attributes
        .get("hasChildren")
        .or_else(|| attributes.get("has_children"))
        .or_else(|| attributes.get("withChildren"))
        .or_else(|| attributes.get("with_children"))
        .and_then(value_as_bool)
        .unwrap_or(false)
}

fn variant_has_any_token(variant: &str, expected: &[&str]) -> bool {
    variant.split_whitespace().any(|part| {
        expected
            .iter()
            .any(|token| part.eq_ignore_ascii_case(token))
    })
}

fn has_non_empty_attribute(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    names: &[&str],
) -> bool {
    names.iter().any(|name| {
        attributes.get(*name).is_some_and(|value| match value {
            toml::Value::String(value) => !value.trim().is_empty(),
            toml::Value::Array(values) => !values.is_empty(),
            toml::Value::Table(values) => !values.is_empty(),
            toml::Value::Integer(_)
            | toml::Value::Float(_)
            | toml::Value::Boolean(_)
            | toml::Value::Datetime(_) => true,
        })
    })
}

fn string_from_toml_map(value: Option<&toml::Value>, key: &str) -> Option<String> {
    let toml::Value::Table(map) = value? else {
        return None;
    };
    map.get(key).and_then(value_as_string)
}

pub(super) fn projected_surface_variant(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> String {
    attributes
        .get("surface_variant")
        .and_then(value_as_string)
        .unwrap_or_else(|| {
            default_mui_surface_variant(attributes, component_role, component_variant)
        })
}

fn default_mui_surface_variant(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> String {
    match component_role {
        "alert" => "alert".to_string(),
        "app-bar" => app_bar_surface_variant(attributes),
        "card"
            if component_variant
                .split_whitespace()
                .any(|part| part == "outlined") =>
        {
            "paper-outlined".to_string()
        }
        "card" => "paper".to_string(),
        "tooltip" => "tooltip".to_string(),
        "snackbar" | "snackbar-content" => "snackbar".to_string(),
        "paper"
            if component_variant
                .split_whitespace()
                .any(|part| part == "outlined") =>
        {
            "paper-outlined".to_string()
        }
        "paper" | "dialog" | "alert-dialog" | "popover" | "menu" => "popup".to_string(),
        "drawer" => "paper".to_string(),
        _ => String::new(),
    }
}

pub(super) fn projected_text_tone(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> String {
    attributes
        .get("text_tone")
        .and_then(value_as_string)
        .unwrap_or_else(|| match component_role {
            "app-bar"
                if matches!(
                    app_bar_color(attributes).as_str(),
                    "inherit" | "transparent"
                ) =>
            {
                "primary".to_string()
            }
            "app-bar" => "inverse".to_string(),
            "alert" if variant_contains(component_variant, "filled") => "inverse".to_string(),
            "alert" => alert_color_severity(attributes),
            "tooltip" | "snackbar" | "snackbar-content" => "inverse".to_string(),
            _ => String::new(),
        })
}

pub(super) fn projected_corner_radius(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
) -> f32 {
    attributes
        .get("corner_radius")
        .or_else(|| attributes.get("radius"))
        .and_then(value_as_f64)
        .map(|value| value as f32)
        .unwrap_or_else(|| match component_role {
            _ if attributes
                .get("square")
                .and_then(value_as_bool)
                .unwrap_or(false) =>
            {
                0.0
            }
            "alert" => 4.0,
            "card" | "paper" | "dialog" | "alert-dialog" | "popover" | "menu" | "tooltip"
            | "snackbar" | "snackbar-content" => 4.0,
            "app-bar" => 0.0,
            "drawer" => 0.0,
            _ => 0.0,
        })
}

pub(super) fn projected_border_width(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> f32 {
    attributes
        .get("border_width")
        .and_then(value_as_f64)
        .map(|value| value as f32)
        .unwrap_or_else(|| {
            if component_role == "alert" && variant_contains(component_variant, "outlined") {
                1.0
            } else if matches!(component_role, "card" | "paper")
                && component_variant
                    .split_whitespace()
                    .any(|part| part == "outlined")
            {
                1.0
            } else {
                0.0
            }
        })
}

pub(super) fn projected_elevation(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> f32 {
    attributes
        .get("elevation")
        .and_then(value_as_f64)
        .map(|value| value as f32)
        .unwrap_or_else(|| default_mui_elevation(attributes, component_role, component_variant))
}

pub(super) fn projected_z_index(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
    node_z_index: i32,
) -> i32 {
    attributes
        .get("z_index")
        .or_else(|| attributes.get("mui_z_index"))
        .or_else(|| attributes.get("zIndex"))
        .and_then(value_as_i32)
        .or_else(|| (node_z_index != 0).then_some(node_z_index))
        .unwrap_or_else(|| default_mui_z_index(component_role))
}

fn default_mui_elevation(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> f32 {
    if component_variant
        .split_whitespace()
        .any(|part| part == "outlined")
    {
        return 0.0;
    }
    match component_role {
        "app-bar" => 4.0,
        "alert" => 0.0,
        "card"
            if attributes
                .get("raised")
                .and_then(value_as_bool)
                .unwrap_or(false) =>
        {
            8.0
        }
        "card" => 1.0,
        "paper" => 1.0,
        "dialog" | "alert-dialog" => 24.0,
        "popover" | "menu" => 8.0,
        "snackbar" | "snackbar-content" => 6.0,
        "drawer" => 16.0,
        _ => 0.0,
    }
}

pub(super) fn projected_validation_level(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
    disabled: bool,
    has_component_descriptor: bool,
) -> String {
    if let Some(level) = attributes
        .get("validation_level")
        .and_then(value_as_string)
        .filter(|level| !level.is_empty())
    {
        return level;
    }
    if disabled {
        return "disabled".to_string();
    }
    if component_role == "alert" {
        return alert_color_severity(attributes);
    }
    if has_component_descriptor {
        "normal".to_string()
    } else {
        String::new()
    }
}

fn alert_color_severity(attributes: &std::collections::BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("color")
        .and_then(value_as_string)
        .filter(|color| !color.is_empty())
        .or_else(|| {
            attributes
                .get("severity")
                .and_then(value_as_string)
                .filter(|severity| !severity.is_empty())
        })
        .unwrap_or_else(|| MUI_ALERT_DEFAULT_SEVERITY.to_string())
}

fn app_bar_color(attributes: &std::collections::BTreeMap<String, toml::Value>) -> String {
    attributes
        .get("color")
        .and_then(value_as_string)
        .filter(|color| !color.is_empty())
        .unwrap_or_else(|| "primary".to_string())
}

fn app_bar_surface_variant(attributes: &std::collections::BTreeMap<String, toml::Value>) -> String {
    match app_bar_color(attributes).as_str() {
        "default" | "inherit" => "paper".to_string(),
        "transparent" => "transparent".to_string(),
        color => color.to_string(),
    }
}

fn variant_contains(component_variant: &str, expected: &str) -> bool {
    component_variant
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case(expected))
}

fn default_mui_z_index(component_role: &str) -> i32 {
    match component_role {
        "mobile-stepper" => MUI_Z_INDEX_MOBILE_STEPPER,
        "fab" | "floating-action-button" | "speed-dial" => MUI_Z_INDEX_FAB,
        "app-bar" => MUI_Z_INDEX_APP_BAR,
        "drawer" => MUI_Z_INDEX_DRAWER,
        // MUI Backdrop is normally nested under Modal; in the retained host it is a sibling,
        // so keep it immediately under modal surfaces while preserving the same global layer.
        "backdrop" => MUI_Z_INDEX_MODAL - 1,
        "modal" | "dialog" | "alert-dialog" | "popover" | "popper" | "menu" => MUI_Z_INDEX_MODAL,
        "snackbar" | "snackbar-content" => MUI_Z_INDEX_SNACKBAR,
        "tooltip" => MUI_Z_INDEX_TOOLTIP,
        _ => 0,
    }
}

fn value_as_i32(value: &toml::Value) -> Option<i32> {
    match value {
        toml::Value::Integer(value) => i32::try_from(*value).ok(),
        toml::Value::Float(value) => value.is_finite().then_some(value.round() as i32),
        toml::Value::String(value) => value.trim().parse::<i32>().ok(),
        _ => None,
    }
}
