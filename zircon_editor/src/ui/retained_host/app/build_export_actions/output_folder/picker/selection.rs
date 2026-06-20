use std::path::{Path, PathBuf};

pub(in crate::ui::retained_host::app::build_export_actions) fn stable_picker_initial_dir(
    preferred: &Path,
    fallback: &Path,
) -> PathBuf {
    preferred
        .ancestors()
        .find(|ancestor| ancestor.is_dir())
        .unwrap_or(fallback)
        .to_path_buf()
}

pub(in crate::ui::retained_host::app::build_export_actions::output_folder) fn parse_selected_folder(
    stdout: &[u8],
) -> Option<PathBuf> {
    let selected = String::from_utf8_lossy(stdout).trim().to_string();
    (!selected.is_empty()).then(|| PathBuf::from(selected))
}
