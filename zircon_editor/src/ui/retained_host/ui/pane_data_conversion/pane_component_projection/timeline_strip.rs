use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;

pub(super) struct ProjectedTimelineStrip {
    pub(super) data: host_contract::TemplatePaneTimelineStripData,
}

pub(super) fn projected_timeline_strip(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedTimelineStrip {
    ProjectedTimelineStrip {
        data: projected_timeline_strip_data(component_role, attributes),
    }
}

pub(in crate::ui::retained_host::ui) fn projected_timeline_strip_data(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> host_contract::TemplatePaneTimelineStripData {
    if component_role != "canvas" || !has_variant(attributes, "timeline-strip") {
        return host_contract::TemplatePaneTimelineStripData::default();
    }

    let duration = positive_number(attributes, "duration").unwrap_or(1.0);
    let current_time = number_attribute(attributes, "current_time")
        .filter(|value| value.is_finite())
        .unwrap_or_default()
        .clamp(0.0, duration);
    let tick_interval = positive_number(attributes, "tick_interval")
        .unwrap_or_else(|| duration.min(0.25))
        .min(duration);

    host_contract::TemplatePaneTimelineStripData {
        duration,
        current_time,
        tick_interval,
        track_label: string_attribute(attributes, "track_label")
            .unwrap_or_default()
            .into(),
        keys: model_rc(timeline_keys(attributes, duration)),
    }
}

fn timeline_keys(
    attributes: &BTreeMap<String, toml::Value>,
    duration: f32,
) -> Vec<host_contract::TemplatePaneTimelineKeyData> {
    attributes
        .get("timeline_keys")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_table)
        .filter_map(|key| {
            let time = number_value(key.get("time")?)?;
            time.is_finite()
                .then(|| host_contract::TemplatePaneTimelineKeyData {
                    time: time.clamp(0.0, duration),
                    label: key
                        .get("label")
                        .and_then(toml::Value::as_str)
                        .unwrap_or_default()
                        .into(),
                    selected: key
                        .get("selected")
                        .and_then(toml::Value::as_bool)
                        .unwrap_or(false),
                })
        })
        .collect()
}

fn has_variant(attributes: &BTreeMap<String, toml::Value>, expected: &str) -> bool {
    ["component_variant", "variant"]
        .into_iter()
        .filter_map(|name| attributes.get(name).and_then(toml::Value::as_str))
        .any(|variant| variant.split_whitespace().any(|token| token == expected))
}

fn positive_number(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<f32> {
    number_attribute(attributes, name).filter(|value| value.is_finite() && *value > 0.0)
}

fn number_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<f32> {
    attributes.get(name).and_then(number_value)
}

fn number_value(value: &toml::Value) -> Option<f32> {
    match value {
        toml::Value::Float(value) => Some(*value as f32),
        toml::Value::Integer(value) => Some(*value as f32),
        _ => None,
    }
}

fn string_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<String> {
    attributes
        .get(name)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
}
