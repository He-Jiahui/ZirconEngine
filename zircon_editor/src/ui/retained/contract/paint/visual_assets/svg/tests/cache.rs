use super::{
    normalize_path, path_aliases, SvgFileStamp, SvgTreeCache, SvgTreeCacheKey,
    MAX_SVG_TREE_CACHE_ENTRIES,
};
use std::path::{Path, PathBuf};

#[test]
fn storing_a_new_svg_generation_replaces_the_old_tree_for_that_path() {
    let path = PathBuf::from("assets/icons/save.svg");
    let mut cache = SvgTreeCache::default();
    cache.insert(
        SvgTreeCacheKey {
            path: path.clone(),
            stamp: SvgFileStamp {
                modified_unix_ns: Some(1),
                len: Some(10),
            },
        },
        None,
    );

    cache.insert(
        SvgTreeCacheKey {
            path: path.clone(),
            stamp: SvgFileStamp {
                modified_unix_ns: Some(2),
                len: Some(11),
            },
        },
        None,
    );

    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.lru_order.len(), 1);
    assert_eq!(cache.entries[&path].stamp.modified_unix_ns, Some(2));
}

#[test]
fn tree_cache_evicts_the_least_recently_used_path_without_scanning_entries() {
    let mut cache = SvgTreeCache::default();
    for index in 0..MAX_SVG_TREE_CACHE_ENTRIES {
        cache.insert(
            SvgTreeCacheKey {
                path: PathBuf::from(format!("assets/icons/{index:04}.svg")),
                stamp: SvgFileStamp {
                    modified_unix_ns: Some(1),
                    len: Some(10),
                },
            },
            None,
        );
    }
    assert!(cache
        .get_by_query_path(std::path::Path::new("assets/icons/0000.svg"))
        .is_some());

    cache.insert(
        SvgTreeCacheKey {
            path: PathBuf::from("assets/icons/new.svg"),
            stamp: SvgFileStamp {
                modified_unix_ns: Some(1),
                len: Some(10),
            },
        },
        None,
    );

    assert_eq!(cache.entries.len(), MAX_SVG_TREE_CACHE_ENTRIES);
    assert!(cache
        .entries
        .contains_key(Path::new("assets/icons/0000.svg")));
    assert!(!cache
        .entries
        .contains_key(Path::new("assets/icons/0001.svg")));
    assert_eq!(cache.lru_order.len(), cache.entries.len());
}

#[test]
fn watcher_relative_path_removes_only_the_matching_svg_tree() {
    let save_path = unique_test_source("targeted-save");
    let close_path = unique_test_source("targeted-close");
    std::fs::write(&save_path, b"save-v1").expect("write save source");
    std::fs::write(&close_path, b"close-v1").expect("write close source");
    let mut cache = SvgTreeCache::default();
    for path in [&save_path, &close_path] {
        cache.insert(
            SvgTreeCacheKey {
                path: path.clone(),
                stamp: SvgFileStamp {
                    modified_unix_ns: Some(1),
                    len: Some(10),
                },
            },
            None,
        );
    }
    std::fs::write(&save_path, b"save-v2").expect("change save source");

    assert_eq!(
        cache.invalidate_paths(&[save_path.to_string_lossy().into_owned()]),
        1
    );
    assert!(!cache.entries.contains_key(&save_path));
    assert!(cache.entries.contains_key(&close_path));
    let _ = std::fs::remove_file(save_path);
    let _ = std::fs::remove_file(close_path);
}

#[test]
fn unchanged_watcher_event_preserves_the_parsed_tree_entry() {
    let path = unique_test_source("unchanged");
    std::fs::write(&path, b"same-svg-bytes").expect("write source");
    let mut cache = SvgTreeCache::default();
    cache.insert(
        SvgTreeCacheKey {
            path: path.clone(),
            stamp: SvgFileStamp {
                modified_unix_ns: Some(1),
                len: Some(14),
            },
        },
        None,
    );

    assert_eq!(
        cache.invalidate_paths(&[path.to_string_lossy().into_owned()]),
        0
    );
    assert!(cache.entries.contains_key(&path));
    let _ = std::fs::remove_file(path);
}

#[test]
fn lag_reconciliation_invalidates_only_svg_sources_whose_content_changed() {
    let changed_path = unique_test_source("lag-changed");
    let stable_path = unique_test_source("lag-stable");
    std::fs::write(&changed_path, b"changed-v1").expect("write changed source");
    std::fs::write(&stable_path, b"stable-v1").expect("write stable source");
    let mut cache = SvgTreeCache::default();
    for path in [&changed_path, &stable_path] {
        cache.insert(
            SvgTreeCacheKey {
                path: path.clone(),
                stamp: SvgFileStamp {
                    modified_unix_ns: Some(1),
                    len: Some(9),
                },
            },
            None,
        );
    }
    std::fs::write(&changed_path, b"changed-v2").expect("change source");

    assert_eq!(cache.reconcile_source_fingerprints(), 1);
    assert!(!cache.entries.contains_key(&changed_path));
    assert!(cache.entries.contains_key(&stable_path));
    let _ = std::fs::remove_file(changed_path);
    let _ = std::fs::remove_file(stable_path);
}

#[test]
fn path_aliases_match_absolute_and_resource_relative_locator_forms() {
    let aliases = path_aliases("E:/project/assets/icons/save.svg");

    assert!(aliases.contains("e:/project/assets/icons/save.svg"));
    assert!(aliases.contains("assets/icons/save.svg"));
    assert!(aliases.contains("icons/save.svg"));
    assert!(!aliases.contains("save.svg"));
    assert!(path_aliases("res://icons/save.svg").contains("icons/save.svg"));
}

#[test]
fn stable_relative_queries_hit_the_memory_index_without_file_metadata() {
    let mut cache = SvgTreeCache::default();
    cache.insert(
        SvgTreeCacheKey {
            path: PathBuf::from("E:/project/assets/icons/save.svg"),
            stamp: SvgFileStamp {
                modified_unix_ns: Some(1),
                len: Some(10),
            },
        },
        None,
    );

    assert!(cache
        .get_by_query_path(std::path::Path::new("assets/icons/save.svg"))
        .is_some());
}

#[test]
fn windows_verbatim_canonical_paths_match_regular_absolute_queries() {
    let canonical = r"\\?\E:\project\assets\icons\save.svg";
    let regular = r"E:\project\assets\icons\save.svg";
    assert_eq!(normalize_path(canonical), normalize_path(regular));

    let mut cache = SvgTreeCache::default();
    cache.insert(
        SvgTreeCacheKey {
            path: PathBuf::from(canonical),
            stamp: SvgFileStamp {
                modified_unix_ns: Some(1),
                len: Some(10),
            },
        },
        None,
    );

    assert!(cache
        .get_by_query_path(std::path::Path::new(regular))
        .is_some());
}

#[test]
fn ambiguous_relative_queries_fall_back_to_the_stamped_path() {
    let mut cache = SvgTreeCache::default();
    for root in ["E:/first", "F:/second"] {
        cache.insert(
            SvgTreeCacheKey {
                path: PathBuf::from(format!("{root}/assets/icons/save.svg")),
                stamp: SvgFileStamp {
                    modified_unix_ns: Some(1),
                    len: Some(10),
                },
            },
            None,
        );
    }

    assert!(cache
        .get_by_query_path(std::path::Path::new("assets/icons/save.svg"))
        .is_none());
    assert!(cache
        .get_by_query_path(std::path::Path::new("E:/first/assets/icons/save.svg"))
        .is_some());
}

fn unique_test_source(label: &str) -> PathBuf {
    static NEXT_TEST_SOURCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let sequence = NEXT_TEST_SOURCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zircon-svg-tree-cache-{label}-{}-{sequence}.svg",
        std::process::id()
    ))
}
