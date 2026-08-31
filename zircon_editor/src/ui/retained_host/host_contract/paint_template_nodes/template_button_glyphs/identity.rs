#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum ButtonGlyph {
    None,
    Plus,
    Trash,
    ChevronDown,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_glyph_for_key(
    key: &str,
) -> ButtonGlyph {
    if key.len() < 3 {
        return ButtonGlyph::None;
    }
    if key.contains("delete") || key.contains("trash") || key.contains("danger") {
        ButtonGlyph::Trash
    } else if key.contains("dropdown") || key.contains("drop-down") || key.contains("menu") {
        ButtonGlyph::ChevronDown
    } else if key.contains("icon") || key.contains("add") || key.contains("plus") {
        ButtonGlyph::Plus
    } else {
        ButtonGlyph::None
    }
}

#[cfg(test)]
#[path = "identity/short_key_tests.rs"]
mod short_key_tests;
