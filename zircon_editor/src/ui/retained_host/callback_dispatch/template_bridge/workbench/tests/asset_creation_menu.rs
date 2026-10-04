use super::*;

#[test]
fn main_menu_width_measures_label_shortcut_and_trailing_icon() {
    let items = [
        "---".to_string(),
        "Command Palette|action=menu.item.command_palette,icon=search|Ctrl+Shift+P".to_string(),
    ];

    assert!(measure_main_menu_width(&items, 190.0, 1.0) > 190.0);
}
