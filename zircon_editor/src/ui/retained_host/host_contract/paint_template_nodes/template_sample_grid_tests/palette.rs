use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::palette::sample_grid_palette_from_host;

#[test]
fn sample_grid_palette_projects_every_visual_role_from_the_host_theme() {
    let mut host = PALETTE;
    host.surface = [1, 2, 3, 4];
    host.border = [5, 6, 7, 8];
    host.surface_inset = [9, 10, 11, 12];
    host.separator_strong = [13, 14, 15, 16];
    host.separator_soft = [17, 18, 19, 20];
    host.text_muted = [21, 22, 23, 24];
    host.text = [25, 26, 27, 28];
    host.accent = [33, 34, 35, 36];
    host.accent_soft = [37, 38, 39, 40];
    host.surface_selected = [41, 42, 43, 44];
    host.popup = [45, 46, 47, 48];

    let palette = sample_grid_palette_from_host(host);

    assert_eq!(palette.outer_surface, [1, 2, 3, 4]);
    assert_eq!(palette.outer_border, [5, 6, 7, 8]);
    assert_eq!(palette.plot_surface, [9, 10, 11, 12]);
    assert_eq!(palette.plot_border, [13, 14, 15, 16]);
    assert_eq!(palette.grid_line, [17, 18, 19, 150]);
    assert_eq!(palette.zero_axis, [13, 14, 15, 210]);
    assert_eq!(palette.tick_text, [21, 22, 23, 24]);
    assert_eq!(palette.axis_text, [25, 26, 27, 28]);
    assert_eq!(palette.point, [21, 22, 23, 24]);
    assert_eq!(palette.selected_point, [33, 34, 35, 36]);
    assert_eq!(palette.selected_label_surface, [45, 46, 47, 48]);
    assert_eq!(palette.selected_label_text, [25, 26, 27, 28]);
}
