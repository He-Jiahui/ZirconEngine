use super::support::collect_rust_files;

fn source_file(path: &[&str]) -> String {
    let mut file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for segment in path {
        file.push(segment);
    }
    std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("expected readable source {file:?}"))
}

fn source_tree(path: &[&str]) -> String {
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for segment in path {
        root.push(segment);
    }
    assert!(root.exists(), "expected readable source tree {root:?}");
    let mut files = collect_rust_files(&root);
    files.sort();
    assert!(
        !files.is_empty(),
        "expected Rust source files under {root:?}"
    );

    let mut source = String::new();
    for file in files {
        source.push_str(
            &std::fs::read_to_string(&file)
                .unwrap_or_else(|_| panic!("expected readable source {file:?}")),
        );
        source.push('\n');
    }
    source
}

fn assert_asset_exists(asset_path: &str) {
    let file =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(asset_path.trim_start_matches('/'));
    assert!(
        file.exists(),
        "expected workbench template asset to exist at {file:?}"
    );
}

fn assert_contains(source_name: &str, source: &str, pattern: &str) {
    assert!(
        source.contains(pattern),
        "expected {source_name} to contain `{pattern}`"
    );
}

fn assert_does_not_contain(source_name: &str, source: &str, pattern: &str) {
    assert!(
        !source.contains(pattern),
        "expected {source_name} to avoid host-only cutover path `{pattern}`"
    );
}

fn assert_no_active_retained_files(root: &std::path::Path) {
    let former_extension = former_generated_ui_extension();
    if !root.exists() {
        return;
    }
    for entry in std::fs::read_dir(root).unwrap_or_else(|_| panic!("expected readable {root:?}")) {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            assert_no_active_retained_files(&path);
            continue;
        }
        assert_ne!(
            path.extension().and_then(|extension| extension.to_str()),
            Some(former_extension.as_str()),
            "active editor host tree must not keep former generated UI source `{}`",
            path.display()
        );
    }
}

fn former_generated_ui_extension() -> String {
    ["sli", "nt"].concat()
}

fn retained_host_import_blocks(source: &str) -> Vec<String> {
    let normalized = source.split_whitespace().collect::<String>();
    let mut blocks = Vec::new();
    let mut rest = normalized.as_str();

    while let Some(start) = rest.find("usecrate::ui::retained_host::{") {
        let after_start = &rest[start..];
        let Some(end) = after_start.find("};") else {
            break;
        };
        blocks.push(after_start[..end + 2].to_string());
        rest = &after_start[end + 2..];
    }

    blocks
}

mod dto_boundary;
mod hit_contract;
mod layout_frames;
mod prototype_store;
mod template_reflection;
