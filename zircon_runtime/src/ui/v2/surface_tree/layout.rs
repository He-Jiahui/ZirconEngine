use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::layout::{
    Anchor, AxisConstraint, BoxConstraints, LayoutBoundary, Pivot, Position, StretchMode, UiAxis,
    UiContainerKind, UiGridBoxConfig, UiLinearBoxConfig, UiScrollableBoxConfig,
    UiScrollbarVisibility, UiSizeBoxConfig, UiVirtualListConfig, UiWrapBoxConfig,
};
use zircon_runtime_interface::ui::tree::UiInputPolicy;
use zircon_runtime_interface::ui::v2::UiV2AssetError;

use super::parse::{
    invalid_layout_contract, layout_table, parse_bool, parse_f32, parse_i32, parse_point,
    parse_usize,
};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct V2LayoutContract {
    pub(super) constraints: BoxConstraints,
    pub(super) anchor: Anchor,
    pub(super) pivot: Pivot,
    pub(super) position: Position,
    pub(super) container: Option<UiContainerKind>,
    pub(super) input_policy: Option<UiInputPolicy>,
    pub(super) clip_to_bounds: bool,
    pub(super) layout_boundary: LayoutBoundary,
    pub(super) stretch_width: bool,
    pub(super) stretch_height: bool,
    pub(super) z_index: i32,
}

pub(super) fn infer_layout_contract(
    asset_id: &str,
    path: &str,
    self_layout: Option<&Value>,
    slot_attributes: &BTreeMap<String, Value>,
    parent_container: Option<UiContainerKind>,
) -> Result<V2LayoutContract, UiV2AssetError> {
    let Some(layout) = merged_layout_table(
        asset_id,
        path,
        self_layout,
        slot_attributes,
        parent_container,
    )?
    else {
        return Ok(V2LayoutContract::default());
    };

    Ok(V2LayoutContract {
        constraints: BoxConstraints {
            width: parse_axis_constraint(asset_id, layout.get("width"), path, "width")?,
            height: parse_axis_constraint(asset_id, layout.get("height"), path, "height")?,
        },
        anchor: parse_point(asset_id, layout.get("anchor"), path, "anchor")?
            .map(|(x, y)| Anchor::new(x, y))
            .unwrap_or_default(),
        pivot: parse_point(asset_id, layout.get("pivot"), path, "pivot")?
            .map(|(x, y)| Pivot::new(x, y))
            .unwrap_or_default(),
        position: parse_point(asset_id, layout.get("position"), path, "position")?
            .map(|(x, y)| Position::new(x, y))
            .unwrap_or_default(),
        container: parse_container(asset_id, layout.get("container"), path)?,
        input_policy: parse_input_policy(asset_id, layout.get("input_policy"), path)?,
        clip_to_bounds: parse_bool(layout.get("clip"))
            .or_else(|| parse_bool(layout.get("clip_to_bounds")))
            .unwrap_or(false),
        layout_boundary: parse_layout_boundary(asset_id, layout.get("boundary"), path)?
            .unwrap_or_default(),
        stretch_width: is_explicit_stretch_axis(layout.get("width")),
        stretch_height: is_explicit_stretch_axis(layout.get("height")),
        z_index: parse_i32(asset_id, layout.get("z_index"), path, "z_index")?.unwrap_or_default(),
    })
}

pub(super) fn infer_container(component: &str) -> UiContainerKind {
    match component {
        "Container" => UiContainerKind::Container,
        "Overlay" => UiContainerKind::Overlay,
        "Space" => UiContainerKind::Space,
        "HorizontalBox" | "HorizontalGroup" => UiContainerKind::HorizontalBox(Default::default()),
        "VerticalBox" | "VerticalGroup" | "ListView" => {
            UiContainerKind::VerticalBox(Default::default())
        }
        "ScrollableBox" => UiContainerKind::ScrollableBox(Default::default()),
        "WrapBox" => UiContainerKind::WrapBox(Default::default()),
        "FlowBox" | "FlexBox" => UiContainerKind::WrapBox(Default::default()),
        "GridBox" | "GridGroup" => UiContainerKind::GridBox(Default::default()),
        "CanvasBox" => UiContainerKind::Free,
        "SizeBox" => UiContainerKind::SizeBox(Default::default()),
        _ => UiContainerKind::Free,
    }
}

fn merged_layout_table(
    asset_id: &str,
    path: &str,
    self_layout: Option<&Value>,
    slot_attributes: &BTreeMap<String, Value>,
    parent_container: Option<UiContainerKind>,
) -> Result<Option<toml::map::Map<String, Value>>, UiV2AssetError> {
    let self_layout = self_layout
        .map(|layout| layout_table(asset_id, layout, path, "layout"))
        .transpose()?;
    let slot_layout = slot_attributes
        .get("layout")
        .map(|layout| layout_table(asset_id, layout, path, "slot.layout"))
        .transpose()?;
    match (self_layout, slot_layout) {
        (None, None) => Ok(None),
        (Some(layout), None) | (None, Some(layout)) => Ok(Some(layout.clone())),
        (Some(self_layout), Some(slot_layout)) => {
            let mut merged = self_layout.clone();
            for (key, value) in slot_layout {
                let _ = merged.insert(key.clone(), value.clone());
            }
            match parent_container {
                Some(UiContainerKind::HorizontalBox(_))
                | Some(UiContainerKind::WrapBox(_))
                | Some(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Horizontal,
                    ..
                })) => restore_axis(&mut merged, self_layout, "width"),
                Some(UiContainerKind::VerticalBox(_))
                | Some(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    ..
                })) => restore_axis(&mut merged, self_layout, "height"),
                _ => {}
            }
            Ok(Some(merged))
        }
    }
}

fn restore_axis(
    target: &mut toml::map::Map<String, Value>,
    source: &toml::map::Map<String, Value>,
    axis: &str,
) {
    if let Some(value) = source.get(axis) {
        let _ = target.insert(axis.to_string(), value.clone());
    }
}

fn parse_axis_constraint(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<AxisConstraint, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(AxisConstraint::default());
    };
    let table = layout_table(asset_id, value, path, field)?;
    Ok(AxisConstraint {
        min: parse_f32(table.get("min")).unwrap_or(0.0),
        max: parse_f32(table.get("max")).unwrap_or(-1.0),
        preferred: parse_f32(table.get("preferred")).unwrap_or(0.0),
        priority: parse_i32(asset_id, table.get("priority"), path, field)?.unwrap_or(0),
        weight: parse_f32(table.get("weight")).unwrap_or(1.0),
        stretch_mode: parse_stretch_mode(asset_id, table.get("stretch"), path, field)?
            .unwrap_or_default(),
    })
}

fn parse_container(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
) -> Result<Option<UiContainerKind>, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let table = layout_table(asset_id, value, path, "container")?;
    let Some(kind) = table.get("kind").and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match kind {
        "Free" => UiContainerKind::Free,
        "Container" => UiContainerKind::Container,
        "Overlay" => UiContainerKind::Overlay,
        "Space" => UiContainerKind::Space,
        "SizeBox" => UiContainerKind::SizeBox(UiSizeBoxConfig {
            aspect_ratio: parse_f32(table.get("aspect_ratio"))
                .or_else(|| parse_f32(table.get("ratio")))
                .unwrap_or(0.0),
        }),
        "HorizontalBox" | "HorizontalGroup" => UiContainerKind::HorizontalBox(UiLinearBoxConfig {
            gap: parse_f32(table.get("gap")).unwrap_or(0.0),
        }),
        "VerticalBox" | "VerticalGroup" | "ListView" => {
            UiContainerKind::VerticalBox(UiLinearBoxConfig {
                gap: parse_f32(table.get("gap")).unwrap_or(0.0),
            })
        }
        "ScrollableBox" => UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
            axis: parse_axis(asset_id, table.get("axis"), path, "container.axis")?
                .unwrap_or_default(),
            gap: parse_f32(table.get("gap")).unwrap_or(0.0),
            scrollbar_visibility: parse_scrollbar_visibility(
                asset_id,
                table.get("scrollbar_visibility"),
                path,
                "container.scrollbar_visibility",
            )?
            .unwrap_or_default(),
            virtualization: parse_virtualization(asset_id, table.get("virtualization"), path)?,
        }),
        "WrapBox" => UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: parse_f32(table.get("horizontal_gap")).unwrap_or(0.0),
            vertical_gap: parse_f32(table.get("vertical_gap")).unwrap_or(0.0),
            item_min_width: parse_f32(table.get("item_min_width")).unwrap_or(0.0),
        }),
        "FlowBox" | "FlexBox" => UiContainerKind::WrapBox(UiWrapBoxConfig {
            horizontal_gap: parse_f32(table.get("horizontal_gap"))
                .or_else(|| parse_f32(table.get("gap")))
                .unwrap_or(0.0),
            vertical_gap: parse_f32(table.get("vertical_gap"))
                .or_else(|| parse_f32(table.get("gap")))
                .unwrap_or(0.0),
            item_min_width: parse_f32(table.get("item_min_width")).unwrap_or(0.0),
        }),
        "GridBox" | "GridGroup" => UiContainerKind::GridBox(UiGridBoxConfig {
            columns: parse_usize(asset_id, table.get("columns"), path, "container.columns")?
                .unwrap_or(1),
            rows: parse_usize(asset_id, table.get("rows"), path, "container.rows")?.unwrap_or(1),
            column_gap: parse_f32(table.get("column_gap"))
                .or_else(|| parse_f32(table.get("gap")))
                .unwrap_or(0.0),
            row_gap: parse_f32(table.get("row_gap"))
                .or_else(|| parse_f32(table.get("gap")))
                .unwrap_or(0.0),
        }),
        other => {
            return Err(invalid_layout_contract(
                asset_id,
                path,
                format!("unsupported container kind {other}"),
            ));
        }
    }))
}

fn parse_virtualization(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
) -> Result<Option<UiVirtualListConfig>, UiV2AssetError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let table = layout_table(asset_id, value, path, "virtualization")?;
    Ok(Some(UiVirtualListConfig {
        item_extent: parse_f32(table.get("item_extent")).unwrap_or(0.0),
        overscan: parse_usize(
            asset_id,
            table.get("overscan"),
            path,
            "container.virtualization.overscan",
        )?
        .unwrap_or(0),
    }))
}

fn parse_layout_boundary(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
) -> Result<Option<LayoutBoundary>, UiV2AssetError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "ContentDriven" => LayoutBoundary::ContentDriven,
        "ParentDirected" => LayoutBoundary::ParentDirected,
        "Fixed" => LayoutBoundary::Fixed,
        other => {
            return Err(invalid_layout_contract(
                asset_id,
                path,
                format!("unsupported layout boundary {other}"),
            ));
        }
    }))
}

fn parse_input_policy(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
) -> Result<Option<UiInputPolicy>, UiV2AssetError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Inherit" => UiInputPolicy::Inherit,
        "Receive" => UiInputPolicy::Receive,
        "Ignore" => UiInputPolicy::Ignore,
        other => {
            return Err(invalid_layout_contract(
                asset_id,
                path,
                format!("unsupported input policy {other}"),
            ));
        }
    }))
}

fn parse_stretch_mode(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<Option<StretchMode>, UiV2AssetError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Fixed" => StretchMode::Fixed,
        "Stretch" => StretchMode::Stretch,
        other => {
            return Err(invalid_layout_contract(
                asset_id,
                path,
                format!("unsupported {field}.stretch {other}"),
            ));
        }
    }))
}

fn parse_axis(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<Option<UiAxis>, UiV2AssetError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Horizontal" => UiAxis::Horizontal,
        "Vertical" => UiAxis::Vertical,
        other => {
            return Err(invalid_layout_contract(
                asset_id,
                path,
                format!("unsupported {field} {other}"),
            ));
        }
    }))
}

fn parse_scrollbar_visibility(
    asset_id: &str,
    value: Option<&Value>,
    path: &str,
    field: &str,
) -> Result<Option<UiScrollbarVisibility>, UiV2AssetError> {
    let Some(value) = value.and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok(Some(match value {
        "Always" => UiScrollbarVisibility::Always,
        "Never" => UiScrollbarVisibility::Never,
        "Auto" => UiScrollbarVisibility::Auto,
        other => {
            return Err(invalid_layout_contract(
                asset_id,
                path,
                format!("unsupported {field} {other}"),
            ));
        }
    }))
}

fn is_explicit_stretch_axis(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_table)
        .and_then(|table| table.get("stretch"))
        .and_then(Value::as_str)
        == Some("Stretch")
}
