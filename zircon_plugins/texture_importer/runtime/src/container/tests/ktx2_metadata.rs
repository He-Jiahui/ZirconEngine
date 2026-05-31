use super::common::*;
use crate::container::support::{
    KTX2_HEADER_SIZE, KTX2_SUPERCOMPRESSION_BASIS_LZ, KTX2_SUPERCOMPRESSION_ZSTANDARD,
};
use zircon_runtime::asset::ImportedAsset;

const DEFAULT_KVD_OFFSET: u32 = KTX2_AFTER_DEFAULT_DFD_OFFSET as u32;
const DEFAULT_SGD_OFFSET: u64 = KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64;

#[test]
fn ktx2_container_importer_rejects_truncated_key_value_data_range() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 56, DEFAULT_KVD_OFFSET);
    write_u32(&mut bytes, 60, 16);

    let error = import_container_error("truncated-kvd.ktx2", bytes);

    assert!(
        error.contains("ktx2 key/value data requires at least 220 bytes, got 204"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_reads_key_value_data_record() {
    let mut bytes = tiny_ktx2_bytes();
    let metadata = ktx2_key_value_data_record(b"KTXorientation\0rd");
    write_u32(&mut bytes, 56, DEFAULT_KVD_OFFSET);
    write_u32(
        &mut bytes,
        60,
        u32::try_from(metadata.len()).expect("metadata length fits u32"),
    );
    bytes.extend_from_slice(&metadata);

    let imported = import_container_fixture("metadata.ktx2", bytes);

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 16);
            assert_eq!(texture.render_image_descriptor().array_layer_count, 12);
            assert_eq!(texture.height, 16);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx2_container_importer_reads_multiple_key_value_data_records() {
    let mut metadata = ktx2_key_value_data_record(b"KTXorientation\0rd");
    metadata.extend_from_slice(&ktx2_key_value_data_record(b"ZirconNote\0debug"));

    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 56, DEFAULT_KVD_OFFSET);
    write_u32(
        &mut bytes,
        60,
        u32::try_from(metadata.len()).expect("metadata length fits u32"),
    );
    bytes.extend_from_slice(&metadata);

    let imported = import_container_fixture("multi-metadata.ktx2", bytes);

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 16);
            assert_eq!(texture.render_image_descriptor().mip_count, 4);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx2_container_importer_rejects_invalid_key_value_data_records() {
    let cases: [(&str, Vec<u8>, &str); 10] = [
        (
            "short-kvd-record-length.ktx2",
            vec![1, 0, 0],
            "ktx2 key/value data record 0 keyAndValueByteLength extends past declared ktx2 key/value data length",
        ),
        (
            "empty-kvd-record.ktx2",
            0_u32.to_le_bytes().to_vec(),
            "ktx2 key/value data record 0 keyAndValueByteLength must be at least 2 bytes",
        ),
        (
            "one-byte-kvd-record.ktx2",
            {
                let mut metadata = 1_u32.to_le_bytes().to_vec();
                metadata.push(b'k');
                metadata
            },
            "ktx2 key/value data record 0 keyAndValueByteLength must be at least 2 bytes",
        ),
        (
            "truncated-kvd-record-payload.ktx2",
            {
                let mut metadata = 8_u32.to_le_bytes().to_vec();
                metadata.extend_from_slice(b"key\0");
                metadata
            },
            "ktx2 key/value data record 0 payload extends past declared ktx2 key/value data length",
        ),
        (
            "unterminated-kvd-record-key.ktx2",
            ktx2_key_value_data_record(b"key-value"),
            "ktx2 key/value data record 0 key must be NUL terminated",
        ),
        (
            "empty-kvd-record-key.ktx2",
            ktx2_key_value_data_record(b"\0value"),
            "ktx2 key/value data record 0 key must be non-empty",
        ),
        (
            "truncated-kvd-record-padding.ktx2",
            {
                let mut metadata = 5_u32.to_le_bytes().to_vec();
                metadata.extend_from_slice(b"key\0v");
                metadata
            },
            "ktx2 key/value data record 0 valuePadding extends past declared ktx2 key/value data length",
        ),
        (
            "bom-kvd-record-key.ktx2",
            ktx2_key_value_data_record(&[0xef, 0xbb, 0xbf, b'k', 0, b'v']),
            "ktx2 key/value data record 0 key must be UTF-8 without BOM",
        ),
        (
            "invalid-utf8-kvd-record-key.ktx2",
            ktx2_key_value_data_record(&[0xff, 0, b'v']),
            "ktx2 key/value data record 0 key must be UTF-8 without BOM",
        ),
        (
            "nonzero-kvd-record-padding.ktx2",
            {
                let mut metadata = ktx2_key_value_data_record(b"key\0v");
                let last = metadata.len() - 1;
                metadata[last] = 1;
                metadata
            },
            "ktx2 key/value data record 0 valuePadding bytes must be zero",
        ),
    ];

    for (path, metadata, expected) in cases {
        let mut bytes = tiny_ktx2_bytes();
        write_u32(&mut bytes, 56, DEFAULT_KVD_OFFSET);
        write_u32(
            &mut bytes,
            60,
            u32::try_from(metadata.len()).expect("metadata length fits u32"),
        );
        bytes.extend_from_slice(&metadata);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn ktx2_container_importer_rejects_invalid_later_key_value_data_record() {
    let mut metadata = ktx2_key_value_data_record(b"KTXorientation\0rd");
    metadata.extend_from_slice(&ktx2_key_value_data_record(b"broken-key"));

    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 56, DEFAULT_KVD_OFFSET);
    write_u32(
        &mut bytes,
        60,
        u32::try_from(metadata.len()).expect("metadata length fits u32"),
    );
    bytes.extend_from_slice(&metadata);

    let error = import_container_error("invalid-second-kvd-record.ktx2", bytes);

    assert!(
        error.contains("ktx2 key/value data record 1 key must be NUL terminated"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_zero_length_metadata_with_nonzero_offsets() {
    let cases: [(&str, fn(&mut Vec<u8>), &str); 2] = [
        (
            "empty-kvd-nonzero-offset.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 56, DEFAULT_KVD_OFFSET);
            },
            "ktx2 key/value data offset must be 0 when length is 0",
        ),
        (
            "empty-sgd-nonzero-offset.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u64(bytes, 64, DEFAULT_SGD_OFFSET);
            },
            "ktx2 supercompression global data offset must be 0 when length is 0",
        ),
    ];

    for (path, mutate, expected) in cases {
        let mut bytes = tiny_ktx2_bytes();
        mutate(&mut bytes);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn ktx2_container_importer_rejects_metadata_ranges_inside_level_index() {
    let cases: [(&str, fn(&mut Vec<u8>), &str); 3] = [
        (
            "dfd-inside-index.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 48, KTX2_HEADER_SIZE as u32);
                write_u32(bytes, 52, KTX2_DEFAULT_DFD_BYTE_LENGTH as u32);
            },
            "ktx2 data format descriptor starts inside header or level index",
        ),
        (
            "kvd-inside-index.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u32(bytes, 56, KTX2_HEADER_SIZE as u32);
                write_u32(bytes, 60, 4);
            },
            "ktx2 key/value data starts inside header or level index",
        ),
        (
            "sgd-inside-index.ktx2",
            |bytes: &mut Vec<u8>| {
                write_u64(bytes, 64, KTX2_HEADER_SIZE as u64);
                write_u64(bytes, 72, 4);
            },
            "ktx2 supercompression global data starts inside header or level index",
        ),
    ];

    for (path, mutate, expected) in cases {
        let mut bytes = tiny_ktx2_bytes();
        mutate(&mut bytes);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn ktx2_container_importer_rejects_overlapping_metadata_ranges() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 56, (KTX2_DEFAULT_DFD_OFFSET + 4) as u32);
    write_u32(&mut bytes, 60, 8);

    let error = import_container_error("overlapping-metadata.ktx2", bytes);

    assert!(
        error.contains("ktx2 key/value data overlaps ktx2 data format descriptor"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_accepts_key_value_data_after_data_format_descriptor_alignment() {
    let mut bytes = tiny_ktx2_bytes();
    let metadata = ktx2_key_value_data_record(b"KTXorientation\0rd");
    write_u32(&mut bytes, 56, DEFAULT_KVD_OFFSET);
    write_u32(
        &mut bytes,
        60,
        u32::try_from(metadata.len()).expect("metadata length fits u32"),
    );
    bytes.extend_from_slice(&metadata);

    let imported = import_container_fixture("dfd-aligned-kvd.ktx2", bytes);

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 16);
            assert_eq!(texture.render_image_descriptor().array_layer_count, 12);
            assert_eq!(texture.height, 16);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ktx2_container_importer_rejects_misaligned_key_value_data_offset() {
    let mut bytes = tiny_ktx2_bytes();
    let metadata = ktx2_key_value_data_record(b"KTXorientation\0rd");
    write_u32(&mut bytes, 56, DEFAULT_KVD_OFFSET + 4);
    write_u32(
        &mut bytes,
        60,
        u32::try_from(metadata.len()).expect("metadata length fits u32"),
    );
    bytes.resize(KTX2_AFTER_DEFAULT_DFD_OFFSET + 4, 0);
    bytes.extend_from_slice(&metadata);

    let error = import_container_error("misaligned-kvd.ktx2", bytes);

    assert!(
        error.contains(
            "ktx2 key/value data offset must be 204 after data format descriptor alignment"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_metadata_range_overlapping_level_payload() {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 44, KTX2_SUPERCOMPRESSION_ZSTANDARD);
    write_u64(
        &mut bytes,
        KTX2_HEADER_SIZE,
        KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u64,
    );
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 16);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 16, 24);
    write_u32(&mut bytes, 48, KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET as u32);
    write_u32(&mut bytes, 52, KTX2_DEFAULT_DFD_BYTE_LENGTH as u32);
    bytes.resize(
        KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET + KTX2_DEFAULT_DFD_BYTE_LENGTH,
        0,
    );
    write_minimal_ktx2_dfd(&mut bytes, KTX2_AFTER_DEFAULT_DFD_8_BYTE_OFFSET);

    let error = import_container_error("metadata-overlaps-level.ktx2", bytes);

    assert!(
        error.contains("ktx2 data format descriptor overlaps ktx2 level 0 payload"),
        "unexpected error: {error}"
    );
}

#[test]
fn ktx2_container_importer_rejects_misaligned_supercompression_global_data_offset() {
    let mut bytes = tiny_ktx2_bytes();
    let metadata = ktx2_key_value_data_record(b"KTXorientation\0rd");
    let key_value_data_end = KTX2_AFTER_DEFAULT_DFD_OFFSET + metadata.len();
    let aligned_sgd_offset = key_value_data_end.next_multiple_of(8);
    write_u32(&mut bytes, 12, 0);
    write_u32(&mut bytes, 44, KTX2_SUPERCOMPRESSION_BASIS_LZ);
    write_u32(&mut bytes, 56, DEFAULT_KVD_OFFSET);
    write_u32(
        &mut bytes,
        60,
        u32::try_from(metadata.len()).expect("metadata length fits u32"),
    );
    write_u64(
        &mut bytes,
        64,
        u64::try_from(aligned_sgd_offset + 8).expect("offset fits u64"),
    );
    write_u64(&mut bytes, 72, 8);
    bytes.extend_from_slice(&metadata);
    bytes.resize(aligned_sgd_offset + 16, 0);

    let error = import_container_error("misaligned-sgd.ktx2", bytes);

    assert!(
        error.contains(&format!(
            "ktx2 supercompression global data offset must be {aligned_sgd_offset} after key/value data alignment"
        )),
        "unexpected error: {error}"
    );
}

fn ktx2_key_value_data_record(key_and_value: &[u8]) -> Vec<u8> {
    let key_and_value_len =
        u32::try_from(key_and_value.len()).expect("metadata record length fits u32");
    let mut metadata = Vec::new();
    metadata.extend_from_slice(&key_and_value_len.to_le_bytes());
    metadata.extend_from_slice(key_and_value);
    metadata.resize(metadata.len() + ktx2_padding(key_and_value.len()), 0);
    metadata
}

fn ktx2_padding(byte_len: usize) -> usize {
    (4 - (byte_len % 4)) % 4
}
