use std::collections::BTreeMap;

use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError, StandardPbrMaterialFeatures,
    STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS, STANDARD_PBR_DEFAULT_IOR,
    STANDARD_PBR_NO_ATTENUATION_DISTANCE,
};

use super::{override_f32, override_vec3, texture_slot_reference, MaterialAsset};

const CLEARCOAT_PROPERTY: &str = "clearcoat";
const CLEARCOAT_ROUGHNESS_PROPERTY: &str = "clearcoat_perceptual_roughness";
const ANISOTROPY_STRENGTH_PROPERTY: &str = "anisotropy_strength";
const ANISOTROPY_ROTATION_PROPERTY: &str = "anisotropy_rotation";
const SPECULAR_TRANSMISSION_PROPERTY: &str = "specular_transmission";
const DIFFUSE_TRANSMISSION_PROPERTY: &str = "diffuse_transmission";
const THICKNESS_PROPERTY: &str = "thickness";
const IOR_PROPERTY: &str = "ior";
const ATTENUATION_COLOR_PROPERTY: &str = "attenuation_color";
const ATTENUATION_DISTANCE_PROPERTY: &str = "attenuation_distance";

impl MaterialAsset {
    pub fn advanced_pbr_features(&self) -> StandardPbrMaterialFeatures {
        StandardPbrMaterialFeatures {
            clearcoat: override_f32(&self.property_values, CLEARCOAT_PROPERTY).unwrap_or(0.0),
            clearcoat_perceptual_roughness: override_f32(
                &self.property_values,
                CLEARCOAT_ROUGHNESS_PROPERTY,
            )
            .unwrap_or(STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS),
            clearcoat_normal_texture: texture_slot_reference(
                &self.texture_slots,
                "clearcoat_normal",
            )
            .or_else(|| texture_slot_reference(&self.texture_slots, "clearcoat_normal_texture")),
            anisotropy_strength: override_f32(&self.property_values, ANISOTROPY_STRENGTH_PROPERTY)
                .unwrap_or(0.0),
            anisotropy_rotation: override_f32(&self.property_values, ANISOTROPY_ROTATION_PROPERTY)
                .unwrap_or(0.0),
            specular_transmission: override_f32(
                &self.property_values,
                SPECULAR_TRANSMISSION_PROPERTY,
            )
            .unwrap_or(0.0),
            diffuse_transmission: override_f32(
                &self.property_values,
                DIFFUSE_TRANSMISSION_PROPERTY,
            )
            .unwrap_or(0.0),
            thickness: override_f32(&self.property_values, THICKNESS_PROPERTY).unwrap_or(0.0),
            ior: override_f32(&self.property_values, IOR_PROPERTY)
                .unwrap_or(STANDARD_PBR_DEFAULT_IOR),
            attenuation_color: override_vec3(&self.property_values, ATTENUATION_COLOR_PROPERTY)
                .unwrap_or([1.0; 3]),
            attenuation_distance: override_f32(
                &self.property_values,
                ATTENUATION_DISTANCE_PROPERTY,
            )
            .unwrap_or(STANDARD_PBR_NO_ATTENUATION_DISTANCE),
        }
        .normalized()
    }
}

pub(super) fn is_material_owned_property(name: &str) -> bool {
    matches!(
        name,
        CLEARCOAT_PROPERTY
            | CLEARCOAT_ROUGHNESS_PROPERTY
            | ANISOTROPY_STRENGTH_PROPERTY
            | ANISOTROPY_ROTATION_PROPERTY
            | SPECULAR_TRANSMISSION_PROPERTY
            | DIFFUSE_TRANSMISSION_PROPERTY
            | THICKNESS_PROPERTY
            | IOR_PROPERTY
            | ATTENUATION_COLOR_PROPERTY
            | ATTENUATION_DISTANCE_PROPERTY
    )
}

pub(super) fn validation_errors(
    values: &BTreeMap<String, toml::Value>,
) -> Vec<RenderMaterialValidationError> {
    let mut errors = Vec::new();
    for property in [
        CLEARCOAT_PROPERTY,
        CLEARCOAT_ROUGHNESS_PROPERTY,
        ANISOTROPY_STRENGTH_PROPERTY,
        SPECULAR_TRANSMISSION_PROPERTY,
        DIFFUSE_TRANSMISSION_PROPERTY,
    ] {
        validate_f32(
            values,
            property,
            "finite number in 0..=1",
            |value| (0.0..=1.0).contains(&value),
            &mut errors,
        );
    }
    validate_f32(
        values,
        ANISOTROPY_ROTATION_PROPERTY,
        "finite number",
        |_| true,
        &mut errors,
    );
    validate_f32(
        values,
        THICKNESS_PROPERTY,
        "non-negative finite number",
        |value| value >= 0.0,
        &mut errors,
    );
    validate_f32(
        values,
        IOR_PROPERTY,
        "finite number greater than or equal to 1",
        |value| value >= 1.0,
        &mut errors,
    );
    validate_f32(
        values,
        ATTENUATION_DISTANCE_PROPERTY,
        "finite number greater than zero",
        |value| value > 0.0,
        &mut errors,
    );
    if let Some(value) = values.get(ATTENUATION_COLOR_PROPERTY) {
        let valid = value.as_array().is_some_and(|channels| {
            channels.len() == 3
                && channels.iter().all(|channel| {
                    toml_number_as_f32(channel).is_some_and(|channel| {
                        channel.is_finite() && (0.0..=1.0).contains(&channel)
                    })
                })
        });
        if !valid {
            errors.push(type_mismatch(
                ATTENUATION_COLOR_PROPERTY,
                "three finite numbers in 0..=1",
            ));
        }
    }
    errors
}

fn validate_f32(
    values: &BTreeMap<String, toml::Value>,
    property: &str,
    expected: &str,
    predicate: impl FnOnce(f32) -> bool,
    errors: &mut Vec<RenderMaterialValidationError>,
) {
    let Some(value) = values.get(property) else {
        return;
    };
    let valid =
        toml_number_as_f32(value).is_some_and(|value| value.is_finite() && predicate(value));
    if !valid {
        errors.push(type_mismatch(property, expected));
    }
}

fn toml_number_as_f32(value: &toml::Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}

fn type_mismatch(property: &str, expected: &str) -> RenderMaterialValidationError {
    RenderMaterialValidationError::PropertyOverrideTypeMismatch {
        source: RenderMaterialDiagnosticSource::MaterialOverride,
        path: format!("overrides.{property}"),
        name: property.to_string(),
        expected: expected.to_string(),
    }
}
