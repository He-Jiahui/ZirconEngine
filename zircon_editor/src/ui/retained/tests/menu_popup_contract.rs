use super::*;

#[test]
fn window_menu_viewport_consumes_shared_height_and_clamped_scroll() {
    let viewport = root_menu_popup_viewport(WINDOW_MENU_INDEX, 732.0, 192.0, 640.0);

    assert_eq!(viewport.height, 192.0);
    assert_eq!(viewport.scroll, 540.0);
}

#[test]
fn ordinary_menu_ignores_stale_window_scroll_state() {
    let viewport = root_menu_popup_viewport(0, 132.0, 192.0, 96.0);

    assert_eq!(viewport.height, 132.0);
    assert_eq!(viewport.scroll, 0.0);
}

#[test]
fn popup_width_follows_longest_runtime_measured_label_and_shortcut() {
    let rows = [("Short", ""), ("UI Component Showcase", "Ctrl+Shift+U")];

    let width = content_measured_menu_popup_width(224.0, 900.0, rows, |text| {
        text.chars().count() as f32 * 8.0
    });

    assert_eq!(width, 312.0);
}

#[test]
fn popup_width_clamps_to_available_shell_width() {
    let rows = [("Extremely long extension menu item", "Ctrl+Shift+Alt+P")];

    let width = content_measured_menu_popup_width(224.0, 260.0, rows, |text| {
        text.chars().count() as f32 * 10.0
    });

    assert_eq!(width, 260.0);
}

#[test]
fn popup_width_reserves_trailing_adornments_before_clamping() {
    let rows = [("Open Project", "Ctrl+O", 24.0)];
    let without_reserve =
        content_measured_menu_popup_width(1.0, 900.0, [("Open Project", "Ctrl+O")], |text| {
            text.chars().count() as f32 * 8.0
        });
    let with_reserve =
        content_measured_menu_popup_width_with_trailing_reserve(1.0, 900.0, rows, |text| {
            text.chars().count() as f32 * 8.0
        });
    let clamped =
        content_measured_menu_popup_width_with_trailing_reserve(1.0, 140.0, rows, |text| {
            text.chars().count() as f32 * 8.0
        });

    assert_eq!(with_reserve, without_reserve + 24.0);
    assert_eq!(clamped, 140.0);
}

#[test]
fn structured_popup_width_skips_separators_and_detects_semantic_adornments() {
    let items = [
        "---",
        "Rename|action=menu.item.rename,icon=edit|F2",
        "Delete|action=menu.item.delete,danger",
    ];
    let width = content_measured_structured_menu_popup_width(1.0, 900.0, items, 24.0, |text| {
        text.chars().count() as f32 * 8.0
    });

    assert_eq!(width, 136.0);
}

#[test]
fn popup_content_height_uses_shared_slate_row_density() {
    assert_eq!(menu_popup_content_height(0), 0.0);
    assert_eq!(menu_popup_content_height(3), 100.0);
}
