use super::super::super::data::TemplatePaneNodeData;
use super::kind::IconButtonGlyphKind;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_glyph_kind(
    node: &TemplatePaneNodeData,
) -> IconButtonGlyphKind {
    let key =
        format!("{} {}", node.control_id.as_str(), node.icon_name.as_str()).to_ascii_lowercase();
    if key.contains("menu") {
        IconButtonGlyphKind::Menu
    } else if key.contains("file-new") || key.contains("toolbarnew") {
        IconButtonGlyphKind::File
    } else if key.contains("add") {
        IconButtonGlyphKind::Plus
    } else if key.contains("open") || key.contains("folder") {
        IconButtonGlyphKind::Folder
    } else if key.contains("save") {
        IconButtonGlyphKind::Save
    } else if key.contains("select") || key.contains("cursor") {
        IconButtonGlyphKind::Cursor
    } else if key.contains("move") {
        IconButtonGlyphKind::Move
    } else if key.contains("rotate") {
        IconButtonGlyphKind::Rotate
    } else if key.contains("scale") || key.contains("fullscreen") {
        IconButtonGlyphKind::Scale
    } else if key.contains("snap") || key.contains("magnet") {
        IconButtonGlyphKind::Snap
    } else if key.contains("play") || key.contains("runplay") || key.contains("railscene") {
        IconButtonGlyphKind::Play
    } else if key.contains("chevron") || key.contains("overflow") || key.contains("runmode") {
        IconButtonGlyphKind::ChevronDown
    } else if key.contains("layout")
        || key.contains("grid")
        || key.contains("columns")
        || key.contains("list")
    {
        IconButtonGlyphKind::Grid
    } else if key.contains("theme") || key.contains("sun") || key.contains("command-palette") {
        IconButtonGlyphKind::Sun
    } else if key.contains("delete") || key.contains("trash") {
        IconButtonGlyphKind::Trash
    } else if key.contains("filter") {
        IconButtonGlyphKind::Filter
    } else if key.contains("cube") {
        IconButtonGlyphKind::Cube
    } else if key.contains("graph") {
        IconButtonGlyphKind::Graph
    } else if key.contains("image") {
        IconButtonGlyphKind::Image
    } else if key.contains("audio") {
        IconButtonGlyphKind::Audio
    } else if key.contains("code") {
        IconButtonGlyphKind::Code
    } else if key.contains("eye-off") || key.contains("eyeoff") {
        IconButtonGlyphKind::EyeOff
    } else if key.contains("eye") {
        IconButtonGlyphKind::Eye
    } else if key.contains("lock") {
        IconButtonGlyphKind::Lock
    } else {
        IconButtonGlyphKind::More
    }
}
