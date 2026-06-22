use super::super::super::style_selector::WORKBENCH_POPUP_ROW_DANGER_TEXT as POPUP_ROW_DANGER_TEXT;
use super::super::super::template_popup_row_adornments::{
    menu_item_flag_value, PopupRowAdornmentKind,
};
use super::super::{menu_item_has_flag, menu_row_adornment_kind, popup_menu_row_style};
use super::support::menu_item;

#[test]
fn menu_item_adornment_kind_reads_icon_danger_and_submenu_flags() {
    let delete = menu_item("Delete|danger,icon=trash", false, false, false);
    let more = menu_item("More Tools|submenu", false, false, false);
    let save = menu_item("Save", false, false, false);

    assert!(menu_item_has_flag(&delete, "danger"));
    assert_eq!(
        menu_item_flag_value(&delete, "icon").as_deref(),
        Some("trash")
    );
    assert_eq!(
        menu_row_adornment_kind(&delete),
        Some(PopupRowAdornmentKind::Trash)
    );
    assert_eq!(
        menu_row_adornment_kind(&more),
        Some(PopupRowAdornmentKind::Chevron)
    );
    assert_eq!(
        menu_row_adornment_kind(&save),
        Some(PopupRowAdornmentKind::Save)
    );
    assert_eq!(popup_menu_row_style(&delete).text, POPUP_ROW_DANGER_TEXT);
    assert_eq!(
        popup_menu_row_style(&delete).adornment,
        POPUP_ROW_DANGER_TEXT
    );
}
