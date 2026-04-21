use std::collections::BTreeMap;

use toml::Value;

use super::resolution::{
    UiAssetPaletteHoverContext, UiAssetPaletteNativeSlotTarget, UiAssetPaletteSlotTargetOverlay,
    UiAssetPaletteTargetFrame,
};

pub(super) fn flow_slot_target_overlays(
    frame: UiAssetPaletteTargetFrame,
    selected_slot: &BTreeMap<String, Value>,
) -> Vec<UiAssetPaletteSlotTargetOverlay> {
    let selected_break_before = selected_slot
        .get("break_before")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let selected_alignment = selected_slot
        .get("alignment")
        .and_then(Value::as_str)
        .unwrap_or("Center");
    flow_slot_targets(UiAssetPaletteHoverContext {
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
        selected: target
            .slot
            .get("break_before")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            == selected_break_before
            && target
                .slot
                .get("alignment")
                .and_then(Value::as_str)
                .unwrap_or("Center")
                == selected_alignment,
    })
    .collect()
}

pub(super) fn flow_slot_for_hover(hover: UiAssetPaletteHoverContext) -> BTreeMap<String, Value> {
    let alignment = if hover.normalized_x() <= 0.33 {
        "Start"
    } else if hover.normalized_x() >= 0.66 {
        "End"
    } else {
        "Center"
    };
    BTreeMap::from([
        (
            "break_before".to_string(),
            Value::Boolean(hover.normalized_y() >= 0.5),
        ),
        (
            "alignment".to_string(),
            Value::String(alignment.to_string()),
        ),
    ])
}

pub(super) fn flow_slot_targets(
    hover: UiAssetPaletteHoverContext,
) -> Vec<UiAssetPaletteNativeSlotTarget> {
    let alignments = ["Start", "Center", "End"];
    let mut targets = Vec::new();
    for row in 0..2 {
        for (column, alignment) in alignments.iter().enumerate() {
            let break_before = row == 1;
            targets.push(UiAssetPaletteNativeSlotTarget {
                label: if break_before {
                    format!("Break {alignment}")
                } else {
                    (*alignment).to_string()
                },
                detail: format!("break_before={break_before}, alignment={alignment}"),
                slot: BTreeMap::from([
                    ("break_before".to_string(), Value::Boolean(break_before)),
                    (
                        "alignment".to_string(),
                        Value::String((*alignment).to_string()),
                    ),
                ]),
                x: hover.x + hover.width * column as f32 / alignments.len() as f32,
                y: hover.y + hover.height * row as f32 / 2.0,
                width: hover.width / alignments.len() as f32,
                height: hover.height / 2.0,
            });
        }
    }
    targets
}
