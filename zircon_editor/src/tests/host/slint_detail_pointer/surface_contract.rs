#[test]
fn shared_detail_scroll_surfaces_do_not_leave_slint_scrollview_as_authority() {
    let pane_content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_content.slint"
    ));
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/assets.slint"
    ));

    for needle in [
        "import { LineEdit, ScrollView } from \"std-widgets.slint\";",
        "ScrollView {\n        width: parent.width;\n        height: parent.height;\n        viewport-y: root.scroll_px * 1px;",
    ] {
        assert!(
            !pane_content.contains(needle),
            "detail panes still leave Slint ScrollView as scroll authority via `{needle}`"
        );
    }

    for needle in [
        "import { LineEdit, ScrollView } from \"std-widgets.slint\";",
        "ScrollView {\n        x: 0px;\n        y: root.header_height + 1px;",
    ] {
        assert!(
            !assets.contains(needle),
            "asset details rail still leaves Slint ScrollView as scroll authority via `{needle}`"
        );
    }
}
