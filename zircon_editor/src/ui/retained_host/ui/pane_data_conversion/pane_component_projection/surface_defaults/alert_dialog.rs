use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_bool;
use super::shared::{
    alert_color_severity, append_variant_token, dialog_severity, has_non_empty_attribute,
    pascal_case,
};

pub(super) fn append_alert_variant_tokens(
    attributes: &BTreeMap<String, toml::Value>,
    variant: &mut String,
) {
    let alert_variant = borrowed_alert_variant(attributes);
    append_variant_token(variant, alert_variant);

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

fn borrowed_alert_variant(attributes: &BTreeMap<String, toml::Value>) -> &str {
    attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("standard")
}

fn alert_has_visible_icon(attributes: &BTreeMap<String, toml::Value>) -> bool {
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

fn alert_has_action(attributes: &BTreeMap<String, toml::Value>) -> bool {
    has_non_empty_attribute(attributes, &["action"]) || alert_has_close_action(attributes)
}

fn alert_has_close_action(attributes: &BTreeMap<String, toml::Value>) -> bool {
    has_non_empty_attribute(attributes, &["onClose", "on_close"])
}

pub(super) fn append_dialog_variant_tokens(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    variant: &mut String,
) {
    if component_role == "dialog" {
        if has_non_empty_attribute(
            attributes,
            &[
                "action",
                "primary_action_text",
                "confirm_text",
                "close_text",
            ],
        ) {
            append_variant_token(variant, "hasAction");
        }
        return;
    }

    let severity = dialog_severity(attributes);
    append_variant_token(variant, &severity);
    append_variant_token(variant, &format!("color{}", pascal_case(&severity)));
    append_variant_token(variant, "hasAction");
    if attributes
        .get("destructive")
        .and_then(value_as_bool)
        .unwrap_or(false)
    {
        append_variant_token(variant, "destructive");
    }
    if attributes
        .get("confirm_enabled")
        .or_else(|| attributes.get("confirmEnabled"))
        .and_then(value_as_bool)
        == Some(false)
    {
        append_variant_token(variant, "confirmDisabled");
    }
}

#[cfg(test)]
#[path = "alert_dialog/borrowed_variant_tests.rs"]
mod borrowed_variant_tests;
