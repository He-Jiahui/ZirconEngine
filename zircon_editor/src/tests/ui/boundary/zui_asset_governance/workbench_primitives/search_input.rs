use super::super::support::{editor_asset_root, load_zui_document};

#[test]
fn canonical_icon_assets_exist_without_dead_override_properties() {
    let path = editor_asset_root()
        .join("ui/editor/components")
        .join("workbench/primitives/inputs/workbench_search_input.zui");
    let document = load_zui_document(&path);
    let root = document
        .nodes
        .get("root")
        .expect("WorkbenchSearchInput root node");

    for property in ["search_icon", "clear_icon"] {
        assert!(
            !root.props.contains_key(property),
            "WorkbenchSearchInput `{property}` is not a customizable host contract"
        );
    }
    for relative_path in [
        "zircon_editor_shell/controls/search.svg",
        "ionicons/close-outline.svg",
    ] {
        let asset_path = editor_asset_root().join("icons").join(relative_path);
        assert!(
            asset_path.is_file(),
            "missing canonical SearchInput icon: {}",
            asset_path.display()
        );
    }
}
