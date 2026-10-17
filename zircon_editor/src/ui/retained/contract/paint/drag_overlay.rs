//! 拖拽反馈的专用提交入口：只呈现上游投影的 payload、许可与目标，不执行落放或修改拖拽状态。
//! 返回 true 代表接管该节点，即使拖拽关闭或当前无法绘制，也应阻止通用 fallback。

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;

#[path = "drag_overlay/identity.rs"]
mod identity;
#[path = "drag_overlay/layout.rs"]
mod layout;
#[path = "drag_overlay/style.rs"]
mod style;
#[path = "drag_overlay/surface.rs"]
mod surface;
#[path = "drag_overlay/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_drag_overlay_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !identity::is_drag_overlay(node) {
        return false;
    }
    if !node.popup_open && !node.dragging {
        return true;
    }

    let metrics = layout::drag_overlay_metrics();
    let palette = style::drag_overlay_palette();
    let preview_rect = layout::preview_frame(node, rect);
    // BUG: [CR-EDITOR-PAINT-OVERLAY-0003] 预览被裁掉时这里提前结束，后面的落点指示器也被跳过；
    // 游标预览和目标是独立投影的矩形，可出现预览在 clip 外而目标仍可见。应分别判定二者的可见性。
    if preview_rect.width > 0.0
        && preview_rect.height > 0.0
        && intersect(&preview_rect, clip).is_some()
    {
        surface::push_preview_surface(
            commands,
            node,
            &preview_rect,
            clip,
            order,
            opacity,
            palette,
            &metrics,
        );
        surface::push_preview_icon(
            commands,
            node,
            &preview_rect,
            clip,
            order + 1,
            opacity,
            palette,
            &metrics,
        );
        text::push_preview_label(
            commands,
            node,
            &preview_rect,
            clip,
            order + 2,
            opacity,
            palette,
            &metrics,
        );
    }

    if let Some(indicator) = layout::indicator_frame(node, &metrics) {
        surface::push_drop_indicator(commands, node, indicator, clip, order + 3, opacity, palette);
    }

    true
}

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    #[test]
    fn visible_drop_target_survives_a_culled_cursor_preview() {
        let mut node = TemplatePaneNodeData::default();
        node.role = "DragOverlay".into();
        node.dragging = true;
        node.has_drag_cursor = true;
        node.drag_cursor_x = 500.0;
        node.drag_cursor_y = 500.0;
        node.drag_preview_width = 60.0;
        node.drag_preview_height = 20.0;
        node.has_drop_target = true;
        node.drop_target_x = 10.0;
        node.drop_target_y = 12.0;
        node.drop_target_width = 40.0;
        node.drop_target_height = 24.0;
        node.drop_indicator_edge = "inside".into();
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let mut commands = Vec::new();
        assert!(push_drag_overlay_commands(
            &mut commands,
            &node,
            &clip,
            &clip,
            0,
            1.0
        ));
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].frame.x, 10.0);
        assert_eq!(commands[0].frame.y, 12.0);
        assert_eq!(commands[0].frame.width, 40.0);
        node.has_drop_target = false;
        commands.clear();
        assert!(push_drag_overlay_commands(
            &mut commands,
            &node,
            &clip,
            &clip,
            0,
            1.0
        ));
        assert!(commands.is_empty());
    }
}
