//! 在测量与成品发布边界检查已计算出的文本几何。
//!
//! 校验借用现有布局并在首个异常返回；发布入口仅在失败时把拒绝证据记入文本会话。
//! 检查覆盖本模块明确列出的几何字段，不替代来源映射、容器裁剪或内存预算。

use crate::core::framework::text::TextLayoutError;
use crate::text::{
    SharedTextLayoutSession, TextLayoutGeometryBudget, TextLayoutGeometryOwner,
    TextLayoutGeometryViolation,
};
use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{UiResolvedTextBox, UiResolvedTextLayout},
};

/// 按轴预算检查尺寸的宽和高，按顺序返回首个几何错误。
///
/// `size` 与预算均按值读取；成功返回 `Ok(())`，不修正尺寸或扣减预算。
pub(super) fn validate_resolved_size_geometry(
    size: UiSize,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    budget.admit_axis_extent(size.width)?;
    budget.admit_axis_extent(size.height)?;
    Ok(())
}

/// 只读检查已解析布局中用于测量、逐行放置及文字盒绘制的几何。
///
/// # 参数与返回
/// `layout` 的借用只持续本次检查；预算按值读取。检查总体宽高、字号行高、
/// 行的两种 frame、行宽/基线/逐字形 advance，以及文字盒 frame 和边框宽度。
/// 全部合法返回 `Ok(())`，否则按遍历顺序返回首个错误，不修改或裁剪原布局。
///
/// # 约束
/// 本函数不验证源范围、字形来源或行盒之间的语义对应关系。
pub(super) fn validate_resolved_layout_geometry(
    layout: &UiResolvedTextLayout,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    budget.admit_axis_extent(layout.measured_width)?;
    budget.admit_axis_extent(layout.measured_height)?;
    budget.admit_axis_extent(layout.font_size)?;
    budget.admit_axis_extent(layout.line_height)?;
    for line in &layout.lines {
        validate_frame(line.frame, budget)?;
        validate_frame(line.placement_frame, budget)?;
        budget.admit_axis_extent(line.measured_width)?;
        budget.admit_coordinate(line.baseline)?;
        for advance in &line.glyph_advances {
            budget.admit_axis_extent(*advance)?;
        }
    }
    validate_resolved_text_boxes_geometry(&layout.boxes, budget)
}

/// 在构建发布产物前检查成品布局，并把失败归因到本次文本会话。
///
/// # 参数与返回
/// `layout` 仅被借用检查；`provider` 提供预算，并在失败时更新几何拒绝报告。
/// 成功返回 `Ok(())`；失败经会话返回 `GeometryTooLarge`，不在此构建或修改产物。
///
/// # 诊断约束
/// 源范围两端均可转为 u32 时才保留范围；工作量使用行数与盒数的饱和和，
/// 不是字符数或完整构建成本。实际发布调用者负责在此成功之后继续构建产物。
///
/// # 参考
/// 调用方为父模块的 `layout_parsed_text_with_provider_and_viewport_outcome`；
/// 拒绝上报由 `SharedTextLayoutSession::reject_geometry` 承接。
pub(super) fn admit_resolved_layout_publication(
    layout: &UiResolvedTextLayout,
    provider: &mut SharedTextLayoutSession,
) -> Result<(), TextLayoutError> {
    validate_resolved_layout_geometry(layout, provider.geometry_budget()).map_err(|violation| {
        let source_range = u32::try_from(layout.source_range.start)
            .ok()
            .zip(u32::try_from(layout.source_range.end).ok());
        let work_units = layout.lines.len().saturating_add(layout.boxes.len());
        provider.reject_geometry(
            TextLayoutGeometryOwner::ResolvedLayoutPublication,
            violation,
            source_range,
            work_units,
        )
    })
}

/// 顺序检查文字盒的 frame 与边框宽度，返回首个几何错误。
///
/// `boxes` 仅借用至检查结束；成功不改动盒列表，也不验证盒与源内容的对应关系。
pub(super) fn validate_resolved_text_boxes_geometry(
    boxes: &[UiResolvedTextBox],
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    for text_box in boxes {
        validate_frame(text_box.frame, budget)?;
        budget.admit_axis_extent(text_box.border_width)?;
    }
    Ok(())
}

fn validate_frame(
    frame: UiFrame,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    budget.admit_coordinate(frame.x)?;
    budget.admit_coordinate(frame.y)?;
    budget.admit_axis_extent(frame.width)?;
    budget.admit_axis_extent(frame.height)?;
    budget.admit_coordinate(frame.right())?;
    budget.admit_coordinate(frame.bottom())?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/geometry_admission.rs"]
mod tests;
