use std::collections::BTreeMap;

use toml::Value;
use zircon_ui::UiNodeDefinition;

use super::resolution::{
    UiAssetPaletteHoverContext, UiAssetPaletteNativeSlotTarget, UiAssetPaletteSlotTargetOverlay,
    UiAssetPaletteTargetFrame,
};

pub(super) fn grid_slot_target_overlays(
    node: &UiNodeDefinition,
    frame: UiAssetPaletteTargetFrame,
    selected_slot: &BTreeMap<String, Value>,
) -> Vec<UiAssetPaletteSlotTargetOverlay> {
    let selected_row = selected_slot.get("row").and_then(Value::as_integer);
    let selected_column = selected_slot.get("column").and_then(Value::as_integer);
    grid_slot_targets(
        node,
        UiAssetPaletteHoverContext {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
            surface_x: frame.x + frame.width * 0.5,
            surface_y: frame.y + frame.height * 0.5,
        },
    )
    .into_iter()
    .map(|target| UiAssetPaletteSlotTargetOverlay {
        label: target.label,
        detail: target.detail,
        x: target.x,
        y: target.y,
        width: target.width,
        height: target.height,
        selected: target.slot.get("row").and_then(Value::as_integer) == selected_row
            && target.slot.get("column").and_then(Value::as_integer) == selected_column,
    })
    .collect()
}

pub(super) fn grid_slot_for_hover(
    node: &UiNodeDefinition,
    hover: UiAssetPaletteHoverContext,
) -> BTreeMap<String, Value> {
    let columns = estimated_grid_axis(node, "column", "column_span").max(2);
    let rows = estimated_grid_axis(node, "row", "row_span").max(2);
    let column = ((hover.normalized_x() * columns as f32).floor() as i64).min(columns - 1);
    let row = ((hover.normalized_y() * rows as f32).floor() as i64).min(rows - 1);
    BTreeMap::from([
        ("row".to_string(), Value::Integer(row)),
        ("column".to_string(), Value::Integer(column)),
    ])
}

pub(super) fn grid_slot_targets(
    node: &UiNodeDefinition,
    hover: UiAssetPaletteHoverContext,
) -> Vec<UiAssetPaletteNativeSlotTarget> {
    let columns = estimated_grid_axis(node, "column", "column_span").max(2) as usize;
    let rows = estimated_grid_axis(node, "row", "row_span").max(2) as usize;
    let mut targets = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            targets.push(UiAssetPaletteNativeSlotTarget {
                label: format!("R{row} C{column}"),
                detail: format!("row {row}, column {column}"),
                slot: BTreeMap::from([
                    ("row".to_string(), Value::Integer(row as i64)),
                    ("column".to_string(), Value::Integer(column as i64)),
                ]),
                x: hover.x + hover.width * column as f32 / columns as f32,
                y: hover.y + hover.height * row as f32 / rows as f32,
                width: hover.width / columns as f32,
                height: hover.height / rows as f32,
            });
        }
    }
    targets
}

pub(super) fn estimated_grid_axis(node: &UiNodeDefinition, key: &str, span_key: &str) -> i64 {
    node.children
        .iter()
        .filter_map(|child| {
            let value = child.slot.get(key)?.as_integer()?;
            let span = child
                .slot
                .get(span_key)
                .and_then(Value::as_integer)
                .unwrap_or(1);
            Some(value + span)
        })
        .max()
        .unwrap_or(1)
}
