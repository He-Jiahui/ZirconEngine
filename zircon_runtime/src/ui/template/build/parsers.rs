use toml::Value;

use zircon_runtime_interface::ui::layout::{
    AxisConstraint, LayoutBoundary, StretchMode, UiAxis, UiContainerKind, UiGridBoxConfig,
    UiLinearBoxConfig, UiMasonryBoxConfig, UiScrollableBoxConfig, UiScrollbarVisibility,
    UiSizeBoxConfig, UiVirtualListConfig, UiWrapBoxConfig,
};
use zircon_runtime_interface::ui::tree::UiInputPolicy;

use crate::ui::layout::MAX_UI_LAYOUT_DISCRETE_VALUE;

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
            columns: parse_usize(table.get("columns"), node_path, "container.columns")?
                .unwrap_or(1),
            rows: parse_usize(table.get("rows"), node_path, "container.rows")?.unwrap_or(1),
            column_gap: parse_f32(table.get("column_gap"))
                .or_else(|| parse_f32(table.get("gap")))
                .unwrap_or(0.0),
            row_gap: parse_f32(table.get("row_gap"))
                .or_else(|| parse_f32(table.get("gap")))
                .unwrap_or(0.0),
        }),
        "Masonry" | "MasonryBox" => UiContainerKind::MasonryBox(UiMasonryBoxConfig {
            columns: parse_usize(table.get("columns"), node_path, "container.columns")?
                .unwrap_or(UiMasonryBoxConfig::default().columns)
                .max(1),
            gap: parse_f32(table.get("gap"))
                .or_else(|| parse_f32(table.get("spacing")))
                .unwrap_or(0.0),
            sequential: parse_bool(table.get("sequential")).unwrap_or(false),
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

pub(super) fn parse_usize(
    value: Option<&Value>,
    node_path: &str,
    field: &str,
) -> Result<Option<usize>, UiTemplateBuildError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let parsed = value
        .as_integer()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: format!("{field} must be a non-negative integer"),
        })?;
    if parsed > MAX_UI_LAYOUT_DISCRETE_VALUE {
        return Err(UiTemplateBuildError::InvalidLayoutContract {
            node_path: node_path.to_string(),
            detail: format!("{field} must not exceed {MAX_UI_LAYOUT_DISCRETE_VALUE}"),
        });
    }
    Ok(Some(parsed))
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn explicit_layout_usize_values_are_bounded_before_runtime_allocation() {
        let maximum = Value::Integer(MAX_UI_LAYOUT_DISCRETE_VALUE as i64);
        assert_eq!(
            parse_usize(Some(&maximum), "root", "container.columns").unwrap(),
            Some(MAX_UI_LAYOUT_DISCRETE_VALUE)
        );

        let oversized = Value::Integer((MAX_UI_LAYOUT_DISCRETE_VALUE + 1) as i64);
        let error = parse_usize(Some(&oversized), "root", "container.columns").unwrap_err();
        assert!(error.to_string().contains(&format!(
            "container.columns must not exceed {MAX_UI_LAYOUT_DISCRETE_VALUE}"
        )));
    }

    #[test]
    #[ignore = "release-only bounded layout admission benchmark"]
    fn bounded_layout_admission_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 21;
        const OPERATIONS_PER_SAMPLE: usize = 64;
        const AUTHORED_TRACK_COUNT: usize = 65_536;

        fn legacy_parse(value: &Value) -> usize {
            value
                .as_integer()
                .and_then(|value| usize::try_from(value).ok())
                .expect("non-negative legacy layout integer")
        }

        fn measure_legacy(value: &Value) -> u128 {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                let track_count = legacy_parse(black_box(value));
                black_box(vec![0.0_f32; track_count]);
            }
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(value: &Value) -> u128 {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(parse_usize(
                    Some(black_box(value)),
                    "benchmark-root",
                    "container.columns",
                ));
            }
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let oversized = Value::Integer(AUTHORED_TRACK_COUNT as i64);
        assert!(parse_usize(Some(&oversized), "benchmark-root", "container.columns").is_err());

        for _ in 0..4 {
            black_box(measure_legacy(&oversized));
            black_box(measure_optimized(&oversized));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&oversized));
                optimized_samples.push(measure_optimized(&oversized));
            } else {
                optimized_samples.push(measure_optimized(&oversized));
                legacy_samples.push(measure_legacy(&oversized));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME76_BOUNDED_LAYOUT_ADMISSION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
operations_per_sample={OPERATIONS_PER_SAMPLE} authored_track_count={AUTHORED_TRACK_COUNT} \
max_discrete_value={MAX_UI_LAYOUT_DISCRETE_VALUE} pair_order=alternating_legacy_even \
legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_track_allocations_per_sample={OPERATIONS_PER_SAMPLE} \
optimized_track_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
            "bounded admission must reduce malicious-track P95 by at least 75%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
