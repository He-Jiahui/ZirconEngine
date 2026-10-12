use std::error::Error as _;
use std::io;
use std::path::PathBuf;

use super::*;

#[test]
fn cargo_error_preserves_process_and_io_sources() {
    let error = EditorExportBuildError::cargo(ExportProcessError::io(
        "failed to invoke Cargo",
        "typed cargo test",
        None,
        Some(PathBuf::from("Cargo.toml")),
        io::Error::new(io::ErrorKind::PermissionDenied, "cargo source"),
    ));

    let process = error
        .source()
        .and_then(|source| source.downcast_ref::<ExportProcessError>())
        .expect("Cargo error must retain its process error");
    let source = process
        .source()
        .and_then(|source| source.downcast_ref::<io::Error>())
        .expect("process error must retain its IO error");
    assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
}

#[test]
fn materialization_error_preserves_io_source() {
    let error = EditorExportBuildError::materialize(io::Error::new(
        io::ErrorKind::WriteZero,
        "materialize source",
    ));

    let source = error
        .source()
        .and_then(|source| source.downcast_ref::<io::Error>())
        .expect("materialization error must retain its IO error");
    assert_eq!(source.kind(), io::ErrorKind::WriteZero);
}
