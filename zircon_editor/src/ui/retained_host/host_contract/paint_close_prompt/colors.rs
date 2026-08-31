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
mod tests {
    use super::close_prompt_palette;
    use crate::ui::retained_host::host_contract::paint_theme::{HostMaterialPalette, PALETTE};

    #[test]
    fn close_prompt_palette_projects_runtime_host_roles() {
        let mut host: HostMaterialPalette = PALETTE;
        host.shell_background = [1, 2, 3, 255];
        host.surface = [4, 5, 6, 255];
        host.surface_inset = [7, 8, 9, 255];
        host.surface_hover = [10, 11, 12, 255];
        host.surface_disabled = [13, 14, 15, 255];
        host.text = [16, 17, 18, 255];
        host.text_muted = [19, 20, 21, 255];
        host.text_disabled = [22, 23, 24, 255];
        host.warning = [25, 26, 27, 255];
        host.focus_ring = [28, 29, 30, 255];

        let palette = close_prompt_palette(host);

        assert_eq!(palette.overlay, [1, 2, 3, 168]);
        assert_eq!(palette.dialog, [4, 5, 6, 255]);
        assert_eq!(palette.dialog_inset, [7, 8, 9, 255]);
        assert_eq!(palette.button, [10, 11, 12, 255]);
        assert_eq!(palette.button_disabled, [13, 14, 15, 255]);
        assert_eq!(palette.text, [16, 17, 18, 255]);
        assert_eq!(palette.text_muted, [19, 20, 21, 255]);
        assert_eq!(palette.text_disabled, [22, 23, 24, 255]);
        assert_eq!(palette.warning, [25, 26, 27, 255]);
        assert_eq!(palette.accent, [28, 29, 30, 255]);
    }
}
