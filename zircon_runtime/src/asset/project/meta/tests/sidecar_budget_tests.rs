use std::cell::Cell;
use std::fs;
use std::io::{Cursor, Read};

use super::*;

struct CountingReader<'a> {
    source: Cursor<Vec<u8>>,
    bytes_read: &'a Cell<usize>,
}

impl Read for CountingReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = self.source.read(buffer)?;
        self.bytes_read.set(self.bytes_read.get() + count);
        Ok(count)
    }
}

#[test]
fn asset_meta_reader_accepts_exact_budget_and_stops_growth_at_limit_plus_one() {
    let exact = read_bounded_meta_text_from_reader(Cursor::new(vec![b' '; 16]), 16, 16).unwrap();
    assert_eq!(exact.len(), 16);

    let bytes_read = Cell::new(0);
    let error = read_bounded_meta_text_from_reader(
        CountingReader {
            source: Cursor::new(vec![b' '; 64]),
            bytes_read: &bytes_read,
        },
        16,
        16,
    )
    .expect_err("a sidecar that grows past admission must fail");
    assert_document_too_large(&error, 16, 17);
    assert_eq!(bytes_read.get(), 17);
}

#[test]
fn asset_meta_reader_rejects_sparse_oversize_before_allocating_or_parsing() {
    let root = std::env::temp_dir().join(format!(
        "zircon-meta-budget-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("oversize.zmeta");
    fs::File::create(&path)
        .unwrap()
        .set_len(MAX_ASSET_META_DOCUMENT_BYTES as u64 + 1)
        .unwrap();

    let error = AssetMetaDocument::load(&path).expect_err("oversize sidecar must not be parsed");
    assert_document_too_large(
        &error,
        MAX_ASSET_META_DOCUMENT_BYTES,
        MAX_ASSET_META_DOCUMENT_BYTES as u64 + 1,
    );
    let locator = AssetUri::parse("res://oversize").unwrap();
    let identity_error =
        crate::asset::reference_resolver::persisted_source_path_for_locator(&root, &locator)
            .expect_err("the persisted-source identity reread must share the same bound");
    assert_document_too_large(
        &identity_error,
        MAX_ASSET_META_DOCUMENT_BYTES,
        MAX_ASSET_META_DOCUMENT_BYTES as u64 + 1,
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn asset_meta_parser_and_writer_use_the_same_document_budget() {
    let document = AssetMetaDocument::new(
        AssetUuid::new(),
        AssetUri::parse("res://budget.json").unwrap(),
        AssetKind::Data,
    );
    let serialized = document.to_pretty_bytes().unwrap();
    assert_eq!(
        AssetMetaDocument::from_toml_str(std::str::from_utf8(&serialized).unwrap()).unwrap(),
        document
    );

    let error = document
        .to_pretty_bytes_with_limit(serialized.len() - 1)
        .expect_err("save must reject an over-budget serialized sidecar");
    assert_document_too_large(&error, serialized.len() - 1, serialized.len() as u64);

    let invalid = read_bounded_meta_text_from_reader(Cursor::new(vec![0xff]), 1, 1)
        .expect_err("invalid UTF-8 within budget must remain an invalid-data error");
    assert_eq!(invalid.kind(), std::io::ErrorKind::InvalidData);
}

fn assert_document_too_large(error: &std::io::Error, max: usize, found: u64) {
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(matches!(
        error.get_ref().and_then(|source| source.downcast_ref::<AssetMetaError>()),
        Some(AssetMetaError::DocumentTooLarge { max: actual_max, found: actual_found })
            if *actual_max == max && *actual_found == found
    ));
}
