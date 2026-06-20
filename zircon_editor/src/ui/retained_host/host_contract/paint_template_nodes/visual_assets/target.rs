const MAX_VECTOR_RASTER_EDGE_VALUE: u32 = 4096;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MAX_VECTOR_RASTER_EDGE: u32 =
    MAX_VECTOR_RASTER_EDGE_VALUE;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_ICON_DEFAULT_EDGE: u32 =
    24;

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct RasterTargetSize {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) width: u32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) height: u32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn raster_size_from_frame(
    width: f32,
    height: f32,
) -> Option<(u32, u32)> {
    let target = RasterTargetSize::from_frame(width, height)?;
    Some((target.width, target.height))
}

impl RasterTargetSize {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn new(
        width: u32,
        height: u32,
    ) -> Option<Self> {
        (width > 0 && height > 0).then_some(Self { width, height })
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn from_frame(
        width: f32,
        height: f32,
    ) -> Option<Self> {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return None;
        }
        Self::new(
            width.ceil().clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32,
            height.ceil().clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32,
        )
    }
}
