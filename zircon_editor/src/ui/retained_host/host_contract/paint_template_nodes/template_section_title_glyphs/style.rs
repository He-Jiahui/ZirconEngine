use super::identity::SectionTitleIcon;

const SECTION_GLYPH: [u8; 4] = [155, 173, 181, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SECTION_TRANSFORM_GLYPH:
    [u8; 4] = [155, 173, 181, 97];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_icon_color(
    icon: SectionTitleIcon,
) -> [u8; 4] {
    match icon {
        SectionTitleIcon::Transform => SECTION_TRANSFORM_GLYPH,
        SectionTitleIcon::Cube | SectionTitleIcon::Mesh => SECTION_GLYPH,
    }
}
