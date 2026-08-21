use std::collections::BTreeMap;

use toml::Value;

use super::resolution::{
    quantized_anchor, round_position, slot_table_numeric_value, table_value,
    UiAssetPaletteHoverContext, UiAssetPaletteNativeSlotTarget, UiAssetPaletteSlotTargetOverlay,
    UiAssetPaletteTargetFrame,
};

pub(super) fn overlay_slot_target_overlays(
    frame: UiAssetPaletteTargetFrame,
    selected_slot: &BTreeMap<String, Value>,
) -> Vec<UiAssetPaletteSlotTargetOverlay> {
    let selected_anchor = overlay_slot_anchor(selected_slot);
    overlay_slot_targets(UiAssetPaletteHoverContext {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
        surface_x: frame.x + frame.width * 0.5,
        surface_y: frame.y + frame.height * 0.5,
    })
    .into_iter()
    .map(|target| UiAssetPaletteSlotTargetOverlay {
        label: target.label,
        detail: target.detail,
        x: target.x,
        y: target.y,
        width: target.width,
        height: target.height,
        selected: overlay_slot_anchor(&target.slot) == selected_anchor,
    })
    .collect()
}

pub(super) fn overlay_slot_anchor(slot: &BTreeMap<String, Value>) -> Option<(f32, f32)> {
    Some((
        slot_table_numeric_value(slot.get("layout")?, &["anchor", "x"])?,
        slot_table_numeric_value(slot.get("layout")?, &["anchor", "y"])?,
    ))
}

pub(super) fn overlay_slot_for_hover(hover: UiAssetPaletteHoverContext) -> BTreeMap<String, Value> {
    let anchor_x = quantized_anchor(hover.normalized_x());
    let anchor_y = quantized_anchor(hover.normalized_y());
    overlay_slot_for_anchor(
        hover.x,
        hover.y,
        hover.width,
        hover.height,
        hover.surface_x,
        hover.surface_y,
        anchor_x,
        anchor_y,
    )
}

pub(super) fn overlay_slot_for_anchor(
    frame_x: f32,
    frame_y: f32,
    frame_width: f32,
    frame_height: f32,
    surface_x: f32,
    surface_y: f32,
    anchor_x: f32,
    anchor_y: f32,
) -> BTreeMap<String, Value> {
    let position_x = round_position(surface_x - (frame_x + frame_width * anchor_x));
    let position_y = round_position(surface_y - (frame_y + frame_height * anchor_y));
    let mut slot = BTreeMap::new();
    let _ = slot.insert(
        "layout".to_string(),
        table_value(&[
            (
                "anchor",
                table_value(&[
                    ("x", Value::Float(anchor_x as f64)),
                    ("y", Value::Float(anchor_y as f64)),
                ]),
            ),
            (
                "pivot",
                table_value(&[
                    ("x", Value::Float(anchor_x as f64)),
                    ("y", Value::Float(anchor_y as f64)),
                ]),
            ),
            (
                "position",
                table_value(&[
                    ("x", super::resolution::numeric_value(position_x)),
                    ("y", super::resolution::numeric_value(position_y)),
                ]),
            ),
        ]),
    );
    slot
}

pub(super) fn overlay_slot_targets(
    hover: UiAssetPaletteHoverContext,
) -> Vec<UiAssetPaletteNativeSlotTarget> {
    let names = [
        ["Top Left", "Top", "Top Right"],
        ["Left", "Center", "Right"],
        ["Bottom Left", "Bottom", "Bottom Right"],
    ];
    let mut targets = Vec::new();
    for row in 0..3 {
        for column in 0..3 {
            let anchor_x = [0.0, 0.5, 1.0][column];
            let anchor_y = [0.0, 0.5, 1.0][row];
            targets.push(UiAssetPaletteNativeSlotTarget {
                label: names[row][column].to_string(),
                detail: format!("anchor {}, {}", anchor_x, anchor_y),
                slot: overlay_slot_for_anchor(
                    hover.x,
                    hover.y,
                    hover.width,
                    hover.height,
                    hover.surface_x,
                    hover.surface_y,
                    anchor_x,
                    anchor_y,
                ),
                x: hover.x + hover.width * column as f32 / 3.0,
                y: hover.y + hover.height * row as f32 / 3.0,
                width: hover.width / 3.0,
                height: hover.height / 3.0,
            });
        }
    }
    targets
}
