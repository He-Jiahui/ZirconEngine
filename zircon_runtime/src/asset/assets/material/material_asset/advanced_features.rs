use std::collections::BTreeMap;

use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError, StandardPbrMaterialFeatures,
    STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS, STANDARD_PBR_DEFAULT_IOR,
    STANDARD_PBR_NO_ATTENUATION_DISTANCE,
};

use super::{override_f32, override_vec3, texture_slot_reference, MaterialAsset};

const CLEARCOAT_PROPERTY: &str = "clearcoat";
const CLEARCOAT_ROUGHNESS_PROPERTY: &str = "clearcoat_perceptual_roughness";
const CLEARCOAT_NORMAL_SCALE_PROPERTY: &str = "clearcoat_normal_scale";
const ANISOTROPY_STRENGTH_PROPERTY: &str = "anisotropy_strength";
const ANISOTROPY_ROTATION_PROPERTY: &str = "anisotropy_rotation";
const SPECULAR_TRANSMISSION_PROPERTY: &str = "specular_transmission";
const DIFFUSE_TRANSMISSION_PROPERTY: &str = "diffuse_transmission";
const THICKNESS_PROPERTY: &str = "thickness";
const IOR_PROPERTY: &str = "ior";
const ATTENUATION_COLOR_PROPERTY: &str = "attenuation_color";
const ATTENUATION_DISTANCE_PROPERTY: &str = "attenuation_distance";
const ADVANCED_VALIDATED_PROPERTY_COUNT: usize = 11;

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
            clearcoat_normal_scale: override_f32(
                &self.property_values,
                CLEARCOAT_NORMAL_SCALE_PROPERTY,
            )
            .unwrap_or(1.0),
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
            | CLEARCOAT_NORMAL_SCALE_PROPERTY
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
    let mut errors = Vec::with_capacity(values.len().min(ADVANCED_VALIDATED_PROPERTY_COUNT));
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
        CLEARCOAT_NORMAL_SCALE_PROPERTY,
        "finite number",
        |_| true,
        &mut errors,
    );
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

#[cfg(test)]
mod optimization_batch_20260830cj_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const PROPERTY_COUNT: usize = 11;

    #[test]
    fn optimization_batch_20260830cj_runtime_material_validation_reserves_bounded_error_capacity() {
        let source = include_str!("advanced_features.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("advanced material implementation");

        assert!(implementation.contains("const ADVANCED_VALIDATED_PROPERTY_COUNT: usize = 11"));
        assert!(implementation
            .contains("Vec::with_capacity(values.len().min(ADVANCED_VALIDATED_PROPERTY_COUNT))"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cj_runtime_material_validation_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME386_MATERIAL_VALIDATION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} properties_per_sample={PROPERTY_COUNT} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..8_192 {
            let mut errors = if use_capacity {
                Vec::with_capacity(PROPERTY_COUNT)
            } else {
                Vec::new()
            };
            for property in 0..PROPERTY_COUNT {
                errors.push(property);
            }
            checksum ^= errors.len();
            std::hint::black_box(errors);
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
