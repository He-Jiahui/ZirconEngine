use std::fs;

use super::*;

#[test]
fn failed_compound_member_capture_keeps_primary_bytes_and_discards_partial_members() {
    let root = std::env::temp_dir().join(format!(
        "zircon-compound-member-failure-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let compound_root = root.join("package");
    fs::create_dir_all(&compound_root).unwrap();
    let meta_path = root.join("package.zmeta");
    let first_member = compound_root.join("first.zshader");
    let missing_member = compound_root.join("missing.zshader");
    let original_primary = b"admitted primary descriptor";
    fs::write(&meta_path, original_primary).unwrap();
    fs::write(&first_member, b"admitted first member").unwrap();

    let mut source = AssetImportSource {
        asset_root: root.clone(),
        path: meta_path.clone(),
        uri: AssetUri::parse("res://package").unwrap(),
        meta_path: meta_path.clone(),
        unit: AssetSourceUnit::Compound,
        included_files: Vec::new(),
        included_paths: vec![first_member, missing_member.clone()],
        compound_root: Some(compound_root),
        source_snapshot: None,
        snapshot_complete: false,
    };

    let error = materialize_compound_source_snapshot(&mut source)
        .expect_err("a missing second member must fail after the primary is captured");
    assert!(error.to_string().contains("missing.zshader"), "{error}");
    assert!(
        source.source_file_snapshots().is_empty(),
        "the first member must not form a partial generation snapshot"
    );

    fs::write(&missing_member, b"late second member").unwrap();
    let retry_error = materialize_compound_source_snapshot(&mut source)
        .expect_err("a retained incomplete snapshot must not be treated as admitted");
    assert!(
        retry_error.to_string().contains("incomplete"),
        "{retry_error}"
    );
    assert!(source.source_file_snapshots().is_empty());

    fs::write(&meta_path, b"replacement primary descriptor").unwrap();
    assert_eq!(
        take_source_bytes_for_import(&mut source).unwrap(),
        original_primary,
        "failed member admission must not reopen the replaced primary path"
    );
    assert!(error.to_string().contains("missing.zshader"), "{error}");

    fs::remove_dir_all(root).unwrap();
}
