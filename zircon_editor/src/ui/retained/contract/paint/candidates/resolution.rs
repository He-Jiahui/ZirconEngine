use std::path::{Path, PathBuf};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn first_existing_path(
    candidates: Vec<PathBuf>,
) -> Option<PathBuf> {
    candidates.into_iter().find(|path| path.exists())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_svg_path(
    path: &Path,
) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
}
