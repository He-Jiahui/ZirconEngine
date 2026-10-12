//! 段与段之间的分隔线出口；位于基础表面上方、选中层下方。

// BUG: [CR-EDITOR-PAINT-FORMS-0005] 这里读取固定 PALETTE.border；主题切换后组表面和文字读取当前动态 palette，分隔线仍保留默认主题颜色。
use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_segmented_control_geometry::segment_divider_rect;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(super) fn push_segment_divider(
    commands: &mut Vec<HostPaintCommand>,
    segment: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let divider = segment_divider_rect(segment);
    if intersect(&divider, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        divider,
        Some(clip.clone()),
        order,
        Some(PALETTE.border),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
