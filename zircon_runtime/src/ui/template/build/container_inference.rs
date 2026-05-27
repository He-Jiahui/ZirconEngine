use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::layout::{
    UiContainerKind, UiGridBoxConfig, UiLinearBoxConfig, UiMasonryBoxConfig,
};

const MUI_DEFAULT_SPACING_UNIT: f32 = 8.0;

pub(super) fn infer_container(
    component: &str,
    attributes: &BTreeMap<String, Value>,
) -> UiContainerKind {
    match component {
        "Container" => UiContainerKind::Container,
        "Overlay" => UiContainerKind::Overlay,
        "Space" => UiContainerKind::Space,
        "HorizontalBox" | "HorizontalGroup" => UiContainerKind::HorizontalBox(Default::default()),
        "VerticalBox" | "VerticalGroup" | "ListView" => {
            UiContainerKind::VerticalBox(Default::default())
        }
        "ScrollableBox" => UiContainerKind::ScrollableBox(Default::default()),
        "WrapBox" => UiContainerKind::WrapBox(Default::default()),
        "FlowBox" | "FlexBox" => UiContainerKind::WrapBox(Default::default()),
        "GridBox" | "GridGroup" => UiContainerKind::GridBox(Default::default()),
        "Grid" if bool_attribute(attributes, "container") => {
            UiContainerKind::GridBox(mui_grid_config(attributes))
        }
        "Stack" => mui_stack_container(attributes),
        "Masonry" | "MasonryBox" => UiContainerKind::MasonryBox(mui_masonry_config(attributes)),
        "CanvasBox" => UiContainerKind::Free,
        "SizeBox" => UiContainerKind::SizeBox(Default::default()),
        _ => UiContainerKind::Free,
    }
}

fn mui_grid_config(attributes: &BTreeMap<String, Value>) -> UiGridBoxConfig {
    let spacing = responsive_f32_attribute(attributes, &["spacing"]).unwrap_or(0.0);
    UiGridBoxConfig {
        columns: responsive_usize_attribute(attributes, &["columns"])
            .unwrap_or(12)
            .max(1),
        rows: 1,
        column_gap: responsive_f32_attribute(attributes, &["columnSpacing", "column_spacing"])
            .unwrap_or(spacing)
            .max(0.0),
        row_gap: responsive_f32_attribute(attributes, &["rowSpacing", "row_spacing"])
            .unwrap_or(spacing)
            .max(0.0),
    }
}

fn mui_stack_container(attributes: &BTreeMap<String, Value>) -> UiContainerKind {
    let gap = responsive_f32_attribute(attributes, &["spacing"])
        .unwrap_or(0.0)
        .max(0.0);
    let config = UiLinearBoxConfig { gap };
    match responsive_string_attribute(attributes, &["direction"]).as_deref() {
        Some("row") | Some("row-reverse") => UiContainerKind::HorizontalBox(config),
        _ => UiContainerKind::VerticalBox(config),
    }
}

fn mui_masonry_config(attributes: &BTreeMap<String, Value>) -> UiMasonryBoxConfig {
    UiMasonryBoxConfig {
        columns: responsive_usize_attribute(attributes, &["columns"])
            .unwrap_or(UiMasonryBoxConfig::default().columns)
            .max(1),
        gap: responsive_f32_attribute(attributes, &["spacing"])
            .unwrap_or(0.0)
            .max(0.0),
        sequential: bool_attribute(attributes, "sequential"),
    }
}

fn bool_attribute(attributes: &BTreeMap<String, Value>, name: &str) -> bool {
    attributes
        .get(name)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn responsive_usize_attribute(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> Option<usize> {
    names
        .iter()
        .find_map(|name| responsive_base_value(attributes.get(*name)))
        .and_then(value_as_usize)
}

fn responsive_f32_attribute(attributes: &BTreeMap<String, Value>, names: &[&str]) -> Option<f32> {
    names
        .iter()
        .find_map(|name| responsive_base_value(attributes.get(*name)))
        .and_then(value_as_f32)
}

fn responsive_string_attribute(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> Option<String> {
    names
        .iter()
        .find_map(|name| responsive_base_value(attributes.get(*name)))
        .and_then(value_as_string)
}

fn responsive_base_value(value: Option<&Value>) -> Option<&Value> {
    match value? {
        Value::Table(values) => values
            .get("xs")
            .or_else(|| values.get("sm"))
            .or_else(|| values.get("md"))
            .or_else(|| values.get("lg"))
            .or_else(|| values.get("xl")),
        Value::Array(values) => values.first(),
        scalar => Some(scalar),
    }
}

fn value_as_usize(value: &Value) -> Option<usize> {
    match value {
        Value::Integer(value) => usize::try_from(*value).ok(),
        Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        Value::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

fn value_as_f32(value: &Value) -> Option<f32> {
    match value {
        Value::Integer(value) => Some(*value as f32 * MUI_DEFAULT_SPACING_UNIT),
        Value::Float(value) if value.is_finite() => Some(*value as f32 * MUI_DEFAULT_SPACING_UNIT),
        Value::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

fn value_as_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
