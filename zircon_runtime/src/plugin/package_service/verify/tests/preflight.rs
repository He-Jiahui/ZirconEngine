use super::*;

fn archive(manifest: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0; 24];
    bytes[..4].copy_from_slice(b"ZRPK");
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes[8..16].copy_from_slice(&24u64.to_le_bytes());
    bytes[16..24].copy_from_slice(&(manifest.len() as u64).to_le_bytes());
    bytes.extend_from_slice(manifest);
    bytes
}

#[test]
fn package_service_preflight_bounds_both_collections_before_full_decode() {
    let mut manifest = serde_json::json!({
        "pack": {"chunks": vec![serde_json::Value::Null; MAX_FILES]},
        "assets": vec![serde_json::Value::Null; MAX_FILES],
    });
    assert!(admit(&archive(&serde_json::to_vec(&manifest).unwrap())).is_ok());
    for path in ["/pack/chunks", "/assets"] {
        let entries = manifest.pointer_mut(path).unwrap().as_array_mut().unwrap();
        entries.push(serde_json::Value::Null);
        assert!(matches!(
            admit(&archive(&serde_json::to_vec(&manifest).unwrap())),
            Err(PackageError::Capacity)
        ));
        manifest
            .pointer_mut(path)
            .unwrap()
            .as_array_mut()
            .unwrap()
            .pop();
    }
}

#[test]
fn package_service_preflight_bounds_manifest_bytes_and_exact_extent() {
    let mut manifest = br#"{"pack":{"chunks":[]},"assets":[]}"#.to_vec();
    manifest.resize(MAX_PACK_MANIFEST_BYTES, b' ');
    assert!(admit(&archive(&manifest)).is_ok());
    manifest.push(b' ');
    assert!(matches!(
        admit(&archive(&manifest)),
        Err(PackageError::Capacity)
    ));
    let mut trailing = archive(br#"{"pack":{"chunks":[]},"assets":[]}"#);
    trailing.push(b' ');
    assert!(matches!(admit(&trailing), Err(PackageError::Invalid)));
    trailing[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(admit(&trailing).is_err());
}
