use std::error::Error as _;

use super::*;

#[test]
fn path_io_preserves_the_operating_system_error_source() {
    let error = ReferenceResolutionError::PathIo {
        path: PathBuf::from("assets/blocked.glb"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "blocked"),
    };
    let source = error.source().expect("path io keeps source chain");
    assert!(source.to_string().contains("blocked"));
}
