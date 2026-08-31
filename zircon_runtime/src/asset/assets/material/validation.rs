use crate::asset::{ShaderAsset, ShaderMaterialPropertyAsset};
use crate::core::framework::render::{
    MaterialPropertyKind, RenderMaterialAlphaMode, RenderMaterialDiagnosticSource,
    RenderMaterialValidationError, RenderQueueValue, StandardMaterialDescriptor,
    STANDARD_MATERIAL_TEXTURE_UV_CHANNEL_COUNT,
};

use super::{is_standard_texture_slot_alias, AlphaMode, MaterialAsset, ZMaterialQueueOverride};

const MATERIAL_QUEUE_OFFSET_MIN: i16 = -100;
const MATERIAL_QUEUE_OFFSET_MAX: i16 = 100;

pub fn validate_alpha_mode(alpha_mode: &AlphaMode) -> Vec<RenderMaterialValidationError> {
    match alpha_mode {
        AlphaMode::Mask { cutoff } if !cutoff.is_finite() || !(0.0..=1.0).contains(cutoff) => {
            vec![RenderMaterialValidationError::InvalidMaskCutoff { cutoff: *cutoff }]
        }
        _ => Vec::new(),
    }
}

pub fn validate_render_queue_alpha_mode(
    alpha_mode: &AlphaMode,
    authored_queue: Option<i32>,
) -> Vec<RenderMaterialValidationError> {
    let Some(authored_queue) = authored_queue else {
        return Vec::new();
    };
    let render_alpha_mode = RenderMaterialAlphaMode::from(alpha_mode);
    let render_queue = RenderQueueValue::from_authored_queue(&render_alpha_mode, authored_queue);
    if matches!(alpha_mode, AlphaMode::Blend)
        && render_queue.raw() <= RenderQueueValue::GEOMETRY_LAST.raw()
    {
        vec![
            RenderMaterialValidationError::RenderQueueAlphaModeConflict {
                source: RenderMaterialDiagnosticSource::MaterialOverride,
                path: "overrides.render_queue".to_string(),
                alpha_mode: "blend".to_string(),
                render_queue: render_queue.raw(),
                expected: format!(
                    "transparent material queue greater than {}",
                    RenderQueueValue::GEOMETRY_LAST.raw()
                ),
            },
        ]
    } else {
        Vec::new()
    }
}

pub fn validate_standard_material_texture_uv_channels(
    descriptor: &StandardMaterialDescriptor,
) -> Vec<RenderMaterialValidationError> {
    descriptor
        .unsupported_texture_uv_channels()
        .into_iter()
        .map(
            |(slot, channel)| RenderMaterialValidationError::UnsupportedTextureUvChannel {
                slot: slot.to_string(),
                channel,
                supported_channel_count: STANDARD_MATERIAL_TEXTURE_UV_CHANNEL_COUNT,
            },
        )
        .collect()
}

pub fn validate_shader_contract(
    material: &MaterialAsset,
    shader: &ShaderAsset,
) -> Vec<RenderMaterialValidationError> {
    let mut errors = Vec::new();
    if !shader.kind.participates_in_material_variants() {
        errors.push(RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source: RenderMaterialDiagnosticSource::ShaderReadiness,
            path: "shader.kind".to_string(),
            diagnostic: format!(
                "material requires a surface shader, found {}",
                shader.kind.token()
            ),
        });
    }
    for (name, value) in material.shader_property_overrides() {
        match shader
            .material_property_layout
            .properties
            .iter()
            .find(|property| property.name == *name)
        {
            Some(property) if !material_property_kind_accepts_value(property.kind, value) => errors
                .push(
                    RenderMaterialValidationError::PropertyOverrideTypeMismatch {
                        source: RenderMaterialDiagnosticSource::ShaderSchema,
                        path: format!("overrides.{name}"),
                        name: name.clone(),
                        expected: property.kind.to_string(),
                    },
                ),
            Some(_) => {}
            None if is_standard_material_override(name) => {}
            None if value.as_str().is_some() => {}
            None => {
                errors.push(RenderMaterialValidationError::UnknownPropertyOverride {
                    source: RenderMaterialDiagnosticSource::MaterialOverride,
                    path: format!("overrides.{name}"),
                    name: name.clone(),
                });
            }
        }
    }
    for schema in &shader.property_schema {
        if schema.required && material.shader_property_override(&schema.name).is_none() {
            errors.push(RenderMaterialValidationError::MissingRequiredProperty {
                source: RenderMaterialDiagnosticSource::ShaderSchema,
                path: format!("overrides.{}", schema.name),
                name: schema.name.clone(),
            });
        }
    }

    for (name, value) in material.material_option_values() {
        match shader.material_option_table.option(name) {
            Some(option) if option.value_bits(value).is_none() => {
                errors.push(RenderMaterialValidationError::MaterialOptionTypeMismatch {
                    source: RenderMaterialDiagnosticSource::ShaderSchema,
                    path: format!("options.{name}"),
                    name: name.clone(),
                    expected: option.expected_value_description(),
                });
            }
            Some(_) => {}
            None => errors.push(RenderMaterialValidationError::UnknownMaterialOption {
                source: RenderMaterialDiagnosticSource::MaterialOverride,
                path: format!("options.{name}"),
                name: name.clone(),
            }),
        }
    }

    for slot in material.texture_slots.keys() {
        if is_standard_texture_slot_alias(slot) {
            continue;
        }
        if !shader
            .material_property_layout
            .texture_bindings
            .iter()
            .any(|binding| binding.name == *slot)
        {
            errors.push(RenderMaterialValidationError::UnknownTextureSlot {
                source: RenderMaterialDiagnosticSource::TextureSlot,
                path: format!("textures.{slot}"),
                slot: slot.clone(),
            });
        }
    }
    for schema in &shader.texture_slots {
        let missing_reference = material
            .texture_slots
            .get(&schema.name)
            .and_then(|slot| slot.reference.as_ref())
            .is_none();
        if schema.required && missing_reference {
            errors.push(RenderMaterialValidationError::MissingRequiredTextureSlot {
                source: RenderMaterialDiagnosticSource::ShaderSchema,
                path: format!("textures.{}", schema.name),
                slot: schema.name.clone(),
            });
        }
    }
    errors
}

fn is_standard_material_override(name: &str) -> bool {
    matches!(
        name,
        "base_color" | "metallic" | "roughness" | "emissive" | "alpha_mode" | "double_sided"
    )
}

pub fn validate_material_queue_override(
    queue: Option<ZMaterialQueueOverride>,
) -> Vec<RenderMaterialValidationError> {
    let Some(queue) = queue else {
        return Vec::new();
    };
    if (MATERIAL_QUEUE_OFFSET_MIN..=MATERIAL_QUEUE_OFFSET_MAX).contains(&queue.offset) {
        Vec::new()
    } else {
        vec![RenderMaterialValidationError::InvalidMaterialQueueOffset {
            source: RenderMaterialDiagnosticSource::MaterialOverride,
            path: "queue.offset".to_string(),
            offset: queue.offset,
            expected: "offset between -100 and 100".to_string(),
        }]
    }
}

pub fn validate_wgsl_captures(shader: &ShaderAsset) -> Vec<RenderMaterialValidationError> {
    let Some(source) = shader.runtime_wgsl_source() else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    for property in &shader.property_schema {
        if !captures_name(source, property) {
            errors.push(RenderMaterialValidationError::MissingWgslCapture {
                source: RenderMaterialDiagnosticSource::WgslCapture,
                path: format!("properties.{}", property.name),
                name: property.name.clone(),
            });
        }
    }
    for slot in &shader.texture_slots {
        if !source.contains(&slot.name) {
            errors.push(RenderMaterialValidationError::MissingWgslCapture {
                source: RenderMaterialDiagnosticSource::WgslCapture,
                path: format!("texture_slots.{}", slot.name),
                name: slot.name.clone(),
            });
        }
    }
    errors
}

fn captures_name(source: &str, property: &ShaderMaterialPropertyAsset) -> bool {
    source.contains(&property.name)
}

fn material_property_kind_accepts_value(kind: MaterialPropertyKind, value: &toml::Value) -> bool {
    match kind {
        MaterialPropertyKind::Bool => value.as_bool().is_some(),
        MaterialPropertyKind::Float => value.as_float().is_some() || value.as_integer().is_some(),
        MaterialPropertyKind::Int => value
            .as_integer()
            .and_then(|value| i32::try_from(value).ok())
            .is_some(),
        MaterialPropertyKind::UInt => value
            .as_integer()
            .and_then(|value| u32::try_from(value).ok())
            .is_some(),
        MaterialPropertyKind::Color | MaterialPropertyKind::Vec4 => numeric_array_len(value, 4),
        MaterialPropertyKind::Vec3 => numeric_array_len(value, 3),
        MaterialPropertyKind::Vec2 => numeric_array_len(value, 2),
    }
}

fn numeric_array_len(value: &toml::Value, len: usize) -> bool {
    value.as_array().is_some_and(|items| {
        items.len() == len
            && items
                .iter()
                .all(|item| item.as_float().is_some() || item.as_integer().is_some())
    })
}
