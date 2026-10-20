use super::super::super::super::data::TemplatePaneNodeData;
use super::super::cells::{split_archived_table_text, table_cells};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::SharedString;

#[test]
fn table_cells_prefer_declared_options_over_archived_text() {
    let node = TemplatePaneNodeData {
        text: "Archived Row".into(),
        options: model_rc(vec![
            SharedString::from("Item_02"),
            SharedString::from("Material"),
            SharedString::from("512 KB"),
            SharedString::from("10m ago"),
        ]),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        table_cells(&node).as_ref(),
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

    assert_eq!(
        table_cells(&node).as_ref(),
        vec!["Host", "UI", "64 KB", "rev 42"]
    );
}

#[test]
fn archived_table_text_keeps_size_and_modified_units_together() {
    assert_eq!(
        split_archived_table_text("Item_03     Texture     1.2 MB      1h ago"),
        vec!["Item_03", "Texture", "1.2 MB", "1h ago"]
    );
}

#[test]
fn projected_cells_borrow_storage_and_reject_changed_text_or_options() {
    let mut node = TemplatePaneNodeData {
        role: "TableRow".into(),
        text: "Host UI 64K r42".into(),
        ..Default::default()
    };
    node.prepare_table_cells();
    assert!(matches!(table_cells(&node), std::borrow::Cow::Borrowed(_)));
    let clone = node.clone();
    assert_eq!(table_cells(&node).as_ptr(), table_cells(&clone).as_ptr());
    node.text = "New Tex 1M r43".into();
    assert_eq!(
        table_cells(&node).as_ref(),
        &["New", "Texture", "1 MB", "rev 43"]
    );
    assert!(matches!(table_cells(&node), std::borrow::Cow::Owned(_)));
    node.prepare_table_cells();
    assert!(matches!(table_cells(&node), std::borrow::Cow::Borrowed(_)));
    node.options = model_rc(vec![
        "Declared".into(),
        "Mat".into(),
        "4K".into(),
        "r44".into(),
    ]);
    assert_eq!(
        table_cells(&node).as_ref(),
        &["Declared", "Material", "4 KB", "rev 44"]
    );
}
