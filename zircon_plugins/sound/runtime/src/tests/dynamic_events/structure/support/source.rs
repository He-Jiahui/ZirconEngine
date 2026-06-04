use std::path::{Path, PathBuf};

pub(crate) fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

pub(super) fn read_source(src: &Path, relative_path: &str) -> String {
    std::fs::read_to_string(src.join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}
