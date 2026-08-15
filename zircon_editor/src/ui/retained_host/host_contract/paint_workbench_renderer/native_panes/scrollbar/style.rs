use super::super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WorkbenchScrollbarMetrics {
    pub thickness: f32,
    pub radius: f32,
    pub track_inset: f32,
    pub min_thumb_length: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WorkbenchScrollbarPalette {
    pub track: [u8; 4],
    pub thumb: [u8; 4],
    pub thumb_active: [u8; 4],
}

pub(super) fn workbench_scrollbar_metrics() -> WorkbenchScrollbarMetrics {
    workbench_scrollbar_metrics_from_host(current_host_metrics())
}

pub(super) fn workbench_scrollbar_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchScrollbarMetrics {
    let thickness = metrics.scrollbar_thickness.max(metrics.border_width * 4.0);
    WorkbenchScrollbarMetrics {
        thickness,
        radius: metrics.radius_control.min(thickness * 0.5),
        track_inset: metrics.border_width,
        min_thumb_length: metrics.scrollbar_min_thumb_length.max(thickness * 2.0),
    }
}

pub(super) fn workbench_scrollbar_palette() -> WorkbenchScrollbarPalette {
    let palette = current_host_palette();
    WorkbenchScrollbarPalette {
        track: palette.track,
        thumb: palette.surface_hover,
        thumb_active: palette.surface_selected,
    }
}
