use std::collections::BTreeMap;

use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialLightingModel, RenderMaterialValidationError,
};

const LIGHTING_MODEL_PROPERTY: &str = "lighting_model";
const CAST_SHADOWS_PROPERTY: &str = "cast_shadows";
const RECEIVE_SHADOWS_PROPERTY: &str = "receive_shadows";
const RENDER_QUEUE_PROPERTY: &str = "render_queue";
const MATERIAL_QUEUE_PROPERTY: &str = "material_queue";
const DEPTH_BIAS_PROPERTY: &str = "depth_bias";
const TAA_REACTIVE_MASK_STRENGTH_PROPERTY: &str = "taa_reactive_mask_strength";

pub(super) fn lighting_model(
    values: &BTreeMap<String, toml::Value>,
) -> Option<RenderMaterialLightingModel> {
    let value = values.get(LIGHTING_MODEL_PROPERTY)?;
    value
        .as_str()
        .and_then(|value| value.parse::<RenderMaterialLightingModel>().ok())
}

pub(super) fn cast_shadows(values: &BTreeMap<String, toml::Value>) -> Option<bool> {
    override_bool(values, CAST_SHADOWS_PROPERTY)
}

pub(super) fn receive_shadows(values: &BTreeMap<String, toml::Value>) -> Option<bool> {
    override_bool(values, RECEIVE_SHADOWS_PROPERTY)
}

pub(super) fn render_queue(values: &BTreeMap<String, toml::Value>) -> Option<i32> {
    override_i32(values, RENDER_QUEUE_PROPERTY)
}

pub(super) fn material_queue(values: &BTreeMap<String, toml::Value>) -> Option<i32> {
    override_i32(values, MATERIAL_QUEUE_PROPERTY)
}

pub(super) fn depth_bias(values: &BTreeMap<String, toml::Value>) -> Option<f32> {
    override_f32(values, DEPTH_BIAS_PROPERTY)
}

pub(super) fn taa_reactive_mask_strength(values: &BTreeMap<String, toml::Value>) -> Option<f32> {
    override_f32(values, TAA_REACTIVE_MASK_STRENGTH_PROPERTY)
        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
}

pub(super) fn validation_errors(
    values: &BTreeMap<String, toml::Value>,
) -> Vec<RenderMaterialValidationError> {
    let mut errors = Vec::new();
    errors.extend(lighting_model_validation_errors(values));
    errors.extend(bool_override_validation_errors(
        values,
        CAST_SHADOWS_PROPERTY,
    ));
    errors.extend(bool_override_validation_errors(
        values,
        RECEIVE_SHADOWS_PROPERTY,
    ));
    errors.extend(i32_override_validation_errors(
        values,
        RENDER_QUEUE_PROPERTY,
    ));
    errors.extend(i32_override_validation_errors(
        values,
        MATERIAL_QUEUE_PROPERTY,
    ));
    errors.extend(f32_override_validation_errors(values, DEPTH_BIAS_PROPERTY));
    errors.extend(normalized_f32_override_validation_errors(
        values,
        TAA_REACTIVE_MASK_STRENGTH_PROPERTY,
    ));
    errors
}

pub(super) fn sync_material_control_overrides(
    overrides: &mut BTreeMap<String, toml::Value>,
    source_values: &BTreeMap<String, toml::Value>,
) {
    sync_default_true_bool_override(
        overrides,
        source_values,
        CAST_SHADOWS_PROPERTY,
        cast_shadows(source_values),
    );
    sync_default_true_bool_override(
        overrides,
        source_values,
        RECEIVE_SHADOWS_PROPERTY,
        receive_shadows(source_values),
    );
    sync_i32_override(
        overrides,
        source_values,
        RENDER_QUEUE_PROPERTY,
        render_queue(source_values),
        0,
    );
    sync_i32_override(
        overrides,
        source_values,
        MATERIAL_QUEUE_PROPERTY,
        material_queue(source_values),
        0,
    );
    sync_f32_override(
        overrides,
        source_values,
        DEPTH_BIAS_PROPERTY,
        depth_bias(source_values),
        0.0,
    );
    sync_f32_override(
        overrides,
        source_values,
        TAA_REACTIVE_MASK_STRENGTH_PROPERTY,
        taa_reactive_mask_strength(source_values),
        0.0,
    );
}

pub(super) fn is_material_owned_property(name: &str) -> bool {
    matches!(
        name,
        LIGHTING_MODEL_PROPERTY
            | CAST_SHADOWS_PROPERTY
            | RECEIVE_SHADOWS_PROPERTY
            | RENDER_QUEUE_PROPERTY
            | MATERIAL_QUEUE_PROPERTY
            | DEPTH_BIAS_PROPERTY
            | TAA_REACTIVE_MASK_STRENGTH_PROPERTY
    )
}

fn lighting_model_validation_errors(
    values: &BTreeMap<String, toml::Value>,
) -> Vec<RenderMaterialValidationError> {
    let Some(value) = values.get(LIGHTING_MODEL_PROPERTY) else {
        return Vec::new();
    };
    let Some(token) = value.as_str() else {
        return vec![RenderMaterialValidationError::InvalidLightingModel {
            path: format!("overrides.{LIGHTING_MODEL_PROPERTY}"),
            value: value.to_string(),
        }];
    };
    if token.parse::<RenderMaterialLightingModel>().is_ok() {
        Vec::new()
    } else {
        vec![RenderMaterialValidationError::InvalidLightingModel {
            path: format!("overrides.{LIGHTING_MODEL_PROPERTY}"),
            value: token.to_string(),
        }]
    }
}

fn bool_override_validation_errors(
    values: &BTreeMap<String, toml::Value>,
    property: &str,
) -> Vec<RenderMaterialValidationError> {
    let Some(value) = values.get(property) else {
        return Vec::new();
    };
    if value.as_bool().is_some() {
        Vec::new()
    } else {
        type_mismatch_error(property, "bool")
    }
}

fn i32_override_validation_errors(
    values: &BTreeMap<String, toml::Value>,
    property: &str,
) -> Vec<RenderMaterialValidationError> {
    let Some(_) = values.get(property) else {
        return Vec::new();
    };
    if override_i32(values, property).is_some() {
        Vec::new()
    } else {
        type_mismatch_error(property, "i32")
    }
}

fn f32_override_validation_errors(
    values: &BTreeMap<String, toml::Value>,
    property: &str,
) -> Vec<RenderMaterialValidationError> {
    let Some(_) = values.get(property) else {
        return Vec::new();
    };
    if override_f32(values, property).is_some() {
        Vec::new()
    } else {
        type_mismatch_error(property, "number")
    }
}

fn normalized_f32_override_validation_errors(
    values: &BTreeMap<String, toml::Value>,
    property: &str,
) -> Vec<RenderMaterialValidationError> {
    let Some(_) = values.get(property) else {
        return Vec::new();
    };
    let Some(value) = override_f32(values, property) else {
        return type_mismatch_error(property, "number in 0..=1");
    };
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Vec::new()
    } else {
        type_mismatch_error(property, "number in 0..=1")
    }
}

fn type_mismatch_error(property: &str, expected: &str) -> Vec<RenderMaterialValidationError> {
    vec![
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source: RenderMaterialDiagnosticSource::MaterialOverride,
            path: format!("overrides.{property}"),
            name: property.to_string(),
            expected: expected.to_string(),
        },
    ]
}

fn override_f32(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<f32> {
    values
        .get(key)
        .and_then(|value| {
            value
                .as_float()
                .or_else(|| value.as_integer().map(|value| value as f64))
        })
        .map(|value| value as f32)
}

fn override_i32(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<i32> {
    values
        .get(key)
        .and_then(toml::Value::as_integer)
        .and_then(|value| i32::try_from(value).ok())
}

pub(super) fn override_bool(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<bool> {
    values.get(key).and_then(toml::Value::as_bool)
}

fn sync_default_true_bool_override(
    values: &mut BTreeMap<String, toml::Value>,
    source_values: &BTreeMap<String, toml::Value>,
    key: &str,
    value: Option<bool>,
) {
    match value {
        Some(false) => {
            values.insert(key.to_string(), toml::Value::Boolean(false));
        }
        Some(true) => {
            values.remove(key);
        }
        None if !source_values.contains_key(key) => {
            values.remove(key);
        }
        None => {}
    }
}

fn sync_i32_override(
    values: &mut BTreeMap<String, toml::Value>,
    source_values: &BTreeMap<String, toml::Value>,
    key: &str,
    value: Option<i32>,
    default: i32,
) {
    match value {
        Some(value) if value != default => {
            values.insert(key.to_string(), toml::Value::Integer(i64::from(value)));
        }
        Some(_) => {
            values.remove(key);
        }
        None if !source_values.contains_key(key) => {
            values.remove(key);
        }
        None => {}
    }
}

fn sync_f32_override(
    values: &mut BTreeMap<String, toml::Value>,
    source_values: &BTreeMap<String, toml::Value>,
    key: &str,
    value: Option<f32>,
    default: f32,
) {
    match value {
        Some(value) if (value - default).abs() > f32::EPSILON => {
            values.insert(key.to_string(), toml::Value::Float(value as f64));
        }
        Some(_) => {
            values.remove(key);
        }
        None if !source_values.contains_key(key) => {
            values.remove(key);
        }
        None => {}
    }
}
