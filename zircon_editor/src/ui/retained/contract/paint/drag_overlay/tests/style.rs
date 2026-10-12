use super::super::super::super::paint_theme::PALETTE;
use super::*;

#[test]
fn drag_overlay_palette_projects_from_host_material_roles() {
    let mut host = PALETTE;
    host.accent_soft = [1, 2, 3, 4];
    host.error_container = [5, 6, 7, 8];
    host.accent = [9, 10, 11, 12];
    host.error = [13, 14, 15, 16];
    host.text = [17, 18, 19, 20];

    let overlay = drag_overlay_palette_from_host(host);

    assert_eq!(overlay.preview_surface, [1, 2, 3, 4]);
    assert_eq!(overlay.preview_surface_blocked, [5, 6, 7, 8]);
    assert_eq!(overlay.preview_border, [9, 10, 11, 12]);
    assert_eq!(overlay.preview_border_blocked, [13, 14, 15, 16]);
    assert_eq!(overlay.preview_text, [17, 18, 19, 20]);
}
