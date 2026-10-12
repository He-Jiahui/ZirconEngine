use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_f64;

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
    let position = transform_components(attributes.get("world_position"), 0.0);
    let rotation = transform_components(attributes.get("world_rotation"), 0.0);
    let scale = transform_components(attributes.get("world_scale"), 1.0);

    ProjectedWorldTransform {
        position_x: position[0],
        position_y: position[1],
        position_z: position[2],
        rotation_x: rotation[0],
        rotation_y: rotation[1],
        rotation_z: rotation[2],
        scale_x: scale[0],
        scale_y: scale[1],
        scale_z: scale[2],
    }
}

fn transform_components(value: Option<&toml::Value>, default: f32) -> [f32; 3] {
    let mut components = [default; 3];
    let Some(toml::Value::Array(values)) = value else {
        return components;
    };
    for (component, value) in components
        .iter_mut()
        .zip(values.iter().filter_map(value_as_f64))
    {
        *component = value as f32;
    }
    components
}
