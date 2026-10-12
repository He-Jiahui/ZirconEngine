use super::*;

#[test]
fn virtualized_target_total_covers_its_visible_window_when_projection_lags() {
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchCommandPalette".into();
    node.virtualization_enabled = true;
    node.pagination_page_size = 12;
    node.virtualization_total_count = 1;
    node.virtualization_visible_start = 12;
    let rows = (0..12).map(row).collect();

    let target = popup_keyboard_target_from_rows(
        &node,
        "workbench_option",
        rows,
        FrameRect::default(),
        &HostPaneInteractionStateData::default(),
    )
    .expect("visible command rows should produce a keyboard target");

    assert_eq!(target.total_count, 24);
}

fn row(index: usize) -> PopupKeyboardRow {
    PopupKeyboardRow {
        action_id: format!("command_{index}").into(),
        value_text: format!("command_{index}").into(),
        identity: format!("command_{index}").into(),
        search_text: format!("Command {index}").into(),
        focused: false,
        selected: false,
        source_index: Some(index),
        frame: FrameRect::default(),
    }
}
