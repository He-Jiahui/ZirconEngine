use super::model::WorkbenchTooltipStyle;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTooltipPalette
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub title: [u8; 4],
    pub body: [u8; 4],
    pub icon: [u8; 4],
    pub shadow: [u8; 4],
    pub disabled_surface: [u8; 4],
    pub disabled_border: [u8; 4],
    pub disabled_text: [u8; 4],
    pub disabled_shadow: [u8; 4],
    pub focused_border: [u8; 4],
    pub hover_icon: [u8; 4],
}

#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_SURFACE: [u8; 4] =
    PALETTE.popup;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_BORDER: [u8; 4] =
    PALETTE.border;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_normal_style(
    state: UiPainterResolvedState,
) -> WorkbenchTooltipStyle {
    tooltip_normal_style_from_palette(state, tooltip_palette())
}

pub(super) fn tooltip_normal_style_from_palette(
    state: UiPainterResolvedState,
    palette: WorkbenchTooltipPalette,
) -> WorkbenchTooltipStyle {
    WorkbenchTooltipStyle {
        surface: palette.surface,
        border: palette.border,
        title: palette.title,
        body: palette.body,
        arrow: palette.surface,
        icon: palette.icon,
        shadow: palette.shadow,
        state,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_palette(
) -> WorkbenchTooltipPalette {
    tooltip_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchTooltipPalette {
    WorkbenchTooltipPalette {
        surface: palette.popup,
        border: palette.border,
        title: palette.text,
        body: palette.text_muted,
        icon: palette.accent,
        shadow: palette.shadow,
        disabled_surface: palette.surface_disabled,
        disabled_border: palette.border_disabled,
        disabled_text: palette.text_disabled,
        disabled_shadow: [
            palette.shadow[0],
            palette.shadow[1],
            palette.shadow[2],
            palette.shadow[3].min(48),
        ],
        focused_border: palette.focus_ring,
        hover_icon: palette.accent,
    }
}
