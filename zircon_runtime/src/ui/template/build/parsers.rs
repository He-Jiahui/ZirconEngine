use crate::ui::layout::{AxisConstraint, LayoutBoundary, StretchMode};
use toml::Value;

use crate::ui::{
    layout::UiAxis, layout::UiContainerKind, layout::UiLinearBoxConfig,
    layout::UiScrollableBoxConfig, layout::UiScrollbarVisibility, layout::UiVirtualListConfig,
    tree::UiInputPolicy,
};

use super::build_error::UiTemplateBuildError;

pub(super) fn parse_axis_constraint(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<AxisConstraint, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(AxisConstraint::default());
    };
    let table = value
        .as_table()
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: format!("{field} must be a table"),
        })?;
    Ok(AxisConstraint {
        min: parse_f32(table.get("min")).unwrap_or(0.0),
        max: parse_f32(table.get("max")).unwrap_or(-1.0),
        preferred: parse_f32(table.get("preferred")).unwrap_or(0.0),
        priority: parse_i32(table.get("priority"), node_path, field)?.unwrap_or(0),
        weight: parse_f32(table.get("weight")).unwrap_or(1.0),
        stretch_mode: parse_stretch_mode(table.get("stretch"), node_path, field)?
            .unwrap_or_default(),
    })
}

pub(super) fn parse_point(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<Option<(f32, f32)>, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let table = value
        .as_table()
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: format!("{field} must be a table"),
        })?;
    Ok(Some((
        parse_f32(table.get("x")).unwrap_or(0.0),
        parse_f32(table.get("y")).unwrap_or(0.0),
    )))
}

pub(super) fn parse_container(
    value: Option<&Value>,
    node_path: &str,
) -> Result<Option<UiContainerKind>, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let table = value
        .as_table()
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: "container must be a table".to_string(),
        })?;
    let Some(kind) = table.get("kind").and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match kind {
        "Free" => UiContainerKind::Free,
        "Container" => UiContainerKind::Container,
        "Overlay" => UiContainerKind::Overlay,
        "Space" => UiContainerKind::Space,
        "HorizontalBox" => UiContainerKind::HorizontalBox(UiLinearBoxConfig {
            gap: parse_f32(table.get("gap")).unwrap_or(0.0),
        }),
        "VerticalBox" => UiContainerKind::VerticalBox(UiLinearBoxConfig {
            gap: parse_f32(table.get("gap")).unwrap_or(0.0),
        }),
        "ScrollableBox" => UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
            axis: parse_axis(table.get("axis"), node_path, "container.axis")?.unwrap_or_default(),
            gap: parse_f32(table.get("gap")).unwrap_or(0.0),
            scrollbar_visibility: parse_scrollbar_visibility(
                table.get("scrollbar_visibility"),
                node_path,
                "container.scrollbar_visibility",
            )?
            .unwrap_or_default(),
            virtualization: parse_virtualization(table.get("virtualization"), node_path)?,
        }),
        other => {
            return Err(UiTemplateBuildError::InvalidLayoutContract {
                node_path: node_path.to_string(),
                detail: format!("unsupported container kind {other}"),
            });
        }
    }))
}

fn parse_virtualization(
    value: Option<&Value>,
    node_path: &str,
) -> Result<Option<UiVirtualListConfig>, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let table = value
        .as_table()
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: "virtualization must be a table".to_string(),
        })?;
    Ok(Some(UiVirtualListConfig {
        item_extent: parse_f32(table.get("item_extent")).unwrap_or(0.0),
        overscan: parse_usize(
            table.get("overscan"),
            node_path,
            "container.virtualization.overscan",
        )?
        .unwrap_or(0),
    }))
}

pub(super) fn parse_layout_boundary(
    value: Option<&Value>,
    node_path: &str,
) -> Result<Option<LayoutBoundary>, UiTemplateBuildError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "ContentDriven" => LayoutBoundary::ContentDriven,
        "ParentDirected" => LayoutBoundary::ParentDirected,
        "Fixed" => LayoutBoundary::Fixed,
        other => {
            return Err(UiTemplateBuildError::InvalidLayoutContract {
                node_path: node_path.to_string(),
                detail: format!("unsupported layout boundary {other}"),
            });
        }
    }))
}

pub(super) fn parse_input_policy(
    value: Option<&Value>,
    node_path: &str,
) -> Result<Option<UiInputPolicy>, UiTemplateBuildError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Inherit" => UiInputPolicy::Inherit,
        "Receive" => UiInputPolicy::Receive,
        "Ignore" => UiInputPolicy::Ignore,
        other => {
            return Err(UiTemplateBuildError::InvalidLayoutContract {
                node_path: node_path.to_string(),
                detail: format!("unsupported input policy {other}"),
            });
        }
    }))
}

fn parse_stretch_mode(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<Option<StretchMode>, UiTemplateBuildError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Fixed" => StretchMode::Fixed,
        "Stretch" => StretchMode::Stretch,
        other => {
            return Err(UiTemplateBuildError::InvalidLayoutContract {
                node_path: node_path.to_string(),
                detail: format!("unsupported {field}.stretch {other}"),
            });
        }
    }))
}

fn parse_axis(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<Option<UiAxis>, UiTemplateBuildError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Horizontal" => UiAxis::Horizontal,
        "Vertical" => UiAxis::Vertical,
        other => {
            return Err(UiTemplateBuildError::InvalidLayoutContract {
                node_path: node_path.to_string(),
                detail: format!("unsupported {field} {other}"),
            });
        }
    }))
}

fn parse_scrollbar_visibility(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<Option<UiScrollbarVisibility>, UiTemplateBuildError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Always" => UiScrollbarVisibility::Always,
        "Never" => UiScrollbarVisibility::Never,
        "Auto" => UiScrollbarVisibility::Auto,
        other => {
            return Err(UiTemplateBuildError::InvalidLayoutContract {
                node_path: node_path.to_string(),
                detail: format!("unsupported {field} {other}"),
            });
        }
    }))
}

pub(super) fn parse_bool(value: Option<&Value>) -> Option<bool> {
    value.and_then(Value::as_bool)
}

fn parse_f32(value: Option<&Value>) -> Option<f32> {
    value.and_then(|value| match value {
        Value::Float(value) => Some(*value as f32),
        Value::Integer(value) => Some(*value as f32),
        _ => None,
    })
}

pub(super) fn parse_i32(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<Option<i32>, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_integer()
        .map(|value| value as i32)
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: format!("{field} must be an integer"),
        })
        .map(Some)
}

fn parse_usize(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<Option<usize>, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_integer()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: format!("{field} must be a non-negative integer"),
        })
        .map(Some)
}
