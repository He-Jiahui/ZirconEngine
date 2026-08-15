use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

const GRID_LINE_ALPHA: u8 = 150;
const ZERO_AXIS_ALPHA: u8 = 210;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SampleGridPalette {
    pub outer_surface: [u8; 4],
    pub outer_border: [u8; 4],
    pub plot_surface: [u8; 4],
    pub plot_border: [u8; 4],
    pub grid_line: [u8; 4],
    pub zero_axis: [u8; 4],
    pub tick_text: [u8; 4],
    pub axis_text: [u8; 4],
    pub point: [u8; 4],
    pub selected_point: [u8; 4],
    pub selected_label_surface: [u8; 4],
    pub selected_label_text: [u8; 4],
}

pub(super) fn sample_grid_palette() -> SampleGridPalette {
    sample_grid_palette_from_host(current_host_palette())
}

pub(super) fn sample_grid_palette_from_host(host: HostMaterialPalette) -> SampleGridPalette {
    SampleGridPalette {
        outer_surface: host.surface,
        outer_border: host.border,
        plot_surface: host.surface_inset,
        plot_border: host.separator_strong,
        grid_line: with_alpha(host.separator_soft, GRID_LINE_ALPHA),
        zero_axis: with_alpha(host.separator_strong, ZERO_AXIS_ALPHA),
        tick_text: host.text_muted,
        axis_text: host.text,
        point: host.text_muted,
        selected_point: host.accent,
        selected_label_surface: host.popup,
        selected_label_text: host.text,
    }
}

fn with_alpha(mut color: [u8; 4], alpha: u8) -> [u8; 4] {
    color[3] = alpha;
    color
}
