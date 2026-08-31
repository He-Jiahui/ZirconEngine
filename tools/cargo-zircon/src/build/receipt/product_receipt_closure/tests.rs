use sha2::{Digest, Sha256};

use super::super::canonical::bytes_to_hex;
use super::{
    capture_artifacts, open_artifacts, ArtifactKind, FileDigestBuffer, ReceiptArtifactSource,
};

#[test]
fn closure_artifact_capture_reuses_one_digest_buffer() {
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-closure-digest-buffer-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let fixtures = [
        ("runtime.exe", b"runtime product bytes".as_slice()),
        (
            "editor.exe",
            b"editor product bytes with another size".as_slice(),
        ),
    ];
    let mut sources = Vec::with_capacity(fixtures.len());
    for (index, (file_name, contents)) in fixtures.iter().enumerate() {
        let path = directory.join(file_name);
        std::fs::write(&path, contents).unwrap();
        sources.push(ReceiptArtifactSource {
            logical_name: format!("product-{index}"),
            relative_path: format!("products/{file_name}"),
            kind: ArtifactKind::Executable,
            source_path: path,
        });
    }

    let opened = open_artifacts(sources).unwrap();
    let mut buffer = FileDigestBuffer::new();
    let artifacts = capture_artifacts(opened, &mut buffer).unwrap();

    for (artifact, (_, contents)) in artifacts.iter().zip(fixtures) {
        assert_eq!(artifact.sha256, bytes_to_hex(&Sha256::digest(contents)));
        assert_eq!(artifact.byte_length, contents.len() as u64);
    }

    std::fs::remove_dir_all(directory).unwrap();
}
