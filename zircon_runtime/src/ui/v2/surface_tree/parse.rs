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
mod tests {
    use super::*;

    #[test]
    fn v2_explicit_layout_usize_values_use_the_runtime_layout_bound() {
        let maximum = Value::Integer(MAX_UI_LAYOUT_DISCRETE_VALUE as i64);
        assert_eq!(
            parse_usize("ui.test.bound", Some(&maximum), "root", "container.rows").unwrap(),
            Some(MAX_UI_LAYOUT_DISCRETE_VALUE)
        );

        let oversized = Value::Integer((MAX_UI_LAYOUT_DISCRETE_VALUE + 1) as i64);
        let error =
            parse_usize("ui.test.bound", Some(&oversized), "root", "container.rows").unwrap_err();
        assert!(error.to_string().contains(&format!(
            "container.rows must not exceed {MAX_UI_LAYOUT_DISCRETE_VALUE}"
        )));
    }
}
