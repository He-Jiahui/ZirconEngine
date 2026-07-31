use super::super::super::data::TemplatePaneMenuItemData;
use super::flags::{menu_item_flag_value, menu_item_has_flag};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum PopupRowAdornmentKind {
    Check,
    Chevron,
    Plus,
    Folder,
    Save,
    Trash,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn option_adornment_kind(
    selected: bool,
) -> Option<PopupRowAdornmentKind> {
    selected.then_some(PopupRowAdornmentKind::Check)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn menu_row_adornment_kind(
    item: &TemplatePaneMenuItemData,
) -> Option<PopupRowAdornmentKind> {
    if menu_item_has_flag(item, "submenu") {
        return Some(PopupRowAdornmentKind::Chevron);
    }
    if item.checked {
        return Some(PopupRowAdornmentKind::Check);
    }
    if let Some(icon) = menu_item_flag_value(item, "icon") {
        return popup_row_adornment_from_icon(icon);
    }
    menu_item_default_adornment(item.label.as_str())
}

fn popup_row_adornment_from_icon(icon: &str) -> Option<PopupRowAdornmentKind> {
    let icon = icon.trim();
    if ascii_matches_any(icon, &["add", "new", "plus"]) {
        Some(PopupRowAdornmentKind::Plus)
    } else if ascii_matches_any(icon, &["open", "folder"]) {
        Some(PopupRowAdornmentKind::Folder)
    } else if ascii_matches_any(icon, &["save", "disk"]) {
        Some(PopupRowAdornmentKind::Save)
    } else if ascii_matches_any(icon, &["delete", "remove", "trash"]) {
        Some(PopupRowAdornmentKind::Trash)
    } else if ascii_matches_any(icon, &["submenu", "more", "chevron"]) {
        Some(PopupRowAdornmentKind::Chevron)
    } else if ascii_matches_any(icon, &["check", "checked"]) {
        Some(PopupRowAdornmentKind::Check)
    } else {
        None
    }
}

fn menu_item_default_adornment(label: &str) -> Option<PopupRowAdornmentKind> {
    let label = label.trim();
    if label.eq_ignore_ascii_case("new") {
        Some(PopupRowAdornmentKind::Plus)
    } else if label.eq_ignore_ascii_case("open") {
        Some(PopupRowAdornmentKind::Folder)
    } else if label.eq_ignore_ascii_case("save") {
        Some(PopupRowAdornmentKind::Save)
    } else if label.eq_ignore_ascii_case("delete") {
        Some(PopupRowAdornmentKind::Trash)
    } else if label.eq_ignore_ascii_case("more tools") {
        Some(PopupRowAdornmentKind::Chevron)
    } else {
        None
    }
}

fn ascii_matches_any(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}
