//! toast 状态标记使用公共提示图标资源入口；value_number 当前承载投影的 status_mark_size。
//! 调用方必须以图标尺寸提供该值，不能复用一般业务数字。

use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchAlertTone as AlertTone;
use super::super::super::template_alert_glyphs::push_alert_mark;
use super::super::layout::{toast_metrics, WorkbenchToastMetrics};
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(super) fn push_toast_status_mark(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    icon: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_alert_mark(
        commands,
        icon,
        clip,
        order,
        // BUG: [CR-EDITOR-PAINT-OVERLAY-0005] toast 标记种类固定为成功，忽略节点声明的严重性/图标；
        // WorkbenchToast 资源声明 severity=info 且 icon=info，这里仍选择成功图形。应由实际投影tone决定资源，而不仅改变颜色。
        super::super::identity::alert_tone(node).unwrap_or(AlertTone::Info),
        color,
        opacity,
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_status_mark_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    toast_status_mark_size_for_metrics(node, toast_metrics())
}

pub(super) fn toast_status_mark_size_for_metrics(
    node: &TemplatePaneNodeData,
    metrics: WorkbenchToastMetrics,
) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        metrics.icon_size
    }
}

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    #[test]
    fn toast_severity_selects_distinct_real_status_assets() {
        let icon = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 24.0,
            height: 24.0,
        };
        let keys = |severity: &str| {
            let mut node = TemplatePaneNodeData::default();
            node.validation_level = severity.into();
            let mut commands = Vec::new();
            push_toast_status_mark(
                &mut commands,
                &node,
                &icon,
                &icon,
                0,
                [127, 127, 127, 255],
                1.0,
            );
            commands
                .iter()
                .find_map(|c| c.image_pixels.as_ref())
                .expect("actual alert asset pixels")
                .resource_key
                .to_string()
        };
        assert_ne!(keys("info"), keys("success"));
        assert_ne!(keys("warning"), keys("error"));
    }
}
