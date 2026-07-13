use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;

pub(super) struct ProjectedWeightHeatmap {
    pub(super) data: host_contract::TemplatePaneWeightHeatmapData,
}

pub(super) fn projected_weight_heatmap(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedWeightHeatmap {
    ProjectedWeightHeatmap {
        data: projected_weight_heatmap_data(component_role, attributes),
    }
}

pub(in crate::ui::retained_host::ui) fn projected_weight_heatmap_data(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> host_contract::TemplatePaneWeightHeatmapData {
    if component_role != "canvas" || !has_variant(attributes, "weight-heatmap") {
        return host_contract::TemplatePaneWeightHeatmapData::default();
    }

    host_contract::TemplatePaneWeightHeatmapData {
        columns: integer_attribute(attributes, "heatmap_columns")
            .unwrap_or(12)
            .clamp(4, 32),
        rows: integer_attribute(attributes, "heatmap_rows")
            .unwrap_or(8)
            .clamp(3, 24),
        low_label: string_attribute(attributes, "low_label")
            .unwrap_or_else(|| "0.0".to_owned())
            .into(),
        high_label: string_attribute(attributes, "high_label")
            .unwrap_or_else(|| "1.0".to_owned())
            .into(),
        sources: model_rc(heat_sources(attributes)),
    }
}

fn has_variant(attributes: &BTreeMap<String, toml::Value>, expected: &str) -> bool {
    ["component_variant", "variant"]
        .into_iter()
        .filter_map(|name| string_attribute(attributes, name))
        .any(|variant| variant.split_whitespace().any(|token| token == expected))
}

fn heat_sources(
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneWeightHeatmapSourceData> {
    attributes
        .get("heat_sources")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_table)
        .filter_map(|source| {
            Some(host_contract::TemplatePaneWeightHeatmapSourceData {
                x: normalized_number(source.get("x")?)?,
                y: normalized_number(source.get("y")?)?,
                weight: normalized_number(source.get("weight")?)?,
                selected: source
                    .get("selected")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
}

fn normalized_number(value: &toml::Value) -> Option<f32> {
    number_value(value).map(|value| value.clamp(0.0, 1.0))
}

fn number_value(value: &toml::Value) -> Option<f32> {
    match value {
        toml::Value::Float(value) if value.is_finite() => Some(*value as f32),
        toml::Value::Integer(value) => Some(*value as f32),
        _ => None,
    }
}

fn integer_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<i32> {
    attributes.get(name).and_then(|value| match value {
        toml::Value::Integer(value) => i32::try_from(*value).ok(),
        _ => None,
    })
}

fn string_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<String> {
    attributes
        .get(name)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
}
