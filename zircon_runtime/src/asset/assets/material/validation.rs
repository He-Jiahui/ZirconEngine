use crate::asset::{ShaderAsset, ShaderMaterialPropertyAsset};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
};

use super::{AlphaMode, MaterialAsset};

pub fn validate_alpha_mode(alpha_mode: &AlphaMode) -> Vec<RenderMaterialValidationError> {
    match alpha_mode {
        AlphaMode::Mask { cutoff } if !cutoff.is_finite() || !(0.0..=1.0).contains(cutoff) => {
            vec![RenderMaterialValidationError::InvalidMaskCutoff { cutoff: *cutoff }]
        }
        _ => Vec::new(),
    }
}

pub fn validate_shader_contract(
    material: &MaterialAsset,
    shader: &ShaderAsset,
) -> Vec<RenderMaterialValidationError> {
    let mut errors = Vec::new();
    for (name, value) in material.property_overrides() {
        match shader
            .property_schema
            .iter()
            .find(|schema| schema.name == *name)
        {
            Some(schema) if !schema.accepts_value(value) => errors.push(
                RenderMaterialValidationError::PropertyOverrideTypeMismatch {
                    source: RenderMaterialDiagnosticSource::ShaderSchema,
                    path: format!("overrides.{name}"),
                    name: name.clone(),
                    expected: schema.kind.clone(),
                },
            ),
            Some(_) => {}
            None => errors.push(RenderMaterialValidationError::UnknownPropertyOverride {
                source: RenderMaterialDiagnosticSource::MaterialOverride,
                path: format!("overrides.{name}"),
                name: name.clone(),
            }),
        }
    }
    for schema in &shader.property_schema {
        if schema.required && !material.property_overrides().contains_key(&schema.name) {
            errors.push(RenderMaterialValidationError::MissingRequiredProperty {
                source: RenderMaterialDiagnosticSource::ShaderSchema,
                path: format!("overrides.{}", schema.name),
                name: schema.name.clone(),
            });
        }
    }

    for slot in material.texture_slots.keys() {
        if !shader
            .texture_slots
            .iter()
            .any(|schema| schema.name == *slot)
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
