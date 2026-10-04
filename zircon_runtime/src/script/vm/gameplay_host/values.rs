//! 这里统一把脚本值和资源引用转换为宿主参数；字符串使用借用闭包限定调用帧寿命，实体与数字错误在场景操作前返回。
use crate::core::framework::script::{ScriptHostCallFrame, ScriptHostError, ScriptHostValueRef};
use crate::core::math::Vec3;
use crate::core::resource::{AssetUuid, ResourceHandle, ResourceId};

pub(super) fn with_string<T>(
    context: &ScriptHostCallFrame<'_>,
    index: usize,
    visitor: impl for<'value> FnOnce(&'value str) -> Result<T, ScriptHostError>,
) -> Result<T, ScriptHostError> {
    context.arguments.with_argument(index, |value| match value {
        ScriptHostValueRef::String(value) => visitor(value),
        value => Err(ScriptHostError::new(format!(
            "argument {index} expected string, received {:?}",
            value.kind()
        ))),
    })
}

pub(super) fn expect_entity(
    context: &ScriptHostCallFrame<'_>,
    index: usize,
) -> Result<u64, ScriptHostError> {
    context.arguments.with_argument(index, |value| match value {
        ScriptHostValueRef::Int(value) if value >= 0 => Ok(value as u64),
        ScriptHostValueRef::HostHandle(value) => Ok(value),
        value => Err(ScriptHostError::new(format!(
            "argument {index} expected entity id, received {:?}",
            value.kind()
        ))),
    })
}

pub(super) fn expect_float(
    context: &ScriptHostCallFrame<'_>,
    index: usize,
) -> Result<f32, ScriptHostError> {
    context.arguments.with_argument(index, |value| match value {
        ScriptHostValueRef::Float(value) => Ok(value as f32),
        ScriptHostValueRef::Int(value) => Ok(value as f32),
        value => Err(ScriptHostError::new(format!(
            "argument {index} expected float, received {:?}",
            value.kind()
        ))),
    })
}

pub(super) fn expect_bool(
    context: &ScriptHostCallFrame<'_>,
    index: usize,
) -> Result<bool, ScriptHostError> {
    context.arguments.with_argument(index, |value| match value {
        ScriptHostValueRef::Bool(value) => Ok(value),
        value => Err(ScriptHostError::new(format!(
            "argument {index} expected bool, received {:?}",
            value.kind()
        ))),
    })
}

pub(super) fn expect_vec3_json(
    context: &ScriptHostCallFrame<'_>,
    index: usize,
) -> Result<Vec3, ScriptHostError> {
    with_string(context, index, vec3_from_json)
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

#[cfg(test)]
#[path = "tests/values.rs"]
mod tests;
