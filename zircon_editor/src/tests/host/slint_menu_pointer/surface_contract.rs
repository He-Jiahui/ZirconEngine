use super::support::*;

#[test]
fn shared_menu_pointer_layout_sync_replaces_direct_slint_menu_button_frame_getters() {
    let pointer_layout = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/pointer_layout.rs"
    ));

    for getter in [
        "get_file_menu_button_frame()",
        "get_edit_menu_button_frame()",
        "get_selection_menu_button_frame()",
        "get_view_menu_button_frame()",
        "get_window_menu_button_frame()",
        "get_help_menu_button_frame()",
    ] {
        assert!(
            !pointer_layout.contains(getter),
            "menu pointer sync should not keep direct Slint geometry getter `{getter}`"
        );
    }

    assert!(
        pointer_layout.contains("build_workbench_menu_pointer_layout("),
        "menu pointer sync should delegate top-level button frame authority to a shared layout builder"
    );
}

#[test]
fn shared_menu_popup_presentation_drops_host_menu_button_frame_setters() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let scaffold = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_scaffold.slint"
    ));
    let host_components = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_components.slint"
    ));
    let pointer_layout = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/pointer_layout.rs"
    ));

    for legacy_anchor in [
        "x: top_bar.file_menu_button_local_frame.x * 1px;",
        "x: top_bar.edit_menu_button_local_frame.x * 1px;",
        "x: top_bar.selection_menu_button_local_frame.x * 1px;",
        "x: top_bar.view_menu_button_local_frame.x * 1px;",
        "x: top_bar.window_menu_button_local_frame.x * 1px;",
        "x: top_bar.help_menu_button_local_frame.x * 1px;",
    ] {
        assert!(
            !workbench.contains(legacy_anchor),
            "menu popup presentation should not anchor to legacy local frame `{legacy_anchor}`"
        );
    }

    for projected_anchor in [
        "x: root.file_menu_button_frame.x * 1px;",
        "x: root.edit_menu_button_frame.x * 1px;",
        "x: root.selection_menu_button_frame.x * 1px;",
        "x: root.view_menu_button_frame.x * 1px;",
        "x: root.window_menu_button_frame.x * 1px;",
        "x: root.help_menu_button_frame.x * 1px;",
    ] {
        assert!(
            host_components.contains(projected_anchor),
            "host menu chrome is missing shared projected anchor `{projected_anchor}`"
        );
    }

    for removed_setter in [
        "set_file_menu_button_frame(",
        "set_edit_menu_button_frame(",
        "set_selection_menu_button_frame(",
        "set_view_menu_button_frame(",
        "set_window_menu_button_frame(",
        "set_help_menu_button_frame(",
    ] {
        assert!(
            !pointer_layout.contains(removed_setter),
            "menu popup presentation should not keep host menu frame setter `{removed_setter}`"
        );
    }

    for removed_binding in [
        "file_menu_button_frame <=> host.file_menu_button_frame",
        "edit_menu_button_frame <=> host.edit_menu_button_frame",
        "selection_menu_button_frame <=> host.selection_menu_button_frame",
        "view_menu_button_frame <=> host.view_menu_button_frame",
        "window_menu_button_frame <=> host.window_menu_button_frame",
        "help_menu_button_frame <=> host.help_menu_button_frame",
    ] {
        assert!(
            !workbench.contains(removed_binding) && !scaffold.contains(removed_binding),
            "menu popup presentation should not keep root/scaffold frame binding `{removed_binding}`"
        );
    }
}
