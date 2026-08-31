use crate::core::framework::text::TextLayoutError;
use crate::text::{SharedTextLayoutSession, TextLayoutGeometryOwner};
use zircon_runtime_interface::ui::surface::UiResolvedTextLayout;

use super::super::super::rich_text::UiParsedText;
use super::super::geometry_admission::validate_resolved_layout_geometry;
use super::axes::TableAxes;

pub(super) fn finite_max_zero(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        value
    }
}

pub(super) fn admit_aggregate_layout_geometry(
    layout: &UiResolvedTextLayout,
    axes: TableAxes,
    consumed_block: f32,
    measured_inline: f32,
    source_range: Option<(u32, u32)>,
    provider: &mut SharedTextLayoutSession,
) -> Result<(f32, f32), TextLayoutError> {
    let budget = provider.geometry_budget();
    let work_units = layout.lines.len().saturating_add(layout.boxes.len());
    if let Err(violation) = validate_resolved_layout_geometry(layout, budget) {
        return Err(provider.reject_geometry(
            TextLayoutGeometryOwner::TableAggregate,
            violation,
            source_range,
            work_units,
        ));
    }
    let block_extent = axes.layout_block_extent(layout);
    let inline_extent = axes.layout_inline_extent(layout);
    let next_consumed = budget
        .checked_add_accumulated(consumed_block, block_extent)
        .map_err(|violation| {
            provider.reject_geometry(
                TextLayoutGeometryOwner::TableAggregate,
                violation,
                source_range,
                work_units,
            )
        })?;
    let next_measured = measured_inline.max(inline_extent);
    budget
        .admit_axis_extent(next_measured)
        .map_err(|violation| {
            provider.reject_geometry(
                TextLayoutGeometryOwner::TableAggregate,
                violation,
                source_range,
                work_units,
            )
        })?;
    Ok((next_consumed, next_measured))
}

pub(super) fn whole_parsed_source_range(parsed: &UiParsedText) -> Option<(u32, u32)> {
    let start = u32::try_from(parsed.source_offset()).ok()?;
    let end = parsed
        .source_offset()
        .checked_add(parsed.text().len())
        .and_then(|end| u32::try_from(end).ok())?;
    Some((start, end))
}
