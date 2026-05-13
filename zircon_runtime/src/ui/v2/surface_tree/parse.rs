use toml::Value;
use zircon_runtime_interface::ui::v2::UiV2AssetError;

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
        .map(|value| value as i32)
        .ok_or_else(|| {
            invalid_layout_contract(asset_id, path, format!("{field} must be an integer"))
        })
        .map(Some)
}

pub(super) fn parse_usize(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<Option<usize>, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_integer()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            invalid_layout_contract(
                asset_id,
                path,
                format!("{field} must be a non-negative integer"),
            )
        })
        .map(Some)
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
