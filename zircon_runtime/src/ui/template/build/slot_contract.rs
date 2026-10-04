//! 将子节点的作者输入保存为父子边上的放置契约，供布局测量、排序和渲染层级共同消费。
//! slot 的种类由父容器决定；子节点自身的约束与边上的 padding、alignment 由不同所有者持有。

use std::collections::BTreeMap;

use toml::Value;

use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::{
    Anchor, Pivot, Position, UiAlignment, UiAlignment2D, UiCanvasSlotPlacement, UiContainerKind,
    UiGridSlotPlacement, UiLinearSlotSizeRule, UiLinearSlotSizing, UiMargin, UiSlot, UiSlotKind,
};
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::build_error::UiTemplateBuildError;
use super::parsers::{parse_bool, parse_i32, parse_point, parse_usize};

const RESPONSIVE_BREAKPOINTS: &[&str] = &["xs", "sm", "md", "lg", "xl"];

/// 构树在插入子节点前调用；只接纳父容器能消费的放置能力，错误保留实例路径。
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
        if matches!(
            parent_container,
            UiContainerKind::Free | UiContainerKind::Canvas
        ) {
            if let Some(placement) = parse_canvas_placement(layout, path)? {
                slot = slot.with_canvas_placement(placement);
            }
        }
        if matches!(slot.kind, UiSlotKind::Overlay | UiSlotKind::Canvas) {
            slot = slot.with_z_order(
                parse_i32(layout.get("z_order"), path, "slot.z_order")?.unwrap_or_default(),
            );
        }
        // TODO: [CR-UI-TEMPLATE-BUILD-0003] 确认仅声明 slot padding/order 是否仍应采用 MUI size/offset；当前整张 layout 表阻断推断和响应式更新；需补两子节点布局测试。
        if slot.kind == UiSlotKind::Grid {
            slot = slot.with_grid_placement(parse_grid_placement(layout, path)?);
        }
    } else if slot.kind == UiSlotKind::Grid {
        if let Some(placement) = mui_grid_item_placement(&node.attributes) {
            slot = slot.with_grid_placement(placement);
        }
    }
    Ok(slot)
}

// 缺少全部放置字段时保留节点级默认位置；只声明 auto_size 也代表调用方显式选择边上的放置策略。
fn parse_canvas_placement(
    layout: &toml::map::Map<String, Value>,
    node_path: &str,
) -> Result<Option<UiCanvasSlotPlacement>, UiTemplateBuildError> {
    let anchor_value = layout.get("anchor");
    let anchor_max_value = layout.get("anchor_max");
    let pivot_value = layout.get("pivot");
    let position_value = layout.get("position");
    let offset_value = layout.get("offset");
    let auto_size_value = layout.get("auto_size");
    if [
        anchor_value,
        anchor_max_value,
        pivot_value,
        position_value,
        offset_value,
        auto_size_value,
    ]
    .iter()
    .all(Option::is_none)
    {
        return Ok(None);
    }

    let anchor = parse_point(anchor_value, node_path, "slot.anchor")?
        .map(|(x, y)| Anchor::new(x, y))
        .unwrap_or_default();
    let anchor_max = parse_point(anchor_max_value, node_path, "slot.anchor_max")?
        .map(|(x, y)| Anchor::new(x, y));
    let pivot = parse_point(pivot_value, node_path, "slot.pivot")?
        .map(|(x, y)| Pivot::new(x, y))
        .unwrap_or_default();
    let position = parse_point(position_value, node_path, "slot.position")?
        .map(|(x, y)| Position::new(x, y))
        .unwrap_or_default();
    let mut placement = UiCanvasSlotPlacement::new(anchor, pivot, position)
        .with_offset(parse_margin(offset_value, node_path, "slot.offset")?)
        .with_auto_size(parse_bool(auto_size_value).unwrap_or(false));
    if let Some(anchor_max) = anchor_max {
        placement = placement.with_anchor_max(anchor_max);
    }
    Ok(Some(placement))
}

fn infer_slot_kind(parent_container: UiContainerKind) -> UiSlotKind {
    parent_container
        .child_slot_kind()
        .unwrap_or(UiSlotKind::Free)
}

// 显式网格位置走有上限的整数解析，避免布局尺寸和后续轨道分配接受无界作者输入。
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

// 弹性分配是父子边的契约；子节点仍保留自身轴约束，布局执行器组合两者而非改写节点。
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

// MUI Grid offset and size values are user-supplied and must be clamped to reasonable
// grid bounds before they reach the layout engine. An unbounded offset or a zero span
// causes column+span overflow in the measure pass; cap at a value that fits safely in
// the grid's coordinate space.
const MUI_GRID_MAX_COLUMNS: usize = 12;

fn mui_grid_item_placement(attributes: &BTreeMap<String, Value>) -> Option<UiGridSlotPlacement> {
    let span = responsive_usize_attribute(attributes, &["size"])?.clamp(1, MUI_GRID_MAX_COLUMNS);
    let column = responsive_usize_attribute(attributes, &["offset"])
        .unwrap_or(0)
        .clamp(0, MUI_GRID_MAX_COLUMNS.saturating_sub(1));
    Some(UiGridSlotPlacement::new(column, 0).with_span(span, 1))
}

fn responsive_usize_attribute(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> Option<usize> {
    names
        .iter()
        .find_map(|name| responsive_base_value(attributes.get(*name)))
        .and_then(value_as_usize)
}

fn responsive_base_value(value: Option<&Value>) -> Option<&Value> {
    match value? {
        Value::Table(values) => RESPONSIVE_BREAKPOINTS
            .iter()
            .find_map(|breakpoint| values.get(*breakpoint)),
        Value::Array(values) => values.first(),
        scalar => Some(scalar),
    }
}

fn value_as_usize(value: &Value) -> Option<usize> {
    match value {
        Value::Integer(value) => usize::try_from(*value).ok(),
        Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        Value::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
// 性能门槛仅在受管理的 release 测量中启用；普通测试保护缺省放置与显式字段存在性。
// 两个构建入口共享源码守卫，新增说明不能伪造它们检索的调用片段。
#[path = "tests/slot_contract_optimization_tests.rs"]
mod optimization_tests;
