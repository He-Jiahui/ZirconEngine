mod guard_layout;
mod production_scan;
mod runtime_owned;
mod runtime_ui;
mod script_host;
mod status_anchor_cleanup;
mod ui_text;

use super::{repo_path, runtime_src_path};

const DEAD_CODE_ALLOW_ATTRIBUTE: &str = concat!("#[allow(", "dead_code", ")]");
const DEAD_CODE_ALLOW_CALL_PREFIX: &str = concat!("allow(", "dead_code");

fn runtime_source_path(relative: &str) -> std::path::PathBuf {
    runtime_src_path(relative)
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_source_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}

fn dead_code_suppression_lines(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let compact: String = line.chars().filter(|ch| !ch.is_whitespace()).collect();
            if compact.contains(DEAD_CODE_ALLOW_CALL_PREFIX)
                && (compact.contains("#[") || compact.contains("#!["))
            {
                Some((index + 1, line.trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn collect_production_rust_sources(
    src_root: &std::path::Path,
    current_dir: &std::path::Path,
    sources: &mut Vec<std::path::PathBuf>,
) {
    for entry in std::fs::read_dir(current_dir)
        .unwrap_or_else(|error| panic!("failed to read directory `{current_dir:?}`: {error}"))
    {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read directory entry under `{current_dir:?}`: {error}")
        });
        let path = entry.path();
        if path.is_dir() {
            collect_production_rust_sources(src_root, &path, sources);
        } else if is_production_rust_source(src_root, &path) {
            sources.push(path);
        }
    }
}

fn is_production_rust_source(root: &std::path::Path, path: &std::path::Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return false;
    }

    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "tests.rs" || file_name.ends_with("_tests.rs") {
        return false;
    }

    let relative = path.strip_prefix(root).unwrap_or(path);
    !relative.components().any(|component| match component {
        std::path::Component::Normal(name) => name == std::ffi::OsStr::new("tests"),
        _ => false,
    })
}
