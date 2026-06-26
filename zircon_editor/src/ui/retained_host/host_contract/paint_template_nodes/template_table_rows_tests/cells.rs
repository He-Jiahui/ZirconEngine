use super::super::super::super::data::TemplatePaneNodeData;
use super::super::cells::{split_legacy_table_text, table_cells};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::SharedString;

#[test]
fn table_cells_prefer_declared_options_over_legacy_text() {
    let node = TemplatePaneNodeData {
        text: "Legacy Row".into(),
        options: model_rc(vec![
            SharedString::from("Item_02"),
            SharedString::from("Material"),
            SharedString::from("512 KB"),
            SharedString::from("10m ago"),
        ]),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        table_cells(&node),
        vec!["Item_02", "Material", "512 KB", "10m ago"]
    );
}

#[test]
fn table_cells_ignore_options_that_look_like_complete_rows() {
    let node = TemplatePaneNodeData {
        text: "Host UI 64K r42".into(),
        options: model_rc(vec![
            SharedString::from("Host UI 64K r42"),
            SharedString::from("Base Style 16K r42"),
            SharedString::from("Folder Tex 1.2M r42"),
            SharedString::from("A11y Widget 16K r42"),
        ]),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(table_cells(&node), vec!["Host", "UI", "64K", "r42"]);
}

#[test]
fn legacy_table_text_keeps_size_and_modified_units_together() {
    assert_eq!(
        split_legacy_table_text("Item_03     Texture     1.2 MB      1h ago"),
        vec!["Item_03", "Texture", "1.2 MB", "1h ago"]
    );
}
