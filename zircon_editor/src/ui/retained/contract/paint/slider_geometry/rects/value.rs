//! 普通滑块尾部值框按宽度决定是否显示，并使用宿主密度规定的最小文字高度。

// BUG: [CR-EDITOR-PAINT-FORMS-0007] 高度未达值框最小高度时仍返回完整值框；节点 clip 默认是窗格 clip，值框可越出滑块自身纵向范围。
use super::super::super::super::data::FrameRect;
use super::super::metrics::workbench_slider_metrics;

/// 仅在横向车道和当前密度最小值框高度都可容纳时提供可选值框。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_value_rect(
    rect: &FrameRect,
) -> Option<FrameRect> {
    let metrics = workbench_slider_metrics();
    if !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width < metrics.value_min_width
        || rect.height < metrics.value_min_height
    {
        return None;
    }
    let height = (rect.height - metrics.value_height_pad)
        .clamp(metrics.value_min_height, metrics.value_max_height);
    Some(FrameRect {
        x: rect.x + rect.width - metrics.horizontal_inset - metrics.value_width,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width: metrics.value_width,
        height,
    })
}

#[cfg(test)]
mod finite_p2_regression_tests {
    use super::*;
    #[test]
    fn slider_value_requires_room_for_the_current_minimum_height() {
        let metrics = workbench_slider_metrics();
        let mut parent = FrameRect {
            x: 5.0,
            y: 7.0,
            width: metrics.value_min_width,
            height: metrics.value_min_height - 0.5,
        };
        assert!(slider_value_rect(&parent).is_none());
        parent.height = metrics.value_min_height;
        let value = slider_value_rect(&parent).expect("exact minimum fits");
        assert!(value.y >= parent.y);
        assert!(value.y + value.height <= parent.y + parent.height);
        parent.height = f32::NAN;
        assert!(slider_value_rect(&parent).is_none());
    }
}
