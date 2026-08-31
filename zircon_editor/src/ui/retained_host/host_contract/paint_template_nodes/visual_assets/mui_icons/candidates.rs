use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use super::names::module_name;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn module_candidates(
    icon_name: &str,
    workspace_root: &Path,
) -> Vec<PathBuf> {
    let Some(module_name) = module_name(icon_name) else {
        return Vec::new();
    };
    vec![workspace_root
        .join("dev/material-ui/packages/mui-icons-material/lib")
        .join(module_name)
        .with_extension("js")]
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_module_path(
    path: &Path,
) -> bool {
    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("js"))
    {
        return false;
    }

    let mut has_leading_component = false;
    let mut previous_was_module = false;
    for component in path.components() {
        let Component::Normal(component) = component else {
            has_leading_component = true;
            previous_was_module = false;
            continue;
        };
        if previous_was_module && component == OsStr::new("lib") {
            return true;
        }
        previous_was_module =
            has_leading_component && component == OsStr::new("mui-icons-material");
        has_leading_component = true;
    }
    false
}

#[cfg(test)]
#[path = "candidates/component_scan_tests.rs"]
mod component_scan_tests;
