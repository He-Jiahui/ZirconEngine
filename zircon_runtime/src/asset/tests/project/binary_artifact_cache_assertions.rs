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
        payload.starts_with(b"ZRARTM06"),
        "artifact cache entry should start with the versioned artifact manifest magic: {artifact}"
    );
    assert!(
        artifact_cache_root.join("chunks").is_dir(),
        "artifact cache should publish content-addressed chunks: {artifact}"
    );
}

pub(crate) fn assert_artifact_cache_files_are_zassets(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) == Some("chunks") {
                assert_content_addressed_chunk_files(&path);
            } else {
                assert_artifact_cache_files_are_zassets(&path);
            }
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

fn assert_content_addressed_chunk_files(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        assert_eq!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("zchunk"),
            "content-addressed artifact chunk should use .zchunk: {}",
            path.display()
        );
    }
}
