//! 对表格分块结果执行几何预算准入，并为拒绝补充会话诊断归属。
//! 几何准入不验证源文本内容、裁剪关系或内存预算。

use crate::core::framework::text::TextLayoutError;
use crate::text::{SharedTextLayoutSession, TextLayoutGeometryOwner};
use zircon_runtime_interface::ui::surface::UiResolvedTextLayout;

use super::super::super::rich_text::UiParsedText;
use super::super::geometry_admission::validate_resolved_layout_geometry;
use super::axes::TableAxes;

/// 只对有限值施加零下限；非有限值原样交回，让后续预算步骤识别并拒绝。
pub(super) fn finite_max_zero(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        value
    }
}

/// 在接受下一分块前校验布局并计算新的块累计量和最大行内长度。
///
/// # 参数与返回
/// `consumed_block`、`measured_inline` 是此前聚合状态；`source_range` 仅用于拒绝诊断归因。
/// 成功返回替代这两个状态的二元组，不修改输入布局；调用方只在成功后更新聚合变量。
///
/// # 错误与借用
/// 借用同一会话的预算；布局、累计值或行内值被拒绝时记录表格聚合诊断并返回几何过大错误。
/// 诊断会修改会话状态，因此失败不等于会话无副作用。
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

/// 将当前解析视图的绝对字节起止转换为诊断范围。
/// 偏移加长度溢出或任一端无法表示为目标整数时返回空值，不修改解析视图或会话。
pub(super) fn whole_parsed_source_range(parsed: &UiParsedText) -> Option<(u32, u32)> {
    let start = u32::try_from(parsed.source_offset()).ok()?;
    let end = parsed
        .source_offset()
        .checked_add(parsed.text().len())
        .and_then(|end| u32::try_from(end).ok())?;
    Some((start, end))
}
