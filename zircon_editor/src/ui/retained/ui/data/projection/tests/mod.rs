use std::collections::BTreeMap;

use super::*;
use crate::ui::retained_host::primitives::Color;
use crate::ui::template_runtime::{RetainedUiHostBindingProjection, RetainedUiHostNodeProjection};
use toml::Value;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::style::UiStyleColor;

#[path = "actions.rs"]
mod actions;
#[path = "collection.rs"]
mod collection;
#[path = "mui_surface_defaults.rs"]
mod mui_surface_defaults;
#[path = "mui_variants.rs"]
mod mui_variants;
#[path = "progress.rs"]
mod progress;
#[path = "sample_grid.rs"]
mod sample_grid;
#[path = "timeline_strip.rs"]
mod timeline_strip;
#[path = "visual_metadata.rs"]
mod visual_metadata;
#[path = "weight_heatmap.rs"]
mod weight_heatmap;
#[path = "world_space.rs"]
mod world_space;

fn projected_node(
    component: &str,
    attributes: impl IntoIterator<Item = (&'static str, Value)>,
) -> RetainedUiHostNodeProjection {
    RetainedUiHostNodeProjection {
        node_id: format!("{component}Node"),
        surface_node_id: None,
        has_workbench_icon_tooltip: false,
        parent_id: None,
        component: component.to_owned(),
        control_id: Some(format!("{component}Control")),
        source_path: None,
        source_node_id: None,
        instance_path: None,
        parent_source_path: None,
        parent_source_node_id: None,
        parent_instance_path: None,
        frame: UiFrame::new(0.0, 0.0, 320.0, 240.0),
        clip_frame: None,
        z_index: 0,
        attributes: attributes
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
        style_overrides: BTreeMap::new(),
        style_tokens: BTreeMap::new(),
        bindings: Vec::new(),
    }
}

fn float_array(values: [f64; 3]) -> Value {
    Value::Array(values.into_iter().map(Value::Float).collect())
}

fn string_array(values: impl Iterator<Item = String>) -> Value {
    Value::Array(values.map(Value::String).collect())
}

fn toml_table(values: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    let mut table = toml::map::Map::new();
    for (name, value) in values {
        table.insert(name.to_owned(), value);
    }
    Value::Table(table)
}

fn assert_variant_token(component_variant: &str, expected: &str) {
    assert!(
        component_variant
            .split_whitespace()
            .any(|part| part == expected),
        "expected component_variant `{component_variant}` to contain `{expected}`"
    );
}

fn style_color_u8(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Role(_) | UiStyleColor::Inherit => None,
    }
}
