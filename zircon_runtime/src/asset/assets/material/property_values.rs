use std::collections::BTreeMap;

use crate::asset::{MaterialAsset, ShaderAsset, ShaderMaterialPropertyAsset};
use crate::core::framework::render::RenderMaterialPropertyValue;

pub fn shader_property_values_for_shader(
    material: &MaterialAsset,
    shader: &ShaderAsset,
) -> BTreeMap<String, RenderMaterialPropertyValue> {
    let mut values = BTreeMap::new();
    for property in &shader.property_schema {
        let value = material
            .property_overrides()
            .get(&property.name)
            .or(property.default.as_ref());
        let Some(value) = value else {
            continue;
        };
        if let Some(value) = render_property_value(property, value) {
            values.insert(property.name.clone(), value);
        }
    }
    values
}

fn render_property_value(
    property: &ShaderMaterialPropertyAsset,
    value: &toml::Value,
) -> Option<RenderMaterialPropertyValue> {
    match property.kind.trim().to_ascii_lowercase().as_str() {
        "bool" | "boolean" => value
            .as_bool()
            .map(|value| RenderMaterialPropertyValue::Bool { value }),
        "float" | "f32" | "number" => {
            toml_number_as_f32(value).map(|value| RenderMaterialPropertyValue::Float { value })
        }
        "int" | "i32" | "integer" => value
            .as_integer()
            .and_then(|value| i32::try_from(value).ok())
            .map(|value| RenderMaterialPropertyValue::Int { value }),
        "u32" | "uint" => value
            .as_integer()
            .and_then(|value| u32::try_from(value).ok())
            .map(|value| RenderMaterialPropertyValue::UInt { value }),
        "string" => value
            .as_str()
            .map(|value| RenderMaterialPropertyValue::String {
                value: value.to_string(),
            }),
        "vec2" => {
            numeric_array::<2>(value).map(|value| RenderMaterialPropertyValue::Vec2 { value })
        }
        "vec3" => {
            numeric_array::<3>(value).map(|value| RenderMaterialPropertyValue::Vec3 { value })
        }
        "color" | "color4" | "vec4" => {
            numeric_array::<4>(value).map(|value| RenderMaterialPropertyValue::Vec4 { value })
        }
        _ => None,
    }
}

fn numeric_array<const N: usize>(value: &toml::Value) -> Option<[f32; N]> {
    let items = value.as_array()?;
    if items.len() != N {
        return None;
    }
    let mut values = [0.0; N];
    for (index, item) in items.iter().enumerate() {
        values[index] = toml_number_as_f32(item)?;
    }
    Some(values)
}

fn toml_number_as_f32(value: &toml::Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}
