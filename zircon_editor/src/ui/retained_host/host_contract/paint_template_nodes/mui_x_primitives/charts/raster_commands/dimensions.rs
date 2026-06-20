use crate::ui::retained_host::host_contract::data::FrameRect;

const MUI_X_CHART_MAX_RASTER_EXTENT: f32 = 192.0;

pub(super) fn chart_raster_dimensions(plot: &FrameRect) -> Option<(u32, u32)> {
    if plot.width <= 0.0 || plot.height <= 0.0 {
        return None;
    }
    Some((
        plot.width.ceil().clamp(1.0, MUI_X_CHART_MAX_RASTER_EXTENT) as u32,
        plot.height.ceil().clamp(1.0, MUI_X_CHART_MAX_RASTER_EXTENT) as u32,
    ))
}
