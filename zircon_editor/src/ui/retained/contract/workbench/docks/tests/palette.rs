use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn dock_chrome_palette_projects_all_surface_roles_from_the_current_theme() {
    let mut palette = PALETTE;
    palette.surface_inset = [1, 2, 3, 255];
    palette.popup = [4, 5, 6, 255];
    palette.border = [7, 8, 9, 255];
    palette.accent = [10, 11, 12, 255];

    assert_eq!(
        dock_chrome_palette(palette),
        DockChromePalette {
            shell: [1, 2, 3, 255],
            document: [1, 2, 3, 255],
            header: [4, 5, 6, 255],
            separator: [7, 8, 9, 255],
            accent: [10, 11, 12, 255],
        }
    );
}
