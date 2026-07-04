use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MaterialFeedbackPalette {
    pub track: [u8; 4],
    pub workbench_track: [u8; 4],
    pub workbench_fill: [u8; 4],
    pub disabled_track: [u8; 4],
    pub disabled_fill: [u8; 4],
    pub accent: [u8; 4],
    pub warning: [u8; 4],
    pub error: [u8; 4],
    pub success: [u8; 4],
    pub info: [u8; 4],
    pub backdrop_scrim: [u8; 4],
}

pub(super) fn material_feedback_palette() -> MaterialFeedbackPalette {
    material_feedback_palette_from_host(current_host_palette())
}

pub(super) fn material_feedback_palette_from_host(
    palette: HostMaterialPalette,
) -> MaterialFeedbackPalette {
    MaterialFeedbackPalette {
        track: palette.track,
        workbench_track: palette.track,
        workbench_fill: palette.separator_strong,
        disabled_track: palette.surface_disabled,
        disabled_fill: palette.text_disabled,
        accent: palette.accent,
        warning: palette.warning,
        error: palette.error,
        success: palette.success,
        info: palette.info,
        backdrop_scrim: palette.shadow,
    }
}
