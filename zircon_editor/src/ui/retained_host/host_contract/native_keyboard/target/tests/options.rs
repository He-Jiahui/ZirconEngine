use super::*;

#[test]
fn asset_dropdown_keeps_its_dispatch_source_for_keyboard_selection() {
    let mut node = TemplatePaneNodeData::default();
    node.dispatch_kind = "asset:browser".into();

    assert_eq!(option_popup_dispatch_kind(&node), "asset:browser");
}

#[test]
fn ordinary_dropdown_keeps_the_workbench_option_dispatch() {
    let node = TemplatePaneNodeData::default();

    assert_eq!(option_popup_dispatch_kind(&node), "workbench_option");
}
