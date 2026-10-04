use std::collections::{HashMap, HashSet};

use crate::asset::{ShaderAsset, ShaderMaterialPropertyAsset};
use crate::core::framework::render::{
    MaterialPropertyKind, RenderMaterialAlphaMode, RenderMaterialDiagnosticSource,
    RenderMaterialTextureDimension, RenderMaterialValidationError, RenderQueueValue,
    StandardMaterialDescriptor, STANDARD_MATERIAL_TEXTURE_UV_CHANNEL_COUNT,
};

#[cfg(any(feature = "graphics", feature = "target-server"))]
use crate::core::framework::render::strip_wgsl_include_directives;
#[cfg(feature = "graphics")]
use crate::graphics::shader::{
    reflect_declared_shader_resources, reflect_validated_shader_module, ShaderBindingResourceType,
    ShaderStageVisibility, ShaderTextureSampleType, ShaderTextureViewDimension,
};
#[cfg(all(feature = "target-server", not(feature = "graphics")))]
#[path = "target_server_reflection.rs"]
mod target_server_reflection;
#[cfg(any(feature = "graphics", feature = "target-server"))]
use naga::front::wgsl;
#[cfg(any(feature = "graphics", feature = "target-server"))]
use naga::valid::{Capabilities, ValidationFlags, Validator};
#[cfg(all(feature = "target-server", not(feature = "graphics")))]
use target_server_reflection::{
    reflect_declared_shader_resources, reflect_validated_shader_module, ShaderBindingResourceType,
    ShaderResourceBindingReflection, ShaderStageVisibility, ShaderTextureSampleType,
    ShaderTextureViewDimension,
};

#[cfg(feature = "graphics")]
type MaterialShaderResourceBindingReflection =
    crate::graphics::shader::template::ShaderResourceBindingReflection;
#[cfg(all(feature = "target-server", not(feature = "graphics")))]
type MaterialShaderResourceBindingReflection = ShaderResourceBindingReflection;

use super::{is_standard_texture_slot_alias, AlphaMode, MaterialAsset, ZMaterialQueueOverride};

const MATERIAL_QUEUE_OFFSET_MIN: i16 = -100;
const MATERIAL_QUEUE_OFFSET_MAX: i16 = 100;
const SHADER_CONTRACT_INDEX_MIN_COMPARISONS: usize = 64;
const SHADER_CONTRACT_INDEX_MIN_CANDIDATES: usize = 8;
const SHADER_CONTRACT_INDEX_MIN_LOOKUPS: usize = 8;

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
    let properties = &shader.material_property_layout.properties;
    let property_override_count = material.shader_property_overrides().count();
    let properties_by_name =
        should_index_contract_lookup(properties.len(), property_override_count).then(|| {
            let mut index = HashMap::with_capacity(properties.len());
            for property in properties {
                index.entry(property.name.as_str()).or_insert(property);
            }
            index
        });
    for (name, value) in material.shader_property_overrides() {
        let property = match &properties_by_name {
            Some(properties_by_name) => properties_by_name.get(name.as_str()).copied(),
            None => properties.iter().find(|property| property.name == *name),
        };
        match property {
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

    let texture_bindings = &shader.material_property_layout.texture_bindings;
    let texture_binding_names =
        should_index_contract_lookup(texture_bindings.len(), material.texture_slots.len()).then(
            || {
                let mut index = HashSet::with_capacity(texture_bindings.len());
                index.extend(texture_bindings.iter().map(|binding| binding.name.as_str()));
                index
            },
        );
    for slot in material.texture_slots.keys() {
        if is_standard_texture_slot_alias(slot) {
            continue;
        }
        let is_known = match &texture_binding_names {
            Some(texture_binding_names) => texture_binding_names.contains(slot.as_str()),
            None => texture_bindings.iter().any(|binding| binding.name == *slot),
        };
        if !is_known {
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

const fn should_index_contract_lookup(candidate_count: usize, lookup_count: usize) -> bool {
    candidate_count >= SHADER_CONTRACT_INDEX_MIN_CANDIDATES
        && lookup_count >= SHADER_CONTRACT_INDEX_MIN_LOOKUPS
        && candidate_count.saturating_mul(lookup_count) >= SHADER_CONTRACT_INDEX_MIN_COMPARISONS
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
    #[cfg(any(feature = "graphics", feature = "target-server"))]
    {
        return validate_reflected_wgsl_captures(shader, source);
    }
    #[cfg(not(any(feature = "graphics", feature = "target-server")))]
    {
        // Capture admission has no textual fallback. A non-render target has no
        // WGSL validator, so it leaves the render admission diagnostic to the
        // graphics importer rather than claiming a false-ready capture.
        let _ = source;
        Vec::new()
    }
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn validate_reflected_wgsl_captures(
    shader: &ShaderAsset,
    source: &str,
) -> Vec<RenderMaterialValidationError> {
    let generated = shader.generated_material_wgsl.trim();
    if generated.is_empty() {
        return Vec::new();
    }
    let expected_visibility = shader_entry_point_visibility(shader);
    let Some((module, resource_bindings, enforce_reachability)) =
        reflect_capture_module(generated, source, &expected_visibility)
    else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    let material_binding = resource_bindings.iter().find(|binding| {
        binding.identity.group == 2
            && binding.identity.binding == 0
            && binding.identity.resource_type == ShaderBindingResourceType::UniformBuffer
            && has_expected_visibility(binding.visibility, &expected_visibility)
    });
    for property in &shader.property_schema {
        let identifier = material_identifier(&property.name);
        let function_name = format!("zr_mat_{identifier}");
        let function_exists = module
            .functions
            .iter()
            .any(|(_, function)| function.name.as_deref() == Some(function_name.as_str()))
            && (!enforce_reachability
                || function_reachable_from_entry_point(&module, function_name.as_str()));
        if !function_exists || material_binding.is_none() {
            push_capture_error(
                &mut errors,
                format!("properties.{}", property.name),
                &property.name,
                find_module_span(&module, "zr_material"),
            );
        }
    }
    for slot in &shader.texture_slots {
        let identifier = material_identifier(&slot.name);
        let texture_binding = shader
            .material_property_layout
            .texture_bindings
            .iter()
            .find(|binding| binding.name == slot.name);
        let Some(texture_binding) = texture_binding else {
            push_capture_error(
                &mut errors,
                format!("texture_slots.{}", slot.name),
                &slot.name,
                None,
            );
            continue;
        };
        let texture_name = format!("zr_tex_{identifier}");
        let sampler_name = format!("zr_smp_{identifier}");
        let texture_ok = resource_bindings.iter().any(|binding| {
            binding.identity.group == 2
                && binding.identity.binding == texture_binding.texture_binding as u32
                && binding.name.as_deref() == Some(texture_name.as_str())
                && texture_binding_matches_kind(&binding.identity.resource_type, &slot.kind)
                && has_expected_visibility(binding.visibility, &expected_visibility)
        });
        let sampler_ok = resource_bindings.iter().any(|binding| {
            binding.identity.group == 2
                && binding.identity.binding == texture_binding.sampler_binding as u32
                && binding.name.as_deref() == Some(sampler_name.as_str())
                && matches!(
                    binding.identity.resource_type,
                    ShaderBindingResourceType::Sampler { .. }
                )
                && has_expected_visibility(binding.visibility, &expected_visibility)
        });
        let sample_function_name = format!("zr_sample_{identifier}");
        let sample_function_exists =
            module.functions.iter().any(|(_, function)| {
                function.name.as_deref() == Some(sample_function_name.as_str())
            }) && (!enforce_reachability
                || function_reachable_from_entry_point(&module, sample_function_name.as_str()));
        if !texture_ok || !sampler_ok || !sample_function_exists {
            push_capture_error(
                &mut errors,
                format!("texture_slots.{}", slot.name),
                &slot.name,
                find_module_span(&module, texture_name.as_str()),
            );
        }
    }
    errors
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn reflect_generated_material_module(
    generated: &str,
    expected_visibility: &[naga::ShaderStage],
) -> Option<(naga::Module, Vec<MaterialShaderResourceBindingReflection>)> {
    let module = wgsl::parse_str(generated).ok()?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator.validate(&module).ok()?;
    let resources = reflect_declared_shader_resources(&module, expected_visibility);
    Some((module, resources))
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn reflect_capture_module(
    generated: &str,
    source: &str,
    expected_visibility: &[naga::ShaderStage],
) -> Option<(
    naga::Module,
    Vec<MaterialShaderResourceBindingReflection>,
    bool,
)> {
    // A standalone authored module with an entry point can be reflected as a
    // complete shader contract. Surface package functions without an entry
    // point intentionally fall back to declaration reflection below because
    // template types are supplied only during later render assembly.
    let combined = format!("{generated}\n{}", strip_wgsl_include_directives(source));
    if let Ok(module) = wgsl::parse_str(&combined) {
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        if let Ok(module_info) = validator.validate(&module) {
            if !module.entry_points.is_empty() {
                let reflection = reflect_validated_shader_module(&module, &module_info);
                return Some((module, reflection.resource_bindings, true));
            }
        }
    }
    reflect_generated_material_module(generated, expected_visibility)
        .map(|(module, resources)| (module, resources, false))
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn function_reachable_from_entry_point(module: &naga::Module, target_name: &str) -> bool {
    let Some(target) = module.functions.iter().find_map(|(handle, function)| {
        (function.name.as_deref() == Some(target_name)).then_some(handle)
    }) else {
        return false;
    };
    module.entry_points.iter().any(|entry_point| {
        entry_point_function_reaches(module, &entry_point.function, target, &mut HashSet::new())
    })
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn entry_point_function_reaches(
    module: &naga::Module,
    function: &naga::Function,
    target: naga::Handle<naga::Function>,
    visited: &mut HashSet<naga::Handle<naga::Function>>,
) -> bool {
    function.expressions.iter().any(|(_, expression)| {
        let naga::Expression::CallResult(called) = expression else {
            return false;
        };
        if *called == target {
            return true;
        }
        if !visited.insert(*called) {
            return false;
        }
        module
            .functions
            .iter()
            .find_map(|(handle, called_function)| (handle == *called).then_some(called_function))
            .is_some_and(|called_function| {
                entry_point_function_reaches(module, called_function, target, visited)
            })
    })
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn shader_entry_point_visibility(shader: &ShaderAsset) -> Vec<naga::ShaderStage> {
    shader
        .entry_points
        .iter()
        .filter_map(
            |entry| match entry.stage.trim().to_ascii_lowercase().as_str() {
                "vertex" => Some(naga::ShaderStage::Vertex),
                "fragment" | "pixel" => Some(naga::ShaderStage::Fragment),
                "compute" => Some(naga::ShaderStage::Compute),
                _ => None,
            },
        )
        .collect()
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn has_expected_visibility(
    visibility: ShaderStageVisibility,
    expected: &[naga::ShaderStage],
) -> bool {
    expected.is_empty()
        || expected
            .iter()
            .copied()
            .any(|stage| visibility.contains(stage))
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn push_capture_error(
    errors: &mut Vec<RenderMaterialValidationError>,
    path: String,
    name: &str,
    span: Option<naga::Span>,
) {
    let path = match span {
        Some(span) => format!("{path} [naga_span={span:?}]"),
        None => path,
    };
    errors.push(RenderMaterialValidationError::MissingWgslCapture {
        source: RenderMaterialDiagnosticSource::WgslCapture,
        path,
        name: name.to_string(),
    });
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn find_module_span(module: &naga::Module, name: &str) -> Option<naga::Span> {
    module
        .global_variables
        .iter()
        .find(|(_, global)| global.name.as_deref() == Some(name))
        .map(|(handle, _)| module.global_variables.get_span(handle))
        .or_else(|| {
            module
                .functions
                .iter()
                .find(|(_, function)| function.name.as_deref() == Some(name))
                .map(|(handle, _)| module.functions.get_span(handle))
        })
}

fn material_identifier(name: &str) -> String {
    let mut identifier = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    if identifier.is_empty() {
        identifier.push_str("unnamed");
    }
    if identifier
        .as_bytes()
        .first()
        .is_some_and(|first| first.is_ascii_digit())
    {
        identifier.insert(0, '_');
    }
    identifier
}

#[cfg(any(feature = "graphics", feature = "target-server"))]
fn texture_binding_matches_kind(resource_type: &ShaderBindingResourceType, kind: &str) -> bool {
    let ShaderBindingResourceType::SampledTexture {
        view_dimension,
        sample_type,
        ..
    } = resource_type
    else {
        return false;
    };
    let expected = RenderMaterialTextureDimension::from_shader_kind(kind);
    let normalized_kind = kind.trim().to_ascii_lowercase();
    let expected_sample = if normalized_kind.contains("depth") {
        ShaderTextureSampleType::Depth
    } else if normalized_kind.contains("uint") {
        ShaderTextureSampleType::Uint
    } else if normalized_kind.contains("sint") {
        ShaderTextureSampleType::Sint
    } else {
        ShaderTextureSampleType::Float
    };
    let actual_dimension = *view_dimension;
    let actual_sample = *sample_type;
    matches!(
        (expected, actual_dimension, actual_sample),
        (RenderMaterialTextureDimension::D1, ShaderTextureViewDimension::D1, actual)
            | (RenderMaterialTextureDimension::D2, ShaderTextureViewDimension::D2, actual)
            | (RenderMaterialTextureDimension::D2Array, ShaderTextureViewDimension::D2Array, actual)
            | (RenderMaterialTextureDimension::Cube, ShaderTextureViewDimension::Cube, actual)
            | (RenderMaterialTextureDimension::CubeArray, ShaderTextureViewDimension::CubeArray, actual)
            | (RenderMaterialTextureDimension::D3, ShaderTextureViewDimension::D3, actual)
                if actual == expected_sample
    )
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

#[cfg(test)]
#[path = "tests/validation_optimization_batch_ie_runtime616_tests.rs"]
mod optimization_batch_ie_runtime616_tests;
