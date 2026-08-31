use std::fs::File;

use sha2::{Digest, Sha256};

use super::super::canonical::bytes_to_hex;
use super::ToolchainSet;

#[test]
fn every_component_change_derives_a_distinct_toolchain_set_identity() {
    let baseline = toolchain('A', 'B', 'C', 'D', 'E');
    let changed = [
        toolchain('F', 'B', 'C', 'D', 'E'),
        toolchain('A', 'F', 'C', 'D', 'E'),
        toolchain('A', 'B', 'F', 'D', 'E'),
        toolchain('A', 'B', 'C', 'F', 'E'),
        toolchain('A', 'B', 'C', 'D', 'F'),
    ];

    assert!(changed
        .iter()
        .all(|candidate| candidate.toolchain_set_id != baseline.toolchain_set_id));
}

#[test]
fn component_mutation_rejects_a_stale_toolchain_set_identity() {
    let mut toolchain = toolchain('A', 'B', 'C', 'D', 'E');
    let stale_identity = toolchain.toolchain_set_id.clone();
    toolchain.sdk_fingerprint = digest('F');

    let error = toolchain.normalize_and_verify_identity().unwrap_err();

    assert_eq!(toolchain.toolchain_set_id, stale_identity);
    assert!(error
        .to_string()
        .contains("ToolchainSet identity does not match its declared components"));
}

#[test]
fn capture_from_files_reuses_buffer_across_tool_binaries() {
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-toolchain-digest-buffer-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let tools = [
        ("cargo.exe", b"cargo tool bytes".as_slice()),
        (
            "rustc.exe",
            b"rustc tool bytes with a different size".as_slice(),
        ),
        ("link.exe", b"linker tool bytes".as_slice()),
    ];
    for (file_name, contents) in tools {
        std::fs::write(directory.join(file_name), contents).unwrap();
    }

    let captured = ToolchainSet::capture_from_files(
        File::open(directory.join("cargo.exe")).unwrap(),
        File::open(directory.join("rustc.exe")).unwrap(),
        Some(File::open(directory.join("link.exe")).unwrap()),
        digest('D'),
        digest('E'),
    )
    .unwrap();

    assert_eq!(
        captured.cargo_sha256,
        bytes_to_hex(&Sha256::digest(tools[0].1))
    );
    assert_eq!(
        captured.rustc_sha256,
        bytes_to_hex(&Sha256::digest(tools[1].1))
    );
    assert_eq!(
        captured.linker_sha256.as_deref(),
        Some(bytes_to_hex(&Sha256::digest(tools[2].1)).as_str())
    );

    std::fs::remove_dir_all(directory).unwrap();
}

fn toolchain(cargo: char, rustc: char, linker: char, sdk: char, environment: char) -> ToolchainSet {
    ToolchainSet::new(
        digest(cargo),
        digest(rustc),
        Some(digest(linker)),
        digest(sdk),
        digest(environment),
    )
    .unwrap()
}

fn digest(value: char) -> String {
    value.to_string().repeat(64)
}
