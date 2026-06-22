use crate::ui::retained_host::primitives::Color;

use super::super::super::template_section_title_glyphs::{
    section_icon_color, SectionTitleIcon, SECTION_TRANSFORM_GLYPH,
};
use super::super::style::{section_text_color, SECTION_MESH_TEXT};
use super::support::title_node;

#[test]
fn mesh_renderer_section_title_uses_audited_title_tone() {
    assert_eq!(
        section_text_color(&title_node("WorkbenchMeshLabel", "Mesh Renderer")),
        SECTION_MESH_TEXT
    );
}

#[test]
fn section_title_uses_declared_title_tone() {
    let mut node = title_node("WorkbenchSelectionTitle", "Checkboxes & Radios");
    node.label_color = Color::from_rgb_u8(152, 163, 168);

    assert_eq!(section_text_color(&node), [152, 163, 168, 255]);
}

#[test]
fn transform_section_title_uses_audited_icon_opacity() {
    assert_eq!(
        section_icon_color(SectionTitleIcon::Transform),
        SECTION_TRANSFORM_GLYPH
    );
    assert_eq!(SECTION_TRANSFORM_GLYPH[3], 97);
}
