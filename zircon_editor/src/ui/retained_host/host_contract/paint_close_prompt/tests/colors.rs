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
