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
        return popup_row_adornment_from_icon(&icon);
    }
    menu_item_default_icon(item.label.as_str()).and_then(popup_row_adornment_from_icon)
}

fn popup_row_adornment_from_icon(icon: &str) -> Option<PopupRowAdornmentKind> {
    match icon.trim().to_ascii_lowercase().as_str() {
        "add" | "new" | "plus" => Some(PopupRowAdornmentKind::Plus),
        "open" | "folder" => Some(PopupRowAdornmentKind::Folder),
        "save" | "disk" => Some(PopupRowAdornmentKind::Save),
        "delete" | "remove" | "trash" => Some(PopupRowAdornmentKind::Trash),
        "submenu" | "more" | "chevron" => Some(PopupRowAdornmentKind::Chevron),
        "check" | "checked" => Some(PopupRowAdornmentKind::Check),
        _ => None,
    }
}

fn menu_item_default_icon(label: &str) -> Option<&'static str> {
    match label.trim().to_ascii_lowercase().as_str() {
        "new" => Some("plus"),
        "open" => Some("folder"),
        "save" => Some("save"),
        "delete" => Some("trash"),
        "more tools" => Some("submenu"),
        _ => None,
    }
}
