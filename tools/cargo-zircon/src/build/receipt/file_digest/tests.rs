use std::fs::File;

use sha2::{Digest, Sha256};

use super::{digest_open_file_handle_bytes_with_buffer, FileDigestBuffer};

#[test]
fn shared_digest_buffer_hashes_multiple_open_files() {
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-shared-digest-buffer-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let mut buffer = FileDigestBuffer::new();

    for (file_name, contents) in [
        ("first.bin", b"first product artifact".as_slice()),
        (
            "second.bin",
            b"a differently sized second product artifact".as_slice(),
        ),
    ] {
        let path = directory.join(file_name);
        std::fs::write(&path, contents).unwrap();
        let mut file = File::open(&path).unwrap();

        let digest = digest_open_file_handle_bytes_with_buffer(&mut file, &mut buffer).unwrap();
        let expected_sha256: [u8; 32] = Sha256::digest(contents).into();

        assert_eq!(digest.sha256, expected_sha256);
        assert_eq!(digest.byte_length, contents.len() as u64);
    }

    std::fs::remove_dir_all(directory).unwrap();
}
