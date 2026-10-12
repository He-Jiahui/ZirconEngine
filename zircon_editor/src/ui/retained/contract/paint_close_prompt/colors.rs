use super::super::paint_theme::HostMaterialPalette;

const OVERLAY_OPACITY: u8 = 168;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) struct ClosePromptPalette {
    pub(in crate::ui::retained_host::host_contract) overlay: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) dialog: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) dialog_inset: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) button: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) button_disabled: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) text: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) text_muted: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) text_disabled: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) warning: [u8; 4],
    pub(in crate::ui::retained_host::host_contract) accent: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract) fn close_prompt_palette(
    palette: HostMaterialPalette,
) -> ClosePromptPalette {
    ClosePromptPalette {
        overlay: [
            palette.shell_background[0],
            palette.shell_background[1],
            palette.shell_background[2],
            OVERLAY_OPACITY,
        ],
        dialog: palette.surface,
        dialog_inset: palette.surface_inset,
        button: palette.surface_hover,
        button_disabled: palette.surface_disabled,
        text: palette.text,
        text_muted: palette.text_muted,
        text_disabled: palette.text_disabled,
        warning: palette.warning,
        accent: palette.focus_ring,
    }
}

#[cfg(test)]
#[path = "tests/colors.rs"]
mod tests;
