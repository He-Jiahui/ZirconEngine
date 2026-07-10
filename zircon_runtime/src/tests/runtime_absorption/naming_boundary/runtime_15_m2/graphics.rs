use std::fs;
use std::path::{Path, PathBuf};

use super::super::support::{
    assert_contains_all, read_repo_text, read_runtime_15_naming_date_map,
    read_runtime_15_naming_status_map, read_runtime_15_naming_status_rows, read_text,
};

#[path = "graphics/gpu_model_embedded_primitive.rs"]
mod gpu_model_embedded_primitive;
#[path = "graphics/hybrid_gi.rs"]
mod hybrid_gi;
#[path = "graphics/offscreen_target_construct.rs"]
mod offscreen_target_construct;
#[path = "graphics/render_fixtures.rs"]
mod render_fixtures;
#[path = "graphics/render_framework_receiver.rs"]
mod render_framework_receiver;
#[path = "graphics/resource_streamer_construction.rs"]
mod resource_streamer_construction;

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("render framework directory should be readable") {
        let entry = entry.expect("render framework entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
