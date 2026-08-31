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

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use toml::Value;

    use super::parse_canvas_placement;

    const PLACEMENT_KEYS: [&str; 6] = [
        "anchor",
        "anchor_max",
        "pivot",
        "position",
        "offset",
        "auto_size",
    ];

    #[test]
    fn optimization_batch_dm_canvas_placement_presence_semantics_are_preserved() {
        let empty = toml::map::Map::new();
        assert!(parse_canvas_placement(&empty, "root").unwrap().is_none());

        let mut auto_size_only = toml::map::Map::new();
        auto_size_only.insert("auto_size".to_string(), Value::Boolean(true));
        assert!(parse_canvas_placement(&auto_size_only, "root")
            .unwrap()
            .is_some());
    }

    #[test]
    fn optimization_batch_dm_both_canvas_parsers_lookup_each_placement_field_once() {
        let template_source = include_str!("slot_contract.rs");
        let v2_source = include_str!("../../v2/surface_tree/slot.rs");

        for source in [template_source, v2_source] {
            let body = source
                .split("fn parse_canvas_placement")
                .nth(1)
                .expect("canvas placement parser")
                .split("fn parse_grid_placement")
                .next()
                .expect("canvas placement body");
            assert!(!body.contains("layout.contains_key"));
            for key in PLACEMENT_KEYS {
                let lookup = format!("layout.get(\"{key}\")");
                assert_eq!(
                    body.matches(lookup.as_str()).count(),
                    1,
                    "placement field {key} should be looked up once"
                );
            }
        }
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dm_cached_canvas_placement_fields_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const PARSES_PER_SAMPLE: usize = 131_072;

        let mut layout = (0..64_u64)
            .map(|index| (format!("filler_{index:02}"), index))
            .collect::<BTreeMap<_, _>>();
        layout.insert("auto_size".to_string(), 1);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_canvas_field_lookups(
                    &layout,
                    PARSES_PER_SAMPLE,
                    true,
                ));
                optimized_samples.push(measure_canvas_field_lookups(
                    &layout,
                    PARSES_PER_SAMPLE,
                    false,
                ));
            } else {
                optimized_samples.push(measure_canvas_field_lookups(
                    &layout,
                    PARSES_PER_SAMPLE,
                    false,
                ));
                legacy_samples.push(measure_canvas_field_lookups(
                    &layout,
                    PARSES_PER_SAMPLE,
                    true,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME421_CACHED_CANVAS_PLACEMENT_FIELDS_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "cached canvas placement fields p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_canvas_field_lookups(
        layout: &BTreeMap<String, u64>,
        parse_count: usize,
        legacy: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..parse_count {
            let values = if legacy {
                let has_placement = PLACEMENT_KEYS
                    .iter()
                    .any(|key| black_box(layout.contains_key(*key)));
                has_placement.then(|| PLACEMENT_KEYS.map(|key| layout.get(key).copied()))
            } else {
                let values = PLACEMENT_KEYS.map(|key| layout.get(key).copied());
                values.iter().any(Option::is_some).then_some(values)
            };
            checksum = checksum.wrapping_add(
                values
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|value| black_box(value))
                    .sum::<u64>(),
            );
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
