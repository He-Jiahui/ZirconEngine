use std::path::Path;

use crate::projects::project_filesystem_path_key;

pub(super) fn source_engine_id(source_dir: &Path) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    let key = source_engine_path_key(source_dir);
    for byte in key.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("source-{hash:016x}")
}

pub(super) fn same_source_engine_path(left: &Path, right: &Path) -> bool {
    source_engine_path_key(left) == source_engine_path_key(right)
}

fn source_engine_path_key(path: &Path) -> String {
    project_filesystem_path_key(path)
}

pub(super) fn source_engine_display_name(source_dir: &Path) -> String {
    source_dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(|name| format!("{name} Source"))
        .unwrap_or_else(|| "Local Source".to_string())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{same_source_engine_path, source_engine_id};

    #[test]
    fn source_engine_paths_share_project_filesystem_key_normalization() {
        assert!(same_source_engine_path(
            Path::new("E:\\Git\\ZirconEngine\\"),
            Path::new("e:/git/zirconengine")
        ));
        assert_eq!(
            source_engine_id(Path::new("E:\\Git\\ZirconEngine\\")),
            source_engine_id(Path::new("e:/git/zirconengine"))
        );
    }
}
