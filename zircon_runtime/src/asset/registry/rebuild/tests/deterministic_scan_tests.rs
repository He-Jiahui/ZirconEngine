use std::collections::HashSet;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{collect_meta_entries, collect_meta_paths_for_root};

#[test]
fn metadata_scan_collects_siblings_in_lexical_depth_first_order() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_registry_deterministic_scan_{}_{}",
        std::process::id(),
        nonce
    ));
    fs::create_dir_all(root.join("b-dir")).unwrap();
    fs::create_dir_all(root.join("a-dir")).unwrap();

    for relative in [
        "z-root.zmeta",
        "b-dir/z-last.zmeta",
        "a-dir/z-last.zmeta",
        "c-root.zmeta",
        "a-dir/a-first.zmeta",
        "b-dir/a-first.zmeta",
        "ignored.txt",
    ] {
        fs::write(root.join(relative), b"metadata fixture").unwrap();
    }

    let mut paths = Vec::new();
    collect_meta_paths_for_root(&root, &mut paths).unwrap();

    let expected = [
        "a-dir/a-first.zmeta",
        "a-dir/z-last.zmeta",
        "b-dir/a-first.zmeta",
        "b-dir/z-last.zmeta",
        "c-root.zmeta",
        "z-root.zmeta",
    ]
    .map(|relative| root.join(relative))
    .to_vec();
    assert_eq!(paths, expected);

    // Supply a known reverse order: some filesystems enumerate sorted even when creation was not.
    let unordered_entries = [
        "z-root.zmeta",
        "ignored.txt",
        "c-root.zmeta",
        "b-dir",
        "a-dir",
    ]
    .map(|relative| root.join(relative))
    .to_vec();
    let canonical_root = fs::canonicalize(&root).unwrap();
    let mut visited = HashSet::from([canonical_root.clone()]);
    let mut injected_paths = Vec::new();
    collect_meta_entries(
        unordered_entries,
        &canonical_root,
        &mut visited,
        &mut injected_paths,
    )
    .unwrap();
    assert_eq!(injected_paths, expected);

    fs::remove_dir_all(root).unwrap();
}
