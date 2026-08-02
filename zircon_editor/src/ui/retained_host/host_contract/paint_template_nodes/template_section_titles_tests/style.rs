use crate::ui::retained_host::primitives::Color;

use super::super::super::super::paint_theme::{METRICS, current_host_palette};
use super::super::super::template_section_title_glyphs::{
    SectionTitleIcon, section_icon_color, section_title_glyph_metrics_from_host,
    section_title_glyph_palette_from_host,
};
use super::super::style::{
    section_text_color, section_title_metrics_from_host, section_title_palette_from_host,
};
use super::support::title_node;

#[test]
fn mesh_renderer_section_title_uses_audited_title_tone() {
    let palette = section_title_palette_from_host(current_host_palette());

    assert_eq!(
        section_text_color(&title_node("WorkbenchMeshLabel", "Mesh Renderer")),
        palette.mesh_text
    );
}

#[test]
fn section_title_uses_declared_title_tone() {
    let mut node = title_node("WorkbenchSelectionTitle", "Checkboxes & Radios");
    node.label_color = Color::from_rgb_u8(152, 163, 168);

    assert_eq!(section_text_color(&node), [152, 163, 168, 255]);
}

#[test]
fn transform_section_title_uses_projected_icon_tone() {
    let palette = section_title_glyph_palette_from_host(current_host_palette());

    assert_eq!(
        section_icon_color(SectionTitleIcon::Transform),
        palette.transform_icon
    );
    assert_eq!(section_icon_color(SectionTitleIcon::Cube), palette.icon);
}

#[test]
fn section_title_metrics_project_from_host_metrics() {
    let title_metrics = section_title_metrics_from_host(METRICS);
    let glyph_metrics = section_title_glyph_metrics_from_host(METRICS);

    assert_eq!(title_metrics.font_size, METRICS.font_body);
    assert_eq!(
        title_metrics.line_height,
        METRICS.line_height(METRICS.font_body)
    );
    assert_eq!(title_metrics.text_left, 8.0);
    assert_eq!(title_metrics.strong_offset_x, 0.5);
    assert_eq!(title_metrics.separator_height, 1.0);
    assert_eq!(glyph_metrics.icon_size, METRICS.font_body);
    assert_eq!(glyph_metrics.icon_gap, 8.0);
}

#[test]
fn section_title_surface_projects_unreal_header_palette() {
    let host = current_host_palette();
    let palette = section_title_palette_from_host(host);

    assert_eq!(palette.header_surface, host.surface_pressed);
    assert_eq!(palette.separator, host.separator_soft);
}
