use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_float_array;
use super::super::attribute_values::vec_component;

pub(super) struct ProjectedWorldTransform {
    pub(super) position_x: f32,
    pub(super) position_y: f32,
    pub(super) position_z: f32,
    pub(super) rotation_x: f32,
    pub(super) rotation_y: f32,
    pub(super) rotation_z: f32,
    pub(super) scale_x: f32,
    pub(super) scale_y: f32,
    pub(super) scale_z: f32,
}

pub(super) fn projected_world_transform(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedWorldTransform {
    let position = attributes
        .get("world_position")
        .and_then(value_as_float_array)
        .unwrap_or_default();
    let rotation = attributes
        .get("world_rotation")
        .and_then(value_as_float_array)
        .unwrap_or_default();
    let scale = attributes
        .get("world_scale")
        .and_then(value_as_float_array)
        .unwrap_or_else(|| vec![1.0, 1.0, 1.0]);

    ProjectedWorldTransform {
        position_x: vec_component(&position, 0, 0.0),
        position_y: vec_component(&position, 1, 0.0),
        position_z: vec_component(&position, 2, 0.0),
        rotation_x: vec_component(&rotation, 0, 0.0),
        rotation_y: vec_component(&rotation, 1, 0.0),
        rotation_z: vec_component(&rotation, 2, 0.0),
        scale_x: vec_component(&scale, 0, 1.0),
        scale_y: vec_component(&scale, 1, 1.0),
        scale_z: vec_component(&scale, 2, 1.0),
    }
}
