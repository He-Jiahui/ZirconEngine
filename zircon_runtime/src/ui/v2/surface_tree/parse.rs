//! 树构建的布局字段解析共用资源身份和节点路径，便于把失败还原到作者的声明位置。

use toml::Value;
use zircon_runtime_interface::ui::v2::UiV2AssetError;

use crate::ui::layout::MAX_UI_LAYOUT_DISCRETE_VALUE;

pub(super) fn layout_table<'a>(
    asset_id: &str,
    value: &'a Value,
    path: &str,
    field: &str,
) -> Result<&'a toml::map::Map<String, Value>, UiV2AssetError> {
    value
        .as_table()
        .ok_or_else(|| invalid_layout_contract(asset_id, path, format!("{field} must be a table")))
}

/// 点表允许省略单个轴并使用零值；类型错误的轴当前也视为缺省，调用方不要据此推断字段已严格校验。
pub(super) fn parse_point(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<Option<(f32, f32)>, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let table = layout_table(asset_id, value, path, field)?;
    Ok(Some((
        parse_f32(table.get("x")).unwrap_or(0.0),
        parse_f32(table.get("y")).unwrap_or(0.0),
    )))
}

pub(super) fn parse_bool(value: Option<&Value>) -> Option<bool> {
    value.and_then(Value::as_bool)
}

pub(super) fn parse_f32(value: Option<&Value>) -> Option<f32> {
    value.and_then(|value| match value {
        Value::Float(value) => Some(*value as f32),
        Value::Integer(value) => Some(*value as f32),
        _ => None,
    })
}

// Layout order, z-index, and priority values use i32 in the surface tree. Values outside
// the i32 range are clamped rather than silently truncating via `as i32`, which would flip
// the sign at i32::MAX + 1 and corrupt sort order.
pub(super) fn parse_i32(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<Option<i32>, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_integer()
        .map(|value| {
            i32::try_from(value).unwrap_or_else(|_| if value > 0 { i32::MAX } else { i32::MIN })
        })
        .ok_or_else(|| {
            invalid_layout_contract(asset_id, path, format!("{field} must be an integer"))
        })
        .map(Some)
}

/// 对会决定轨道数量、跨度和缓冲区规模的资源整数施加共同上限；应在进入布局算法前失败。
pub(super) fn parse_usize(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<Option<usize>, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let parsed = value
        .as_integer()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            invalid_layout_contract(
                asset_id,
                path,
                format!("{field} must be a non-negative integer"),
            )
        })?;
    if parsed > MAX_UI_LAYOUT_DISCRETE_VALUE {
        return Err(invalid_layout_contract(
            asset_id,
            path,
            format!("{field} must not exceed {MAX_UI_LAYOUT_DISCRETE_VALUE}"),
        ));
    }
    Ok(Some(parsed))
}

pub(super) fn invalid_layout_contract(
    asset_id: &str,
    node_path: &str,
    detail: impl Into<String>,
) -> UiV2AssetError {
    UiV2AssetError::InvalidDocument {
        asset_id: asset_id.to_string(),
        detail: format!("invalid layout contract at {node_path}: {}", detail.into()),
    }
}

#[cfg(test)]
#[path = "tests/parse.rs"]
mod tests;
