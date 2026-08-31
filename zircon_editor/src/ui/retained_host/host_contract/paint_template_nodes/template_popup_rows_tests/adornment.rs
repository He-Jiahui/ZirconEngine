use super::super::super::style_selector::WORKBENCH_POPUP_ROW_DANGER_TEXT as POPUP_ROW_DANGER_TEXT;
use super::super::super::template_popup_row_adornments::{
    menu_item_flag_value, PopupRowAdornmentKind,
};
use super::super::{
    menu_item_has_flag, menu_row_adornment_kind, popup_menu_row_style, popup_row_content_style,
};
use super::support::menu_item;

#[test]
fn menu_item_adornment_kind_reads_icon_danger_and_submenu_flags() {
    let delete = menu_item("Delete|danger,icon=trash", false, false, false);
    let more = menu_item("More Tools|submenu", false, false, false);
    let save = menu_item("Save", false, false, false);

    assert!(menu_item_has_flag(&delete, "danger"));
    let icon: Option<&str> = menu_item_flag_value(&delete, "icon");
    assert_eq!(icon, Some("trash"));
    assert_eq!(
        menu_row_adornment_kind(&delete),
        Some(PopupRowAdornmentKind::Icon("trash"))
    );
    assert_eq!(
        menu_row_adornment_kind(&more),
        Some(PopupRowAdornmentKind::Chevron)
    );
    assert_eq!(
        menu_row_adornment_kind(&save),
        Some(PopupRowAdornmentKind::Icon("save"))
    );
    let content_style = popup_row_content_style(&popup_menu_row_style(&delete));
    assert_eq!(content_style.text, POPUP_ROW_DANGER_TEXT);
    assert_eq!(content_style.adornment, POPUP_ROW_DANGER_TEXT);
}

#[test]
fn menu_item_adornment_forwards_product_semantic_icons_without_a_whitelist() {
    for icon in [
        "copy",
        "edit",
        "grid",
        "pin",
        "play",
        "rotate-ccw",
        "search",
        "target",
    ] {
        let raw = format!("Action|icon={icon}");
        let item = menu_item(raw.as_str(), false, false, false);
        assert_eq!(
            menu_row_adornment_kind(&item),
            Some(PopupRowAdornmentKind::Icon(icon)),
            "{icon}"
        );
    }
}

#[test]
fn popup_rows_clip_before_model_clone_and_classify_adornment_once() {
    let menu = include_str!("../template_popup_rows/menu/entry.rs");
    let option = include_str!("../template_popup_rows/options/entry.rs");

    assert!(
        menu.find("let Some(row_rect) = menu_item_row_frame")
            .expect("menu frame")
            < menu.find("row_data(row)").expect("menu row data")
    );
    assert!(
        option
            .find("let Some(row_rect) = template_option_row_frame_within")
            .expect("option frame")
            < option.find("row_data(row)").expect("option row data")
    );
    assert_eq!(menu.matches("menu_row_adornment_kind(&item)").count(), 1);
}
