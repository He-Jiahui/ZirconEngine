use toml::Value;

use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::{
    Anchor, Pivot, Position, UiAlignment, UiAlignment2D, UiCanvasSlotPlacement, UiContainerKind,
    UiGridSlotPlacement, UiLinearSlotSizeRule, UiLinearSlotSizing, UiMargin, UiScrollableBoxConfig,
    UiSlot, UiSlotKind,
};
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::build_error::UiTemplateBuildError;
use super::parsers::{parse_bool, parse_i32, parse_point, parse_usize};

pub(super) fn infer_slot_contract(
    node: &UiTemplateNode,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    parent_container: UiContainerKind,
    path: &str,
) -> Result<UiSlot, UiTemplateBuildError> {
    let mut slot = UiSlot::new(parent_id, child_id, infer_slot_kind(parent_container));
    let layout = node.slot_attributes.get("layout").and_then(Value::as_table);
    if let Some(layout) = layout {
        slot = slot
            .with_padding(parse_margin(layout.get("padding"), path, "slot.padding")?)
            .with_alignment(parse_alignment(
                layout.get("alignment"),
                path,
                "slot.alignment",
            )?)
            .with_order(parse_i32(layout.get("order"), path, "slot.order")?.unwrap_or_default());
        if slot.kind == UiSlotKind::Linear {
            if let Some(linear_sizing) = parse_linear_sizing(layout.get("linear_size"), path)? {
                slot = slot.with_linear_sizing(linear_sizing);
            }
        }
        if parent_container == UiContainerKind::Free {
            if let Some(placement) = parse_canvas_placement(layout, path)? {
                slot = slot.with_canvas_placement(placement);
            }
        }
        if slot.kind == UiSlotKind::Overlay {
            slot = slot.with_z_order(
                parse_i32(layout.get("z_order"), path, "slot.z_order")?.unwrap_or_default(),
            );
        }
        if slot.kind == UiSlotKind::Grid {
            slot = slot.with_grid_placement(parse_grid_placement(layout, path)?);
        }
    }
    Ok(slot)
}

fn parse_canvas_placement(
    layout: &toml::map::Map<String, Value>,
    node_path: &str,
) -> Result<Option<UiCanvasSlotPlacement>, UiTemplateBuildError> {
    let has_placement = layout.contains_key("anchor")
        || layout.contains_key("pivot")
        || layout.contains_key("position")
        || layout.contains_key("offset")
        || layout.contains_key("auto_size");
    if !has_placement {
        return Ok(None);
    }

    let anchor = parse_point(layout.get("anchor"), node_path, "slot.anchor")?
        .map(|(x, y)| Anchor::new(x, y))
        .unwrap_or_default();
    let pivot = parse_point(layout.get("pivot"), node_path, "slot.pivot")?
        .map(|(x, y)| Pivot::new(x, y))
        .unwrap_or_default();
    let position = parse_point(layout.get("position"), node_path, "slot.position")?
        .map(|(x, y)| Position::new(x, y))
        .unwrap_or_default();
    Ok(Some(
        UiCanvasSlotPlacement::new(anchor, pivot, position)
            .with_offset(parse_margin(
                layout.get("offset"),
                node_path,
                "slot.offset",
            )?)
            .with_auto_size(parse_bool(layout.get("auto_size")).unwrap_or(false)),
    ))
}

fn infer_slot_kind(parent_container: UiContainerKind) -> UiSlotKind {
    match parent_container {
        UiContainerKind::Free => UiSlotKind::Free,
        UiContainerKind::Container => UiSlotKind::Container,
        UiContainerKind::Overlay => UiSlotKind::Overlay,
        UiContainerKind::Space => UiSlotKind::Free,
        UiContainerKind::HorizontalBox(_) | UiContainerKind::VerticalBox(_) => UiSlotKind::Linear,
        UiContainerKind::WrapBox(_) => UiSlotKind::Flow,
        UiContainerKind::GridBox(_) => UiSlotKind::Grid,
        UiContainerKind::ScrollableBox(UiScrollableBoxConfig { .. }) => UiSlotKind::Scrollable,
    }
}

fn parse_grid_placement(
    layout: &toml::map::Map<String, toml::Value>,
    path: &str,
) -> Result<UiGridSlotPlacement, UiTemplateBuildError> {
    Ok(UiGridSlotPlacement::new(
        parse_usize(layout.get("column"), path, "slot.column")?.unwrap_or(0),
        parse_usize(layout.get("row"), path, "slot.row")?.unwrap_or(0),
    )
    .with_span(
        parse_usize(layout.get("column_span"), path, "slot.column_span")?.unwrap_or(1),
        parse_usize(layout.get("row_span"), path, "slot.row_span")?.unwrap_or(1),
    ))
}

fn parse_margin(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<UiMargin, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(UiMargin::default());
    };
    let table = value
        .as_table()
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: format!("{field} must be a table"),
        })?;
    Ok(UiMargin::new(
        parse_f32(table.get("left")).unwrap_or(0.0),
        parse_f32(table.get("top")).unwrap_or(0.0),
        parse_f32(table.get("right")).unwrap_or(0.0),
        parse_f32(table.get("bottom")).unwrap_or(0.0),
    ))
}

fn parse_alignment(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<UiAlignment2D, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(UiAlignment2D::default());
    };
    let table = value
        .as_table()
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: format!("{field} must be a table"),
        })?;
    Ok(UiAlignment2D::new(
        parse_alignment_axis(
            table.get("horizontal"),
            node_path,
            "slot.alignment.horizontal",
        )?
        .unwrap_or(UiAlignment::Start),
        parse_alignment_axis(table.get("vertical"), node_path, "slot.alignment.vertical")?
            .unwrap_or(UiAlignment::Start),
    ))
}

fn parse_alignment_axis(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<Option<UiAlignment>, UiTemplateBuildError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Start" => UiAlignment::Start,
        "Center" => UiAlignment::Center,
        "End" => UiAlignment::End,
        "Fill" => UiAlignment::Fill,
        other => {
            return Err(UiTemplateBuildError::InvalidLayoutContract {
                node_path: node_path.to_string(),
                detail: format!("unsupported {field} {other}"),
            });
        }
    }))
}

fn parse_linear_sizing(
    value: Option<&Value>,
    node_path: &str,
) -> Result<Option<UiLinearSlotSizing>, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let table = value
        .as_table()
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: "slot.linear_size must be a table".to_string(),
        })?;
    let rule = parse_linear_size_rule(table.get("rule"), node_path)?;
    let mut sizing = UiLinearSlotSizing::new(rule);
    if let Some(value) = parse_f32(table.get("value")) {
        sizing = sizing.with_value(value);
    }
    if let Some(value) = parse_f32(table.get("shrink_value")) {
        sizing = sizing.with_shrink_value(value);
    }
    if let Some(value) = parse_f32(table.get("min")) {
        sizing = sizing.with_min(value);
    }
    if let Some(value) = parse_f32(table.get("max")) {
        sizing = sizing.with_max(value);
    }
    Ok(Some(sizing))
}

fn parse_linear_size_rule(
    value: Option<&Value>,
    node_path: &str,
) -> Result<UiLinearSlotSizeRule, UiTemplateBuildError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(UiLinearSlotSizeRule::Stretch);
    };
    Ok(match value {
        "Auto" => UiLinearSlotSizeRule::Auto,
        "Stretch" => UiLinearSlotSizeRule::Stretch,
        "StretchContent" => UiLinearSlotSizeRule::StretchContent,
        other => {
            return Err(UiTemplateBuildError::InvalidLayoutContract {
                node_path: node_path.to_string(),
                detail: format!("unsupported slot.linear_size.rule {other}"),
            });
        }
    })
}

fn parse_f32(value: Option<&Value>) -> Option<f32> {
    value.and_then(|value| match value {
        Value::Float(value) => Some(*value as f32),
        Value::Integer(value) => Some(*value as f32),
        _ => None,
    })
}
