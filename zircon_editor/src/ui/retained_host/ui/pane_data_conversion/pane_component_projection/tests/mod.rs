use std::collections::BTreeMap;

use super::*;
use crate::ui::retained_host::primitives::Color;
use crate::ui::template_runtime::{RetainedUiHostBindingProjection, RetainedUiHostNodeProjection};
use toml::Value;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::style::UiStyleColor;

mod actions;
mod collection;
mod mui_surface_defaults;
mod mui_variants;
mod visual_metadata;
mod world_space;

fn projected_node(
    component: &str,
    attributes: impl IntoIterator<Item = (&'static str, Value)>,
) -> RetainedUiHostNodeProjection {
    RetainedUiHostNodeProjection {
        node_id: format!("{component}Node"),
        parent_id: None,
        component: component.to_owned(),
        control_id: Some(format!("{component}Control")),
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
