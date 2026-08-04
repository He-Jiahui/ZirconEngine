use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::template_runtime::{RetainedUiHostRouteProjection, RetainedUiHostValue};
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiFrame};

pub(super) fn template_frame(frame: UiFrame) -> host_contract::TemplateNodeFrameData {
    host_contract::TemplateNodeFrameData {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

pub(super) fn shared_string_list(
    values: Vec<String>,
) -> ModelRc<crate::ui::retained_host::primitives::SharedString> {
    model_rc(values.into_iter().map(Into::into).collect())
}

pub(super) fn first_string_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    keys: &[&str],
) -> Option<String> {
    keys.iter()
        .find_map(|key| string_property(properties, key))
        .filter(|value| !value.is_empty())
}

pub(super) fn string_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<String> {
    match properties.get(key) {
        Some(RetainedUiHostValue::String(value)) => Some(value.clone()),
        Some(RetainedUiHostValue::Integer(value)) => Some(value.to_string()),
        Some(RetainedUiHostValue::Float(value)) => Some(value.to_string()),
        Some(RetainedUiHostValue::Bool(value)) => Some(value.to_string()),
        _ => None,
    }
}

pub(super) fn numeric_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<f64> {
    match properties.get(key) {
        Some(RetainedUiHostValue::Float(value)) => Some(*value),
        Some(RetainedUiHostValue::Integer(value)) => Some(*value as f64),
        Some(RetainedUiHostValue::String(value)) => value.parse().ok(),
        _ => None,
    }
}

pub(super) fn integer_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<i32> {
    match properties.get(key) {
        Some(RetainedUiHostValue::Integer(value)) => i32::try_from(*value).ok(),
        Some(RetainedUiHostValue::Float(value)) => Some(*value as i32),
        Some(RetainedUiHostValue::String(value)) => value.parse().ok(),
        _ => None,
    }
}

pub(super) fn bool_property(properties: &BTreeMap<String, RetainedUiHostValue>, key: &str) -> bool {
    match properties.get(key) {
        Some(RetainedUiHostValue::Bool(value)) => *value,
        Some(RetainedUiHostValue::String(value)) => value.parse().unwrap_or(false),
        _ => false,
    }
}

pub(super) fn normalized_percent(properties: &BTreeMap<String, RetainedUiHostValue>) -> f32 {
    let Some(value) = numeric_property(properties, "value") else {
        return 0.0;
    };
    let min = numeric_property(properties, "min").unwrap_or(0.0);
    let max = numeric_property(properties, "max").unwrap_or(100.0);
    if max <= min {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0) as f32
    }
}

pub(super) fn string_array_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
    fallback: &[String],
) -> Vec<String> {
    match properties.get(key) {
        Some(RetainedUiHostValue::Array(values)) => values
            .iter()
            .filter_map(host_value_display_text)
            .filter(|value| !value.is_empty())
            .collect(),
        Some(value) => host_value_display_text(value).into_iter().collect(),
        None => fallback.to_vec(),
    }
}

fn host_value_display_text(value: &RetainedUiHostValue) -> Option<String> {
    match value {
        RetainedUiHostValue::String(value) => Some(value.clone()),
        RetainedUiHostValue::Integer(value) => Some(value.to_string()),
        RetainedUiHostValue::Float(value) => Some(value.to_string()),
        RetainedUiHostValue::Bool(value) => Some(value.to_string()),
        RetainedUiHostValue::Datetime(value) => Some(value.clone()),
        RetainedUiHostValue::Array(_) | RetainedUiHostValue::Table(_) => None,
    }
}

pub(super) fn preferred_route_binding<const N: usize>(
    routes: &[RetainedUiHostRouteProjection],
    kinds: [UiEventKind; N],
) -> Option<String> {
    kinds
        .iter()
        .find_map(|kind| routes.iter().find(|route| route.event_kind == *kind))
        .or_else(|| routes.first())
        .map(|route| route.binding_id.clone())
}

pub(super) fn preferred_route_action_id<const N: usize>(
    routes: &[RetainedUiHostRouteProjection],
    kinds: [UiEventKind; N],
) -> Option<String> {
    preferred_route(routes, kinds).map(|route| {
        if route.action_id.is_empty() {
            route.binding_id.clone()
        } else {
            route.action_id.clone()
        }
    })
}

fn preferred_route<const N: usize>(
    routes: &[RetainedUiHostRouteProjection],
    kinds: [UiEventKind; N],
) -> Option<&RetainedUiHostRouteProjection> {
    kinds
        .iter()
        .find_map(|kind| routes.iter().find(|route| route.event_kind == *kind))
        .or_else(|| routes.first())
}

pub(super) fn color_property(
    properties: &BTreeMap<String, RetainedUiHostValue>,
    key: &str,
) -> Option<crate::ui::retained_host::primitives::Color> {
    let RetainedUiHostValue::String(value) = properties.get(key)? else {
        return None;
    };
    let rgba = parse_hex_rgba(value)?;
    Some(crate::ui::retained_host::primitives::Color::from_argb_u8(
        rgba[3], rgba[0], rgba[1], rgba[2],
    ))
}

fn parse_hex_rgba(raw: &str) -> Option<[u8; 4]> {
    let hex = raw.trim().strip_prefix('#')?;
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    match hex.len() {
        6 => Some([channel(0..2)?, channel(2..4)?, channel(4..6)?, 255]),
        8 => Some([
            channel(0..2)?,
            channel(2..4)?,
            channel(4..6)?,
            channel(6..8)?,
        ]),
        _ => None,
    }
}
