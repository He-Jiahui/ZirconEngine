use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::layout::{
    Anchor, AxisConstraint, BoxConstraints, LayoutBoundary, Pivot, Position, StretchMode, UiAxis,
    UiContainerKind, UiGridBoxConfig, UiLinearBoxConfig, UiMasonryBoxConfig, UiScrollableBoxConfig,
    UiScrollbarVisibility, UiSizeBoxConfig, UiVirtualListConfig, UiWrapBoxConfig,
};
use zircon_runtime_interface::ui::tree::UiInputPolicy;
use zircon_runtime_interface::ui::v2::UiV2AssetError;

use super::parse::{
    invalid_layout_contract, layout_table, parse_bool, parse_f32, parse_i32, parse_point,
    parse_usize,
};

const MUI_DEFAULT_SPACING_UNIT: f32 = 8.0;

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

pub(super) fn infer_container(
    component: &str,
    attributes: &BTreeMap<String, Value>,
) -> UiContainerKind {
    match component {
        "Container" => UiContainerKind::Container,
        "Block" | "BlockBox" => UiContainerKind::BlockBox,
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
        "Grid" if parse_bool(attributes.get("container")).unwrap_or(false) => {
            UiContainerKind::GridBox(mui_grid_config(attributes))
        }
        "Stack" => mui_stack_container(attributes),
        "Masonry" | "MasonryBox" => UiContainerKind::MasonryBox(mui_masonry_config(attributes)),
        "Canvas" | "CanvasBox" => UiContainerKind::Canvas,
        "SizeBox" => UiContainerKind::SizeBox(Default::default()),
        _ => UiContainerKind::Free,
    }
}

fn mui_grid_config(attributes: &BTreeMap<String, Value>) -> UiGridBoxConfig {
    let spacing = responsive_f32_attribute(attributes, &["spacing"]).unwrap_or(0.0);
    UiGridBoxConfig {
        columns: responsive_usize_attribute(attributes, &["columns"])
            .unwrap_or(12)
            .max(1),
        rows: 1,
        column_gap: responsive_f32_attribute(attributes, &["columnSpacing", "column_spacing"])
            .unwrap_or(spacing)
            .max(0.0),
        row_gap: responsive_f32_attribute(attributes, &["rowSpacing", "row_spacing"])
            .unwrap_or(spacing)
            .max(0.0),
    }
}

fn mui_stack_container(attributes: &BTreeMap<String, Value>) -> UiContainerKind {
    let gap = responsive_f32_attribute(attributes, &["spacing"])
        .unwrap_or(0.0)
        .max(0.0);
    let config = UiLinearBoxConfig { gap };
    match responsive_string_attribute(attributes, &["direction"]).as_deref() {
        Some("row") | Some("row-reverse") => UiContainerKind::HorizontalBox(config),
        _ => UiContainerKind::VerticalBox(config),
    }
}

fn mui_masonry_config(attributes: &BTreeMap<String, Value>) -> UiMasonryBoxConfig {
    UiMasonryBoxConfig {
        columns: responsive_usize_attribute(attributes, &["columns"])
            .unwrap_or(UiMasonryBoxConfig::default().columns)
            .max(1),
        gap: responsive_f32_attribute(attributes, &["spacing"])
            .unwrap_or(0.0)
            .max(0.0),
        sequential: parse_bool(attributes.get("sequential")).unwrap_or(false),
    }
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

fn responsive_f32_attribute(attributes: &BTreeMap<String, Value>, names: &[&str]) -> Option<f32> {
    names
        .iter()
        .find_map(|name| responsive_base_value(attributes.get(*name)))
        .and_then(value_as_f32)
}

fn responsive_string_attribute(
    attributes: &BTreeMap<String, Value>,
    names: &[&str],
) -> Option<String> {
    names
        .iter()
        .find_map(|name| responsive_base_value(attributes.get(*name)))
        .and_then(value_as_string)
}

fn responsive_base_value(value: Option<&Value>) -> Option<&Value> {
    match value? {
        Value::Table(values) => values
            .get("xs")
            .or_else(|| values.get("sm"))
            .or_else(|| values.get("md"))
            .or_else(|| values.get("lg"))
            .or_else(|| values.get("xl")),
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

fn value_as_f32(value: &Value) -> Option<f32> {
    match value {
        Value::Integer(value) => Some(*value as f32 * MUI_DEFAULT_SPACING_UNIT),
        Value::Float(value) if value.is_finite() => Some(*value as f32 * MUI_DEFAULT_SPACING_UNIT),
        Value::String(value) => value.trim().parse().ok(),
        _ => None,
    }
}

fn value_as_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn merged_layout_table<'a>(
    asset_id: &str,
    path: &str,
    self_layout: Option<&'a Value>,
    slot_attributes: &'a BTreeMap<String, Value>,
    parent_container: Option<UiContainerKind>,
) -> Result<Option<V2LayeredLayoutTable<'a>>, UiV2AssetError> {
    let self_layout = self_layout
        .map(|layout| layout_table(asset_id, layout, path, "layout"))
        .transpose()?;
    let slot_layout = slot_attributes
        .get("layout")
        .map(|layout| layout_table(asset_id, layout, path, "slot.layout"))
        .transpose()?;
    if self_layout.is_none() && slot_layout.is_none() {
        return Ok(None);
    }
    Ok(Some(V2LayeredLayoutTable::new(
        self_layout,
        slot_layout,
        parent_container,
    )))
}

#[derive(Clone, Copy)]
struct V2LayeredLayoutTable<'a> {
    self_layout: Option<&'a toml::map::Map<String, Value>>,
    slot_layout: Option<&'a toml::map::Map<String, Value>>,
    restored_axis: Option<&'static str>,
}

impl<'a> V2LayeredLayoutTable<'a> {
    fn new(
        self_layout: Option<&'a toml::map::Map<String, Value>>,
        slot_layout: Option<&'a toml::map::Map<String, Value>>,
        parent_container: Option<UiContainerKind>,
    ) -> Self {
        let restored_axis = match parent_container {
            Some(UiContainerKind::HorizontalBox(_))
            | Some(UiContainerKind::WrapBox(_))
            | Some(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                axis: UiAxis::Horizontal,
                ..
            })) => Some("width"),
            Some(UiContainerKind::VerticalBox(_))
            | Some(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                axis: UiAxis::Vertical,
                ..
            })) => Some("height"),
            _ => None,
        };
        Self {
            self_layout,
            slot_layout,
            restored_axis,
        }
    }

    fn get(&self, key: &str) -> Option<&'a Value> {
        if self.restored_axis == Some(key) {
            if let Some(value) = self.self_layout.and_then(|layout| layout.get(key)) {
                return Some(value);
            }
        }
        self.slot_layout
            .and_then(|layout| layout.get(key))
            .or_else(|| self.self_layout.and_then(|layout| layout.get(key)))
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
        "Canvas" | "CanvasBox" => UiContainerKind::Canvas,
        "Container" => UiContainerKind::Container,
        "Block" | "BlockBox" => UiContainerKind::BlockBox,
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
        "Masonry" | "MasonryBox" => UiContainerKind::MasonryBox(UiMasonryBoxConfig {
            columns: parse_usize(asset_id, table.get("columns"), path, "container.columns")?
                .unwrap_or(UiMasonryBoxConfig::default().columns)
                .max(1),
            gap: parse_f32(table.get("gap"))
                .or_else(|| parse_f32(table.get("spacing")))
                .unwrap_or(0.0),
            sequential: parse_bool(table.get("sequential")).unwrap_or(false),
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

#[cfg(test)]
mod optimization_batch_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_du_v2_layered_layout_preserves_precedence() {
        let mut self_layout = toml::map::Map::new();
        self_layout.insert("width".to_string(), Value::Integer(320));
        self_layout.insert("height".to_string(), Value::Integer(180));
        self_layout.insert("gap".to_string(), Value::Integer(4));
        let mut slot_layout = toml::map::Map::new();
        slot_layout.insert("width".to_string(), Value::Integer(640));
        slot_layout.insert("height".to_string(), Value::Integer(360));
        slot_layout.insert("gap".to_string(), Value::Integer(12));

        let horizontal = V2LayeredLayoutTable::new(
            Some(&self_layout),
            Some(&slot_layout),
            Some(UiContainerKind::HorizontalBox(Default::default())),
        );
        assert_eq!(
            horizontal.get("width").and_then(Value::as_integer),
            Some(320)
        );
        assert_eq!(
            horizontal.get("height").and_then(Value::as_integer),
            Some(360)
        );
        assert_eq!(horizontal.get("gap").and_then(Value::as_integer), Some(12));

        let vertical = V2LayeredLayoutTable::new(
            Some(&self_layout),
            Some(&slot_layout),
            Some(UiContainerKind::VerticalBox(Default::default())),
        );
        assert_eq!(vertical.get("width").and_then(Value::as_integer), Some(640));
        assert_eq!(
            vertical.get("height").and_then(Value::as_integer),
            Some(180)
        );
    }

    #[test]
    fn optimization_batch_du_v2_layout_avoids_table_materialization() {
        let production = include_str!("layout.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("V2 layout production source");
        let merge = production
            .split("fn merged_layout_table")
            .nth(1)
            .expect("V2 layered layout function");

        assert!(merge.contains("V2LayeredLayoutTable::new"));
        assert!(merge.contains("self.slot_layout"));
        assert!(!merge.contains("self_layout.clone()"));
        assert!(!merge.contains("value.clone()"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_du_v2_layered_layout_lookup_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const LOOKUPS_PER_SAMPLE: usize = 4_096;
        const EXTRA_ATTRIBUTES: usize = 96;

        let mut self_layout = toml::map::Map::new();
        let mut slot_layout = toml::map::Map::new();
        for index in 0..EXTRA_ATTRIBUTES {
            self_layout.insert(
                format!("self_attribute_{index:03}"),
                Value::String(format!("self_value_{index:03}")),
            );
            slot_layout.insert(
                format!("slot_attribute_{index:03}"),
                Value::String(format!("slot_value_{index:03}")),
            );
        }
        for (key, self_value, slot_value) in [
            ("width", 320, 640),
            ("height", 180, 360),
            ("gap", 4, 12),
            ("z_index", 2, 8),
            ("clip", 0, 1),
        ] {
            self_layout.insert(key.to_string(), Value::Integer(self_value));
            slot_layout.insert(key.to_string(), Value::Integer(slot_value));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_layout_lookups(
                    &self_layout,
                    &slot_layout,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
                optimized_samples.push(measure_layout_lookups(
                    &self_layout,
                    &slot_layout,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_layout_lookups(
                    &self_layout,
                    &slot_layout,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
                legacy_samples.push(measure_layout_lookups(
                    &self_layout,
                    &slot_layout,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME429_V2_LAYERED_LAYOUT_LOOKUP_BENCH_V1 lookups_per_sample={LOOKUPS_PER_SAMPLE} extra_attributes={EXTRA_ATTRIBUTES} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "V2 layered layout lookup p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_layout_lookups(
            self_layout: &toml::map::Map<String, Value>,
            slot_layout: &toml::map::Map<String, Value>,
            lookup_count: usize,
            optimized: bool,
        ) -> u128 {
            let started_at = Instant::now();
            let mut checksum = 0_i64;
            for _ in 0..lookup_count {
                if optimized {
                    let layout = V2LayeredLayoutTable::new(
                        Some(self_layout),
                        Some(slot_layout),
                        Some(UiContainerKind::HorizontalBox(Default::default())),
                    );
                    checksum = checksum.wrapping_add(layout_checksum(&layout));
                    black_box(layout);
                } else {
                    let mut merged = self_layout.clone();
                    for (key, value) in slot_layout {
                        merged.insert(key.clone(), value.clone());
                    }
                    if let Some(width) = self_layout.get("width") {
                        merged.insert("width".to_string(), width.clone());
                    }
                    checksum = checksum.wrapping_add(
                        ["width", "height", "gap", "z_index", "clip"]
                            .into_iter()
                            .filter_map(|key| merged.get(key).and_then(Value::as_integer))
                            .sum::<i64>(),
                    );
                    black_box(&merged);
                }
            }
            black_box(checksum);
            started_at.elapsed().as_nanos()
        }

        fn layout_checksum(layout: &V2LayeredLayoutTable<'_>) -> i64 {
            ["width", "height", "gap", "z_index", "clip"]
                .into_iter()
                .filter_map(|key| layout.get(key).and_then(Value::as_integer))
                .sum()
        }

        fn p95(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
        }
    }
}
