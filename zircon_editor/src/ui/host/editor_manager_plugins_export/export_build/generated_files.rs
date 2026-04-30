use std::path::PathBuf;

pub(super) fn should_invoke_cargo(generated_files: &[PathBuf]) -> bool {
    generated_files
        .iter()
        .any(|path| path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml"))
}

pub(super) fn should_probe_exported_native_manifest(generated_files: &[PathBuf]) -> bool {
    generated_files
        .iter()
        .any(|path| path.file_name().and_then(|name| name.to_str()) == Some("native_plugins.toml"))
}
