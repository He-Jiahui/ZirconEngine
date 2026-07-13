use std::fs;
use std::path::Path;

use crate::asset::AssetUri;

pub(crate) fn assert_binary_artifact_cache(artifact_cache_root: &Path, artifact_uri: &AssetUri) {
    let artifact = artifact_uri.to_string();
    assert!(
        artifact.starts_with("lib://"),
        "artifact locator should use lib://, got {artifact}"
    );
    assert!(
        artifact.ends_with(".zasset"),
        "artifact cache entry should use .zasset binary cache, got {artifact}"
    );
    let payload = fs::read(artifact_cache_root.join(artifact_uri.path())).unwrap();
    assert!(
        payload.starts_with(b"ZRARTZ01"),
        "artifact cache entry should start with the compressed binary cache magic: {artifact}"
    );
    assert_ne!(
        payload.get(b"ZRARTZ01".len()..b"ZRARTZ01".len() + 4),
        Some(&b"JSON"[..]),
        "artifact cache entry should not carry a JSON cache marker: {artifact}"
    );
    assert_ne!(
        payload.get(b"ZRARTZ01".len()..b"ZRARTZ01".len() + 4),
        Some(&b"BIN\0"[..]),
        "artifact cache entry should not carry a legacy bincode format marker: {artifact}"
    );
    let cache = zstd::stream::decode_all(&payload[b"ZRARTZ01".len()..]).unwrap();
    assert!(
        !matches!(cache.first(), Some(b'{') | Some(b'[')),
        "decompressed artifact cache entry should be bincode payload bytes: {artifact}"
    );
}

pub(crate) fn assert_artifact_cache_files_are_zassets(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            assert_artifact_cache_files_are_zassets(&path);
        } else {
            assert_eq!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("zasset"),
                "runtime artifact cache file should use .zasset: {}",
                path.display()
            );
        }
    }
}
