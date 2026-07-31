use std::io;

use crate::ui::host::project_access::percent_encode_diagnostic_token;
use crate::ui::workbench::snapshot::EditorDataSnapshot;

pub(super) fn editor_product_frame_diagnostics(
    snapshot: &EditorDataSnapshot,
) -> Result<String, io::Error> {
    if !snapshot.project_open || snapshot.project_path.trim().is_empty() {
        return Err(io::Error::other(
            "editor product frame capture requires one opened project",
        ));
    }
    let mut selected_entries = snapshot.scene_entries.iter().filter(|entry| entry.selected);
    let selected = selected_entries.next().ok_or_else(|| {
        io::Error::other("editor product frame capture requires one selected scene node")
    })?;
    if selected_entries.next().is_some() {
        return Err(io::Error::other(
            "editor product frame capture found multiple selected scene nodes",
        ));
    }
    let inspector = snapshot.inspector.as_ref().ok_or_else(|| {
        io::Error::other("editor product frame capture requires a visible Inspector projection")
    })?;
    if inspector.id != selected.id || inspector.name != selected.name {
        return Err(io::Error::other(format!(
            "editor product frame capture Inspector '{}' ({}) differs from selected node '{}' ({})",
            inspector.name, inspector.id, selected.name, selected.id
        )));
    }
    if inspector
        .translation
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return Err(io::Error::other(
            "editor product frame capture Inspector has an empty translation field",
        ));
    }

    Ok(format!(
        "editor_product_frame_diagnostics project_path={} selected_node_id={} selected_node_name={} inspector_translation_x={} inspector_translation_y={} inspector_translation_z={}",
        percent_encode_diagnostic_token(&snapshot.project_path),
        selected.id,
        percent_encode_diagnostic_token(&selected.name),
        percent_encode_diagnostic_token(&inspector.translation[0]),
        percent_encode_diagnostic_token(&inspector.translation[1]),
        percent_encode_diagnostic_token(&inspector.translation[2]),
    ))
}
