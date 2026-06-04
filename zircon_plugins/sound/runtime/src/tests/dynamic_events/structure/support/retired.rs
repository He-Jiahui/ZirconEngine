use std::path::{Path, PathBuf};

pub(crate) fn retired_flat_module(src: &Path, directory: &str, module: &str) -> PathBuf {
    let file_name = format!("{module}.rs");
    if directory.is_empty() {
        src.join(file_name)
    } else {
        src.join(directory).join(file_name)
    }
}
