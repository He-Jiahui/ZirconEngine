use std::io::Cursor;

use super::*;

#[test]
fn streaming_sha256_matches_the_in_memory_identity() {
    let bytes = vec![0x5a; STREAM_HASH_BUFFER_BYTES * 3 + 17];

    let streaming = ZrRuntimeDigestV1::sha256_reader(Cursor::new(&bytes)).unwrap();

    assert_eq!(streaming, ZrRuntimeDigestV1::sha256(&bytes));
}
