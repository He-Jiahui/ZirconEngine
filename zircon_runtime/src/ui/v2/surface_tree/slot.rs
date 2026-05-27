use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::{
    Anchor, Pivot, Position, UiAlignment, UiAlignment2D, UiCanvasSlotPlacement, UiContainerKind,
    UiGridSlotPlacement, UiLinearSlotSizeRule, UiLinearSlotSizing, UiMargin, UiSlot, UiSlotKind,
};
use zircon_runtime_interface::ui::v2::UiV2AssetError;

use super::parse::{
    invalid_layout_contract, layout_table, parse_bool, parse_f32, parse_i32, parse_point,
    parse_usize,
};

const RESPONSIVE_BREAKPOINTS: &[&str] = &["xs", "sm", "md", "lg", "xl"];

pub(super) fn infer_slot_contract(
    asset_id: &str,
    path: &str,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    parent_container: UiContainerKind,
    slot_attributes: &BTreeMap<String, Value>,
    attributes: &BTreeMap<String, Value>,
) -> Result<UiSlot, UiV2AssetError> {
    let mut slot = UiSlot::new(parent_id, child_id, infer_slot_kind(parent_container));
    let layout = slot_attributes
        .get("layout")
        .map(|layout| layout_table(asset_id, layout, path, "slot.layout"))
        .transpose()?;
    if let Some(layout) = layout {
        slot = slot
            .with_padding(parse_margin(
                asset_id,
                layout.get("padding"),
                path,
                "slot.padding",
            )?)
            .with_alignment(parse_alignment(
                asset_id,
                layout.get("alignment"),
                path,
                "slot.alignment",
            )?)
            .with_order(
                parse_i32(asset_id, layout.get("order"), path, "slot.order")?.unwrap_or_default(),
            );
        if slot.kind == UiSlotKind::Linear {
            if let Some(linear_sizing) =
                parse_linear_sizing(asset_id, layout.get("linear_size"), path)?
            {
                slot = slot.with_linear_sizing(linear_sizing);
            }
        }
        if parent_container == UiContainerKind::Free {
            if let Some(placement) = parse_canvas_placement(asset_id, layout, path)? {
                slot = slot.with_canvas_placement(placement);
            }
        }
        if slot.kind == UiSlotKind::Overlay {
            slot = slot.with_z_order(
                parse_i32(asset_id, layout.get("z_order"), path, "slot.z_order")?
                    .unwrap_or_default(),
            );
        }
        if slot.kind == UiSlotKind::Grid {
            slot = slot.with_grid_placement(parse_grid_placement(asset_id, layout, path)?);
        }
    } else if slot.kind == UiSlotKind::Grid {
        if let Some(placement) = mui_grid_item_placement(attributes) {
            slot = slot.with_grid_placement(placement);
        }
    }
    Ok(slot)
}

fn infer_slot_kind(parent_container: UiContainerKind) -> UiSlotKind {
    match parent_container {
        UiContainerKind::Free => UiSlotKind::Free,
        UiContainerKind::Container | UiContainerKind::SizeBox(_) => UiSlotKind::Container,
        UiContainerKind::Overlay => UiSlotKind::Overlay,
        UiContainerKind::Space => UiSlotKind::Free,
        UiContainerKind::HorizontalBox(_) | UiContainerKind::VerticalBox(_) => UiSlotKind::Linear,
        UiContainerKind::WrapBox(_) | UiContainerKind::MasonryBox(_) => UiSlotKind::Flow,
        UiContainerKind::GridBox(_) => UiSlotKind::Grid,
        UiContainerKind::ScrollableBox(_) => UiSlotKind::Scrollable,
    }
}

fn parse_canvas_placement(
    asset_id: &str,
    layout: &toml::map::Map<String, Value>,
    path: &str,
) -> Result<Option<UiCanvasSlotPlacement>, UiV2AssetError> {
    let has_placement = layout.contains_key("anchor")
        || layout.contains_key("pivot")
        || layout.contains_key("position")
        || layout.contains_key("offset")
        || layout.contains_key("auto_size");
    if !has_placement {
        return Ok(None);
    }

    let anchor = parse_point(asset_id, layout.get("anchor"), path, "slot.anchor")?
        .map(|(x, y)| Anchor::new(x, y))
        .unwrap_or_default();
    let pivot = parse_point(asset_id, layout.get("pivot"), path, "slot.pivot")?
        .map(|(x, y)| Pivot::new(x, y))
        .unwrap_or_default();
    let position = parse_point(asset_id, layout.get("position"), path, "slot.position")?
        .map(|(x, y)| Position::new(x, y))
        .unwrap_or_default();
    Ok(Some(
        UiCanvasSlotPlacement::new(anchor, pivot, position)
            .with_offset(parse_margin(
                asset_id,
                layout.get("offset"),
                path,
                "slot.offset",
            )?)
            .with_auto_size(parse_bool(layout.get("auto_size")).unwrap_or(false)),
    ))
}

fn parse_grid_placement(
    asset_id: &str,
    layout: &toml::map::Map<String, Value>,
    path: &str,
) -> Result<UiGridSlotPlacement, UiV2AssetError> {
    Ok(UiGridSlotPlacement::new(
        parse_usize(asset_id, layout.get("column"), path, "slot.column")?.unwrap_or(0),
        parse_usize(asset_id, layout.get("row"), path, "slot.row")?.unwrap_or(0),
    )
    .with_span(
        parse_usize(
            asset_id,
            layout.get("column_span"),
            path,
            "slot.column_span",
        )?
        .unwrap_or(1),
        parse_usize(asset_id, layout.get("row_span"), path, "slot.row_span")?.unwrap_or(1),
    ))
}

fn parse_margin(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<UiMargin, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(UiMargin::default());
    };
    let table = layout_table(asset_id, value, path, field)?;
    Ok(UiMargin::new(
        parse_f32(table.get("left")).unwrap_or(0.0),
        parse_f32(table.get("top")).unwrap_or(0.0),
        parse_f32(table.get("right")).unwrap_or(0.0),
        parse_f32(table.get("bottom")).unwrap_or(0.0),
    ))
}

fn parse_alignment(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<UiAlignment2D, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(UiAlignment2D::default());
    };
    let table = layout_table(asset_id, value, path, field)?;
    Ok(UiAlignment2D::new(
        parse_alignment_axis(
            asset_id,
            table.get("horizontal"),
            path,
            "slot.alignment.horizontal",
        )?
        .unwrap_or(UiAlignment::Start),
        parse_alignment_axis(
            asset_id,
            table.get("vertical"),
            path,
            "slot.alignment.vertical",
        )?
        .unwrap_or(UiAlignment::Start),
    ))
}

fn parse_alignment_axis(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<Option<UiAlignment>, UiV2AssetError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Start" => UiAlignment::Start,
        "Center" => UiAlignment::Center,
        "End" => UiAlignment::End,
        "Fill" => UiAlignment::Fill,
        other => {
            return Err(invalid_layout_contract(
                asset_id,
                path,
                format!("unsupported {field} {other}"),
            ));
        }
    }))
}

fn parse_linear_sizing(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
) -> Result<Option<UiLinearSlotSizing>, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let table = layout_table(asset_id, value, path, "slot.linear_size")?;
    let rule = parse_linear_size_rule(asset_id, table.get("rule"), path)?;
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
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
) -> Result<UiLinearSlotSizeRule, UiV2AssetError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(UiLinearSlotSizeRule::Stretch);
    };
    Ok(match value {
        "Auto" => UiLinearSlotSizeRule::Auto,
        "Stretch" => UiLinearSlotSizeRule::Stretch,
        "StretchContent" => UiLinearSlotSizeRule::StretchContent,
        other => {
            return Err(invalid_layout_contract(
                asset_id,
                path,
                format!("unsupported slot.linear_size.rule {other}"),
            ));
        }
    })
}

fn mui_grid_item_placement(attributes: &BTreeMap<String, Value>) -> Option<UiGridSlotPlacement> {
    let span = responsive_usize_attribute(attributes, &["size"])?;
    let column = responsive_usize_attribute(attributes, &["offset"]).unwrap_or(0);
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
