use super::super::super::super::paint_theme::PALETTE;
use super::*;

#[test]
fn inspector_row_palette_projects_from_host_material_roles() {
    let mut host = PALETTE;
    host.surface_inset = [1, 2, 3, 4];
    host.border = [5, 6, 7, 8];
    host.surface_hover = [9, 10, 11, 12];
    host.text_muted = [13, 14, 15, 16];
    host.text = [17, 18, 19, 20];
    host.focus_ring = [21, 22, 23, 24];
    host.accent_soft = [25, 26, 27, 28];
    host.accent = [29, 30, 31, 32];

    let inspector = inspector_row_palette_from_host(host);

    assert_eq!(inspector.field_surface, [1, 2, 3, 4]);
    assert_eq!(inspector.field_border, [5, 6, 7, 8]);
    assert_eq!(inspector.field_hover, [9, 10, 11, 12]);
    assert_eq!(inspector.label, [13, 14, 15, 16]);
    assert_eq!(inspector.value, [17, 18, 19, 20]);
    assert_eq!(inspector.count, [13, 14, 15, 16]);
    assert_eq!(inspector.glyph, [13, 14, 15, 16]);
    assert_eq!(inspector.focus_border, [21, 22, 23, 24]);
    assert_eq!(inspector.checked_surface, [25, 26, 27, 28]);
    assert_eq!(inspector.checked_border, [29, 30, 31, 32]);
}

#[test]
fn inspector_row_defaults_follow_the_projected_host_palette_and_metric() {
    let mut host = PALETTE;
    host.text_muted = [61, 62, 63, 255];
    let palette = inspector_row_palette_from_host(host);
    let node = TemplatePaneNodeData::default();

    assert_eq!(
        resource_label_color_from_palette(&node, palette),
        [61, 62, 63, 255]
    );
    assert_eq!(
        resource_glyph_color_from_palette(&node, palette),
        [61, 62, 63, 255]
    );
    assert_eq!(resource_chevron_size_with_default(&node, 14.0), 14.0);
}
