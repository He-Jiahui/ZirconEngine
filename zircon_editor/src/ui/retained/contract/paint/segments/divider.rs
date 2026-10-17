//! 段与段之间的分隔线出口；位于基础表面上方、选中层下方。

// BUG: [CR-EDITOR-PAINT-FORMS-0005] 这里读取固定 PALETTE.border；主题切换后组表面和文字读取当前动态 palette，分隔线仍保留默认主题颜色。
use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::current_host_palette;
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
        Some(current_host_palette().border),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_from_tokens_for_test,
    };
    use zircon_runtime_interface::ui::{design_tokens::EditorDesignTokens, style::UiRgbaColor};
    #[test]
    fn segment_divider_records_the_active_border_color() {
        let mut tokens = EditorDesignTokens::workbench_dark();
        tokens.palette.border = UiRgbaColor::from_u8(91, 83, 73, 255);
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_from_tokens_for_test(&tokens));
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 32.0,
        };
        let clip = FrameRect {
            x: -10.0,
            y: -10.0,
            width: 120.0,
            height: 60.0,
        };
        let mut commands = Vec::new();
        push_segment_divider(&mut commands, &rect, &clip, 0, 1.0);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].background_color, Some([91, 83, 73, 255]));
    }
}
