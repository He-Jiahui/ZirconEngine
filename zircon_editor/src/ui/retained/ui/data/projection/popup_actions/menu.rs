use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;

use super::super::super::pane_menu_projection::structured_menu_items;
use super::super::super::pane_value_conversion::value_as_options;

pub(super) struct ProjectedPopupMenu {
    pub(super) menu_items: Vec<String>,
    pub(super) structured_menu_items: Vec<host_contract::TemplatePaneMenuItemData>,
}

pub(super) fn projected_popup_menu(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedPopupMenu {
    let menu_items = attributes
        .get("menu_items")
        .and_then(value_as_options)
        .unwrap_or_default();
    let structured_menu_items = structured_menu_items(&menu_items);

    ProjectedPopupMenu {
        menu_items,
        structured_menu_items,
    }
}
