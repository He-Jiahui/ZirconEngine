use super::super::super::super::data::FrameRect;

/// 保留布局阶段的小数帧供最终栅格化；这里提前对齐像素会改变紧邻控件的覆盖范围。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paint_rect(
    rect: &FrameRect,
) -> FrameRect {
    // Material fields share the same final physical-pixel coverage path as Workbench controls.
    // Preserve fractional post-DPI geometry until that rasterizer stage.
    rect.clone()
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
