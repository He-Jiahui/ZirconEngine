use crate::core::framework::script::{ScriptHostCallContext, ScriptHostError, ScriptHostValue};
use crate::core::math::Vec3;
use crate::core::resource::{AssetUuid, ResourceHandle, ResourceId};

pub(super) fn expect_string(
    context: &ScriptHostCallContext,
    index: usize,
) -> Result<String, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::String(value)) => Ok(value.clone()),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected string, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
}

pub(super) fn expect_entity(
    context: &ScriptHostCallContext,
    index: usize,
) -> Result<u64, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::Int(value)) if *value >= 0 => Ok(*value as u64),
        Some(ScriptHostValue::HostHandle(value)) => Ok(*value),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected entity id, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
}

pub(super) fn expect_float(
    context: &ScriptHostCallContext,
    index: usize,
) -> Result<f32, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::Float(value)) => Ok(*value as f32),
        Some(ScriptHostValue::Int(value)) => Ok(*value as f32),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected float, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
}

pub(super) fn expect_bool(
    context: &ScriptHostCallContext,
    index: usize,
) -> Result<bool, ScriptHostError> {
    match context.arguments.get(index) {
        Some(ScriptHostValue::Bool(value)) => Ok(*value),
        Some(value) => Err(ScriptHostError::new(format!(
            "argument {index} expected bool, received {:?}",
            value.kind()
        ))),
        None => Err(ScriptHostError::new(format!(
            "argument {index} was not provided"
        ))),
    }
}

pub(super) fn expect_vec3_json(
    context: &ScriptHostCallContext,
    index: usize,
) -> Result<Vec3, ScriptHostError> {
    let value = expect_string(context, index)?;
    vec3_from_json(&value)
}

pub(super) fn parse_key_code(key: &str) -> Option<u32> {
    key.strip_prefix("KeyCode:")
        .and_then(|value| value.parse::<u32>().ok())
        .or_else(|| key.parse::<u32>().ok())
}

pub(super) fn vec3_from_json(value: &str) -> Result<Vec3, ScriptHostError> {
    let array = serde_json::from_str::<[f32; 3]>(value).map_err(json_error)?;
    Ok(Vec3::new(array[0], array[1], array[2]))
}

pub(super) fn vec3_to_array(value: Vec3) -> [f32; 3] {
    [value.x, value.y, value.z]
}

pub(super) fn resource_handle_from_script_ref<T>(value: &str) -> ResourceHandle<T> {
    let value = value.trim();
    let id = value
        .parse::<AssetUuid>()
        .map(ResourceId::from_asset_uuid)
        .or_else(|_| value.parse::<ResourceId>())
        .unwrap_or_else(|_| ResourceId::from_stable_label(value));
    ResourceHandle::new(id)
}

pub(super) fn to_json_string<T: serde::Serialize>(value: &T) -> Result<String, ScriptHostError> {
    serde_json::to_string(value).map_err(json_error)
}

pub(super) fn json_error(error: serde_json::Error) -> ScriptHostError {
    ScriptHostError::new(format!("invalid JSON payload: {error}"))
}

pub(super) fn script_core_error(error: crate::core::CoreError) -> ScriptHostError {
    ScriptHostError::new(error.to_string())
}
