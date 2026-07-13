use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;

pub(super) struct ProjectedSampleGrid {
    pub(super) data: host_contract::TemplatePaneSampleGridData,
}

pub(super) fn projected_sample_grid(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedSampleGrid {
    ProjectedSampleGrid {
        data: projected_sample_grid_data(component_role, attributes),
    }
}

pub(in crate::ui::retained_host::ui) fn projected_sample_grid_data(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> host_contract::TemplatePaneSampleGridData {
    if component_role != "canvas" || !has_variant(attributes, "sample-grid") {
        return host_contract::TemplatePaneSampleGridData::default();
    }

    let mut data = host_contract::TemplatePaneSampleGridData {
        x_axis_label: string_attribute(attributes, "x_axis_label")
            .unwrap_or_default()
            .into(),
        y_axis_label: string_attribute(attributes, "y_axis_label")
            .unwrap_or_default()
            .into(),
        x_min: number_attribute(attributes, "x_min").unwrap_or(0.0),
        x_max: number_attribute(attributes, "x_max").unwrap_or(1.0),
        y_min: number_attribute(attributes, "y_min").unwrap_or(0.0),
        y_max: number_attribute(attributes, "y_max").unwrap_or(1.0),
        x_ticks: model_rc(number_array_attribute(attributes, "x_ticks")),
        y_ticks: model_rc(number_array_attribute(attributes, "y_ticks")),
        points: model_rc(sample_points(attributes)),
    };
    normalize_range(&mut data.x_min, &mut data.x_max);
    normalize_range(&mut data.y_min, &mut data.y_max);
    data
}

fn has_variant(attributes: &BTreeMap<String, toml::Value>, expected: &str) -> bool {
    ["component_variant", "variant"]
        .into_iter()
        .filter_map(|name| string_attribute(attributes, name))
        .any(|variant| variant.split_whitespace().any(|token| token == expected))
}

fn sample_points(
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<host_contract::TemplatePaneSampleGridPointData> {
    attributes
        .get("sample_points")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_table)
        .filter_map(|point| {
            Some(host_contract::TemplatePaneSampleGridPointData {
                x: number_value(point.get("x")?)?,
                y: number_value(point.get("y")?)?,
                label: point
                    .get("label")
                    .and_then(toml::Value::as_str)
                    .unwrap_or_default()
                    .into(),
                selected: point
                    .get("selected")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
}

fn number_array_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Vec<f32> {
    attributes
        .get(name)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(number_value)
        .collect()
}

fn string_attribute(attributes: &BTreeMap<String, toml::Value>, name: &str) -> Option<String> {
    attributes
        .get(name)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
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

fn normalize_range(min: &mut f32, max: &mut f32) {
    if !min.is_finite() || !max.is_finite() || *max <= *min {
        *min = 0.0;
        *max = 1.0;
    }
}
