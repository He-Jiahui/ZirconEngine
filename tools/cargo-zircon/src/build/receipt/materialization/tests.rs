use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    inventory_relative_path, materialization_path_into, open_locked_file, reject_reparse_metadata,
};

#[test]
fn inventory_relative_path_appends_one_entry_to_the_carried_directory() {
    assert_eq!(
        inventory_relative_path("", OsStr::new("zircon_runtime.dll")).unwrap(),
        "zircon_runtime.dll"
    );
    assert_eq!(
        inventory_relative_path("editor/bin", OsStr::new("zircon_editor.exe")).unwrap(),
        "editor/bin/zircon_editor.exe"
    );
}

#[test]
fn inventory_relative_path_rejects_non_unicode_entry_names() {
    let invalid_name = OsString::from_wide(&[0xD800]);

    let error = inventory_relative_path("runtime", &invalid_name)
        .err()
        .unwrap();

    assert!(error.to_string().contains("path is not Unicode"));
}

#[test]
fn materialization_path_reuses_capacity_across_artifacts() {
    let root = Path::new(r"C:\artifact-root");
    let mut path = PathBuf::new();
    materialization_path_into(&mut path, root, "products/generated/long/artifact-name.dll");
    let retained_capacity = path.capacity();

    materialization_path_into(&mut path, root, "bin/app.exe");

    assert_eq!(path, root.join("bin/app.exe"));
    assert_eq!(path.capacity(), retained_capacity);
}

#[test]
fn opened_artifact_handle_supplies_reparse_validation_metadata() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "cargo-zircon-materialization-metadata-{}-{nonce}.bin",
        std::process::id()
    ));
    fs::write(&path, b"receipt-artifact").unwrap();

    let file = open_locked_file(&path, "fixture artifact").unwrap();
    let metadata = file.metadata().unwrap();

    assert!(metadata.is_file());
    reject_reparse_metadata(&metadata, "fixture artifact").unwrap();

    drop(file);
    fs::remove_file(path).unwrap();
}
