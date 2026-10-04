use crate::ui::asset_editor::UiAssetEditorSession;

pub(super) fn select_palette_entry(session: &mut UiAssetEditorSession, label: &str) {
    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == label)
        .unwrap_or_else(|| panic!("palette item {label}"));
    session
        .select_palette_index(palette_index)
        .expect("select palette item");
}

pub(super) fn preview_frame(
    session: &UiAssetEditorSession,
    node_id: &str,
) -> crate::ui::asset_editor::UiAssetEditorPreviewCanvasNode {
    session
        .pane_presentation()
        .preview_canvas_items
        .into_iter()
        .find(|item| item.node_id == node_id)
        .unwrap_or_else(|| panic!("preview frame {node_id}"))
}

pub(super) fn numeric_slot_value(
    slot: &std::collections::BTreeMap<String, toml::Value>,
    path: &[&str],
) -> Option<f64> {
    let mut current = slot.get(path.first().copied()?)?;
    for segment in &path[1..] {
        current = current.as_table()?.get(*segment)?;
    }
    current
        .as_float()
        .or_else(|| current.as_integer().map(|value| value as f64))
}
