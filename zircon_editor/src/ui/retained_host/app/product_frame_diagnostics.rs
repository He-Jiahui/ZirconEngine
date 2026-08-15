use std::io;
use std::path::Path;

use zircon_runtime::asset::project::ProjectPaths;

use crate::core::logging::{EditorLogService, LogEntry, LogSeverity, LogSource};
use crate::ui::host::project_access::percent_encode_diagnostic_token;
use crate::ui::workbench::snapshot::EditorDataSnapshot;

// Product-frame evidence is gathered before the retained UI begins frame pumping.
const UNKNOWN_PRODUCT_FRAME_LOG_FRAME: u64 = 0;

pub(super) fn editor_product_frame_diagnostics(
    snapshot: &EditorDataSnapshot,
) -> Result<String, io::Error> {
    if !snapshot.project_open || snapshot.project_path.trim().is_empty() {
        return Err(io::Error::other(
            "editor product frame capture requires one opened project",
        ));
    }
    let mut selected_entries = snapshot
        .scene_entries
        .iter()
        .filter(|entry| snapshot.scene_entries.is_selected(entry.entity));
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
    if inspector.id != selected.entity || inspector.name != selected.display_name {
        return Err(io::Error::other(format!(
            "editor product frame capture Inspector '{}' ({}) differs from selected node '{}' ({})",
            inspector.name, inspector.id, selected.display_name, selected.entity
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
    if inspector.scale.iter().any(|value| value.trim().is_empty()) {
        return Err(io::Error::other(
            "editor product frame capture Inspector has an empty scale field",
        ));
    }

    Ok(format!(
        "editor_product_frame_diagnostics project_path={} selected_node_id={} selected_node_name={} inspector_translation_x={} inspector_translation_y={} inspector_translation_z={} inspector_scale_x={} inspector_scale_y={} inspector_scale_z={}",
        product_frame_project_path_token(&snapshot.project_path),
        selected.entity,
        percent_encode_diagnostic_token(&selected.display_name),
        percent_encode_diagnostic_token(&inspector.translation[0]),
        percent_encode_diagnostic_token(&inspector.translation[1]),
        percent_encode_diagnostic_token(&inspector.translation[2]),
        percent_encode_diagnostic_token(&inspector.scale[0]),
        percent_encode_diagnostic_token(&inspector.scale[1]),
        percent_encode_diagnostic_token(&inspector.scale[2]),
    ))
}

pub(super) fn emit_product_frame_log(logs: &EditorLogService, diagnostic: String) {
    let entry = LogEntry::new(
        LogSource::editor(),
        LogSeverity::Info,
        diagnostic,
        UNKNOWN_PRODUCT_FRAME_LOG_FRAME,
        None,
    )
    .or_else(|_| {
        LogEntry::new(
            LogSource::editor(),
            LogSeverity::Info,
            "editor_product_frame diagnostic exceeds the log-entry limit.",
            UNKNOWN_PRODUCT_FRAME_LOG_FRAME,
            None,
        )
    });
    if let Ok(entry) = entry {
        let _ = logs.emit(entry);
    }
}

fn product_frame_project_path_token(project_path: &str) -> String {
    let display_path = ProjectPaths::display_path(Path::new(project_path));
    percent_encode_diagnostic_token(&display_path.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};

    use super::{emit_product_frame_log, product_frame_project_path_token};

    #[cfg(windows)]
    #[test]
    fn product_frame_diagnostic_uses_a_display_path_for_verbatim_project_roots() {
        let token = product_frame_project_path_token(r"\\?\C:\projects\renderable empty");

        assert_eq!(token, "C%3A%5Cprojects%5Crenderable%20empty");
    }

    #[test]
    fn oversized_product_frame_diagnostic_uses_a_bounded_info_log() {
        let logs = EditorLogService::default();

        emit_product_frame_log(&logs, "x".repeat(9 * 1024));

        let records = logs.snapshot(&LogFilter::default());
        assert_eq!(records.len(), 1);
        let entry = records[0].entry();
        assert_eq!(entry.source(), &LogSource::editor());
        assert_eq!(entry.severity(), LogSeverity::Info);
        assert_eq!(
            entry.message(),
            "editor_product_frame diagnostic exceeds the log-entry limit."
        );
    }
}
