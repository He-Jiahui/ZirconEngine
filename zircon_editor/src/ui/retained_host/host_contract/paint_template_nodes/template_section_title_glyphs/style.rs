use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::identity::SectionTitleIcon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSectionTitleGlyphPalette
{
    pub icon: [u8; 4],
    pub transform_icon: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_title_glyph_palette(
) -> WorkbenchSectionTitleGlyphPalette {
    section_title_glyph_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_title_glyph_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchSectionTitleGlyphPalette {
    WorkbenchSectionTitleGlyphPalette {
        icon: palette.text_muted,
        transform_icon: palette.text_disabled,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_icon_color(
    icon: SectionTitleIcon,
) -> [u8; 4] {
    let palette = section_title_glyph_palette();
    match icon {
        SectionTitleIcon::Transform => palette.transform_icon,
        SectionTitleIcon::Cube | SectionTitleIcon::Mesh => palette.icon,
    }
}
