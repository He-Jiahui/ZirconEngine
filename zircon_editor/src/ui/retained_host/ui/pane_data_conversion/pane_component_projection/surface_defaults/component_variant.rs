use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_string};
use super::alert_dialog::{append_alert_variant_tokens, append_dialog_variant_tokens};
use super::badge::append_badge_variant_tokens;
use super::chip::append_chip_variant_tokens;
use super::shared::{append_variant_token, pascal_case};
use super::skeleton::append_skeleton_variant_tokens;

pub(super) fn projected_component_variant(
    attributes: &BTreeMap<String, toml::Value>,
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
        append_divider_variant_tokens(attributes, &mut variant);
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

    if matches!(component_role, "dialog" | "confirm-dialog" | "alert-dialog") {
        append_dialog_variant_tokens(attributes, component_role, &mut variant);
    }

    if component_role == "chip" {
        append_chip_variant_tokens(attributes, &mut variant);
    }

    if component_role == "skeleton" {
        append_skeleton_variant_tokens(attributes, &mut variant);
    }

    variant
}

fn append_divider_variant_tokens(attributes: &BTreeMap<String, toml::Value>, variant: &mut String) {
    if let Some(orientation) = attributes.get("orientation").and_then(value_as_string) {
        append_variant_token(variant, &orientation);
    }
    if attributes
        .get("flexItem")
        .or_else(|| attributes.get("flex_item"))
        .and_then(value_as_bool)
        .unwrap_or(false)
    {
        append_variant_token(variant, "flexItem");
    }
    if divider_has_children(attributes) {
        append_variant_token(variant, "withChildren");
    }
    if let Some(text_align) = attributes
        .get("textAlign")
        .or_else(|| attributes.get("text_align"))
        .and_then(value_as_string)
    {
        if matches!(text_align.as_str(), "left" | "right") {
            append_variant_token(variant, &format!("textAlign{}", pascal_case(&text_align)));
        }
    }
}

fn divider_has_children(attributes: &BTreeMap<String, toml::Value>) -> bool {
    attributes
        .get("text")
        .or_else(|| attributes.get("label"))
        .and_then(value_as_string)
        .is_some_and(|value| !value.is_empty())
}
