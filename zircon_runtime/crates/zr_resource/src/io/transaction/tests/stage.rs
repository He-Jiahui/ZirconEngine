use std::io;

use super::{classify_file_metadata, FilePresence};

#[test]
fn file_presence_treats_only_not_found_as_missing() {
    let missing = classify_file_metadata(Err(io::Error::from(io::ErrorKind::NotFound)))
        .expect("NotFound is the only missing-file classification");
    assert_eq!(missing, FilePresence::Missing);

    let error = classify_file_metadata(Err(io::Error::from(io::ErrorKind::PermissionDenied)))
        .expect_err("metadata access failures must not become missing-file evidence");
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
}
