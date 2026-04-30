use std::fs;

use super::native_dynamic_preparation::NativeDynamicPreparation;

pub(in crate::ui::host) fn cleanup_native_dynamic_preparation(
    preparation: &NativeDynamicPreparation,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for root in [&preparation.plugin_root, &preparation.build_root] {
        if !root.exists() {
            continue;
        }
        if let Err(error) = fs::remove_dir_all(root) {
            diagnostics.push(format!(
                "failed to remove native dynamic temporary directory {}: {error}",
                root.display()
            ));
        }
    }
    diagnostics
}
