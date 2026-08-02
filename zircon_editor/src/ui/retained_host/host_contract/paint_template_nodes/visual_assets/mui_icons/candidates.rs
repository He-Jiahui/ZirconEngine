use std::path::{Path, PathBuf};

use super::names::module_name;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn module_candidates(
    icon_name: &str,
    workspace_root: &Path,
) -> Vec<PathBuf> {
    let Some(module_name) = module_name(icon_name) else {
        return Vec::new();
    };
    vec![
        workspace_root
            .join("dev/material-ui/packages/mui-icons-material/lib")
            .join(module_name)
            .with_extension("js"),
    ]
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_module_path(
    path: &Path,
) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("js"))
        && path
            .to_string_lossy()
            .replace('\\', "/")
            .contains("/mui-icons-material/lib/")
}
