use crate::ui::retained_host::host_contract::data::FrameRect;

const MUI_X_CHART_MAX_RASTER_EXTENT: f32 = 192.0;

/// 小图表位图分配前先跳过非正尺寸绘图区，并把每轴像素上限限制在 192；宿主图像命令仍使用原绘图区显示框。
pub(super) fn chart_raster_dimensions(plot: &FrameRect) -> Option<(u32, u32)> {
    if plot.width <= 0.0 || plot.height <= 0.0 {
        return None;
    }
    Some((
        plot.width.ceil().clamp(1.0, MUI_X_CHART_MAX_RASTER_EXTENT) as u32,
        plot.height.ceil().clamp(1.0, MUI_X_CHART_MAX_RASTER_EXTENT) as u32,
    ))
}
