const WORKBENCH_CAPTION_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/data/workbench_caption.zui"
);
const WORKBENCH_LABEL_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/data/workbench_label.zui"
);
const WORKBENCH_SECTION_TITLE_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui"
);
const WORKBENCH_TREE_ROW_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/data/workbench_tree_row.zui"
);
const WORKBENCH_LIST_ROW_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/data/workbench_list_row.zui"
);
const WORKBENCH_TABLE_ROW_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/data/workbench_table_row.zui"
);

#[test]
fn bounded_text_primitives_declare_runtime_ellipsis() {
    for (asset_name, asset) in [
        ("workbench_caption.zui", WORKBENCH_CAPTION_ASSET),
        ("workbench_label.zui", WORKBENCH_LABEL_ASSET),
        ("workbench_section_title.zui", WORKBENCH_SECTION_TITLE_ASSET),
    ] {
        assert!(
            asset.contains("text_overflow = \"ellipsis\""),
            "{asset_name} has a fixed-height layout and must ellipsize text through the runtime text path"
        );
        assert!(
            asset.contains("line_height_ratio = \"$editor.typography.line_height\""),
            "{asset_name} must retain the shared runtime text line-height token"
        );
    }
}

#[test]
fn bounded_row_primitives_declare_runtime_ellipsis() {
    for (asset_name, asset) in [
        ("workbench_tree_row.zui", WORKBENCH_TREE_ROW_ASSET),
        ("workbench_list_row.zui", WORKBENCH_LIST_ROW_ASSET),
        ("workbench_table_row.zui", WORKBENCH_TABLE_ROW_ASSET),
    ] {
        assert!(
            asset.contains("text_overflow = \"ellipsis\""),
            "{asset_name} has a fixed row height and must ellipsize dynamic names through the runtime text path"
        );
        assert!(
            asset.contains("layout_min_height = \"$editor.control.height.dense\""),
            "{asset_name} must keep its shared dense-row metric"
        );
    }
}
