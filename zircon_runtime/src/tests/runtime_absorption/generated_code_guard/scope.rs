use std::path::{Path, PathBuf};

use super::support::{collect_rust_source_files, relative_path};

#[test]
fn export_template_scan_scope_stays_folder_backed() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let template_files = export_template_files(manifest_root);
    let relative_files = template_files
        .iter()
        .map(|path| relative_path(manifest_root, path))
        .collect::<Vec<_>>();

    assert!(
        !relative_files.is_empty(),
        "generated-code guard should scan export template files"
    );
    assert!(
        relative_files
            .iter()
            .all(|path| path.starts_with("src/plugin/export_build_plan/")),
        "generated-code guard must stay scoped to export_build_plan templates:\n{}",
        relative_files.join("\n")
    );
}

pub(super) fn export_template_files(manifest_root: &Path) -> Vec<PathBuf> {
    let export_root = manifest_root
        .join("src")
        .join("plugin")
        .join("export_build_plan");
    let mut files = Vec::new();
    collect_rust_source_files(&export_root, &mut files);
    files.retain(|path| is_export_template_file(&export_root, path));
    files.sort();
    files
}

fn is_export_template_file(export_root: &Path, path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let relative = path
        .strip_prefix(export_root)
        .expect("export template path should live under export root")
        .to_string_lossy()
        .replace('\\', "/");

    file_name.contains("template")
        || file_name == "generated_files.rs"
        || file_name == "platform_host_files.rs"
        || relative.starts_with("platform_host_files/")
}
