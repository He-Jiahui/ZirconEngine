//! 普通滑块尾部值框按宽度决定是否显示，并使用宿主密度规定的最小文字高度。

// BUG: [CR-EDITOR-PAINT-FORMS-0007] 高度未达值框最小高度时仍返回完整值框；节点 clip 默认是窗格 clip，值框可越出滑块自身纵向范围。
use super::super::super::super::data::FrameRect;
use super::super::metrics::workbench_slider_metrics;

/// 仅以横向宽度判断是否存在；当前纵向最小高度可能大于输入框，消费端没有二次完整容纳检查。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_value_rect(
    rect: &FrameRect,
) -> Option<FrameRect> {
    let metrics = workbench_slider_metrics();
    if rect.width < metrics.value_min_width {
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
