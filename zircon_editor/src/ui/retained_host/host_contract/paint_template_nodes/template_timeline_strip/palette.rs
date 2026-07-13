use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TimelineStripPalette {
    pub outer_surface: [u8; 4],
    pub outer_border: [u8; 4],
    pub plot_surface: [u8; 4],
    pub grid_line: [u8; 4],
    pub track_surface: [u8; 4],
    pub track_progress: [u8; 4],
    pub playhead: [u8; 4],
    pub key: [u8; 4],
    pub key_center: [u8; 4],
    pub track_text: [u8; 4],
    pub tick_text: [u8; 4],
    pub footer_surface: [u8; 4],
}

pub(super) fn timeline_palette() -> TimelineStripPalette {
    timeline_palette_from_host(current_host_palette())
}

pub(super) fn timeline_palette_from_host(host: HostMaterialPalette) -> TimelineStripPalette {
    TimelineStripPalette {
        outer_surface: host.surface_inset,
        outer_border: host.border,
        plot_surface: host.shell_background,
        grid_line: host.separator_soft,
        track_surface: host.track,
        track_progress: host.accent_soft,
        playhead: host.accent,
        key: host.accent,
        key_center: host.text,
        track_text: host.text,
        tick_text: host.text_muted,
        footer_surface: host.surface,
    }
}
