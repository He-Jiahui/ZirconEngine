use std::path::Path;

pub(super) fn is_meta_sidecar(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".meta.toml"))
}
