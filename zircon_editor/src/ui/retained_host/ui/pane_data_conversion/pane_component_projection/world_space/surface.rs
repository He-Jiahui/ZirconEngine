use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::{value_as_f64, value_as_float_array};
use super::super::attribute_values::vec_component;

pub(super) struct ProjectedWorldSurface {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) pixels_per_meter: f32,
}

pub(super) fn projected_world_surface(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedWorldSurface {
    let size = attributes
        .get("world_size")
        .and_then(value_as_float_array)
        .unwrap_or_default();

    ProjectedWorldSurface {
        width: vec_component(&size, 0, 0.0),
        height: vec_component(&size, 1, 0.0),
        pixels_per_meter: attributes
            .get("pixels_per_meter")
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
    }
}
