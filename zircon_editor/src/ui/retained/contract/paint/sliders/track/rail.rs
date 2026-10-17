//! 轨道底色和从范围下限到当前值的填充共享归一化区间。

// BUG: [CR-EDITOR-PAINT-FORMS-0008] 右端极窄填充被强制放大至 1 像素，x 仍按原始区间起点计算，结果可能超过轨道右边界。
use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchSliderStyle;
use super::super::super::template_slider_geometry::{slider_fill_span, workbench_slider_metrics};
use super::super::layers::track_fill_order;

/// 调用方传入已归一化主值和可选下限；底轨总绘制，正宽区间才追加填充。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_track(
    commands: &mut Vec<HostPaintCommand>,
    style: &WorkbenchSliderStyle,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    range_min_percent: Option<f32>,
    opacity: f32,
) {
    let metrics = workbench_slider_metrics();
    commands.push(HostPaintCommand::quad(
        track_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.track),
        None,
        0.0,
        metrics.track_radius,
        opacity,
    ));

    let (fill_start, fill_end) = slider_fill_span(percent, range_min_percent);
    let fill_width = (track_rect.width * (fill_end - fill_start)).max(0.0);
    if fill_width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: track_rect.x + track_rect.width * fill_start,
            y: track_rect.y,
            width: fill_width,
            height: track_rect.height,
        },
        Some(clip.clone()),
        track_fill_order(order),
        Some(style.fill),
        None,
        0.0,
        metrics.track_radius,
        opacity,
    ));
}

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
    use crate::ui::retained_host::host_contract::paint_template_nodes::style_selector::select_workbench_slider_style;
    #[test]
    fn subpixel_fill_near_the_right_edge_stays_inside_the_rail() {
        let style = select_workbench_slider_style(&TemplatePaneNodeData::default());
        let rail = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 4.0,
        };
        let mut commands = Vec::new();
        push_slider_track(
            &mut commands,
            &style,
            &rail,
            &rail,
            0,
            1.0,
            Some(0.999),
            1.0,
        );
        assert_eq!(commands.len(), 2);
        let fill = &commands[1].frame;
        assert!(fill.width > 0.0 && fill.width < 1.0);
        assert!(fill.x + fill.width <= rail.x + rail.width);
        commands.clear();
        push_slider_track(&mut commands, &style, &rail, &rail, 0, 1.0, Some(1.0), 1.0);
        assert_eq!(commands.len(), 1, "an empty interval has no synthetic fill");
    }
}
