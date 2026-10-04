use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn root_skeleton_palette_projects_every_surface_from_the_current_theme_roles() {
    let mut palette = PALETTE;
    palette.popup = [1, 2, 3, 255];
    palette.surface = [4, 5, 6, 255];
    palette.surface_inset = [7, 8, 9, 255];
    palette.shell_background = [10, 11, 12, 255];
    palette.surface_hover = [13, 14, 15, 255];
    palette.border = [16, 17, 18, 255];
    palette.accent = [19, 20, 21, 255];
    palette.text_muted = [22, 23, 24, 255];

    assert_eq!(
        root_skeleton_palette(palette),
        RootSkeletonPalette {
            top_bar: [1, 2, 3, 255],
            center_band: [4, 5, 6, 255],
            dock: [7, 8, 9, 255],
            document: [7, 8, 9, 255],
            viewport: [10, 11, 12, 255],
            status: [13, 14, 15, 255],
            separator: [16, 17, 18, 255],
            accent: [19, 20, 21, 255],
            text_muted: [22, 23, 24, 255],
            marker_surface: [7, 8, 9, 255],
        }
    );
}
