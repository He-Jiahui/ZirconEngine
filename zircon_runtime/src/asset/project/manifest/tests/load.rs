use std::cell::Cell;
use std::io::{Cursor, Read};

use super::*;

struct CountingReader<'a> {
    source: Cursor<Vec<u8>>,
    bytes_read: &'a Cell<usize>,
}

impl Read for CountingReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.source.read(buffer)?;
        self.bytes_read.set(self.bytes_read.get() + read);
        Ok(read)
    }
}

#[test]
fn project_manifest_bounded_file_read_caps_growth_at_limit_plus_one() {
    let bytes_read = Cell::new(0);
    let reader = CountingReader {
        source: Cursor::new(vec![b' '; MAX_PROJECT_MANIFEST_BYTES + 64]),
        bytes_read: &bytes_read,
    };

    let error = read_bounded_manifest_from_reader(reader, MAX_PROJECT_MANIFEST_BYTES)
        .expect_err("a manifest that grows beyond its hint must be rejected");

    assert!(matches!(
        error,
        ProjectManifestError::Summary(ProjectManifestSummaryError::DocumentTooLarge {
            max: MAX_PROJECT_MANIFEST_BYTES,
            found,
        }) if found == MAX_PROJECT_MANIFEST_BYTES + 1
    ));
    assert_eq!(bytes_read.get(), MAX_PROJECT_MANIFEST_BYTES + 1);
}

#[test]
fn project_manifest_bounded_file_read_accepts_exact_limit_and_preserves_utf8_error() {
    let exact = read_bounded_manifest_from_reader(
        Cursor::new(vec![b' '; MAX_PROJECT_MANIFEST_BYTES]),
        MAX_PROJECT_MANIFEST_BYTES + 1,
    )
    .expect("the exact byte limit must remain readable");
    assert_eq!(exact.len(), MAX_PROJECT_MANIFEST_BYTES);

    let invalid = read_bounded_manifest_from_reader(Cursor::new(vec![0xff]), 1)
        .expect_err("invalid UTF-8 must retain its typed error");
    assert!(matches!(
        invalid,
        ProjectManifestError::Summary(ProjectManifestSummaryError::InvalidUtf8 { .. })
    ));
}

#[test]
fn project_manifest_string_entry_rejects_document_over_byte_budget_before_parse() {
    let oversized = "\u{00e9}".repeat(MAX_PROJECT_MANIFEST_BYTES / 2 + 1);

    let error = ProjectManifest::from_toml_str(&oversized)
        .expect_err("the in-memory parser must enforce the shared byte budget");

    assert!(matches!(
        error,
        ProjectManifestError::Summary(ProjectManifestSummaryError::DocumentTooLarge {
            max: MAX_PROJECT_MANIFEST_BYTES,
            found,
        }) if found == MAX_PROJECT_MANIFEST_BYTES + 2
    ));
}
