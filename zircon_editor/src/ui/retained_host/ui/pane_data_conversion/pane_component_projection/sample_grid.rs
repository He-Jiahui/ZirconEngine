use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use crate::ui::sample_grid::{SampleGridGeneration, SampleGridGenerationInput, SampleGridPoint};

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

    let mut x_min = number_attribute(attributes, "x_min").unwrap_or(0.0);
    let mut x_max = number_attribute(attributes, "x_max").unwrap_or(1.0);
    let mut y_min = number_attribute(attributes, "y_min").unwrap_or(0.0);
    let mut y_max = number_attribute(attributes, "y_max").unwrap_or(1.0);
    normalize_range(&mut x_min, &mut x_max);
    normalize_range(&mut y_min, &mut y_max);
    host_contract::TemplatePaneSampleGridData {
        generation: SampleGridGeneration::new(SampleGridGenerationInput {
            x_axis_label: string_attribute(attributes, "x_axis_label").unwrap_or_default(),
            y_axis_label: string_attribute(attributes, "y_axis_label").unwrap_or_default(),
            x_min,
            x_max,
            y_min,
            y_max,
            x_ticks: number_array_attribute(attributes, "x_ticks"),
            y_ticks: number_array_attribute(attributes, "y_ticks"),
            points: sample_points(attributes),
        }),
    }
}

fn has_variant(attributes: &BTreeMap<String, toml::Value>, expected: &str) -> bool {
    ["component_variant", "variant"]
        .into_iter()
        .filter_map(|name| attributes.get(name).and_then(toml::Value::as_str))
        .any(|variant| variant.split_whitespace().any(|token| token == expected))
}

fn sample_points(attributes: &BTreeMap<String, toml::Value>) -> Vec<SampleGridPoint> {
    attributes
        .get("sample_points")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_table)
        .filter_map(|point| {
            Some(SampleGridPoint::new(
                number_value(point.get("x")?)?,
                number_value(point.get("y")?)?,
                point
                    .get("label")
                    .and_then(toml::Value::as_str)
                    .unwrap_or_default(),
                point
                    .get("selected")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
            ))
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
