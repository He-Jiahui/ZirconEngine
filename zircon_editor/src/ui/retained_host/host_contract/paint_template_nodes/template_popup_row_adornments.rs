mod flags;
mod geometry;
mod glyphs;
mod selection;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use flags::menu_item_flag_value;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use flags::{
    menu_item_has_flag, menu_item_loading_and_danger,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use glyphs::push_popup_row_adornment;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::{
    menu_row_adornment_kind, option_adornment_kind, PopupRowAdornmentKind,
};
