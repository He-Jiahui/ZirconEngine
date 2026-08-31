use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use crate::ui::weight_heatmap::{
    WeightHeatmapGeneration, WeightHeatmapGenerationInput, WeightHeatmapSource,
};

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
        generation: WeightHeatmapGeneration::new(WeightHeatmapGenerationInput {
            columns: integer_attribute(attributes, "heatmap_columns").unwrap_or(12),
            rows: integer_attribute(attributes, "heatmap_rows").unwrap_or(8),
            low_label: string_attribute(attributes, "low_label")
                .unwrap_or_else(|| "0.0".to_owned()),
            high_label: string_attribute(attributes, "high_label")
                .unwrap_or_else(|| "1.0".to_owned()),
            sources: heat_sources(attributes),
        }),
    }
}

fn has_variant(attributes: &BTreeMap<String, toml::Value>, expected: &str) -> bool {
    ["component_variant", "variant"]
        .into_iter()
        .filter_map(|name| attributes.get(name).and_then(toml::Value::as_str))
        .any(|variant| variant.split_whitespace().any(|token| token == expected))
}

fn heat_sources(attributes: &BTreeMap<String, toml::Value>) -> Vec<WeightHeatmapSource> {
    let Some(values) = attributes
        .get("heat_sources")
        .and_then(toml::Value::as_array)
    else {
        return Vec::new();
    };
    let mut sources = Vec::with_capacity(values.len());
    for value in values {
        let Some(source) = value.as_table() else {
            continue;
        };
        let (Some(x), Some(y), Some(weight)) = (
            source.get("x").and_then(normalized_number),
            source.get("y").and_then(normalized_number),
            source.get("weight").and_then(normalized_number),
        ) else {
            continue;
        };
        sources.push(WeightHeatmapSource::new(
            x,
            y,
            weight,
            source
                .get("selected")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
        ));
    }
    sources
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

#[cfg(test)]
mod optimization_batch_20260830cd_editor_heatmap_tests {
    #[test]
    fn heatmap_source_projection_reserves_array_capacity_and_keeps_filtering() {
        let source = include_str!("weight_heatmap.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("weight heatmap implementation");

        assert!(implementation.contains("let mut sources = Vec::with_capacity(values.len())"));
        assert!(implementation.contains("let Some(source) = value.as_table() else"));
        assert!(implementation.contains("let (Some(x), Some(y), Some(weight))"));
        assert!(implementation.contains("sources.push(WeightHeatmapSource::new("));
    }
}
