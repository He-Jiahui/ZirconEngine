use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::write_if_changed;

#[test]
fn generated_file_writer_reuses_parent_directory_checks() {
    let source = include_str!("../generated.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("generated-file production source");

    assert!(production.contains("created_parents.insert"));
    assert!(production.contains("Vec::with_capacity(plan.generated_files.len())"));
    assert!(production.contains("HashSet::with_capacity(plan.generated_files.len())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cw_generated_parent_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const FILES_PER_BATCH: usize = 32;
    const MARKER: &str = "RUNTIME510_GENERATED_PARENT_CAPACITY_BENCH_V1";

    let legacy_growth_events = parent_set_growth_events(BATCH_COUNT, FILES_PER_BATCH, false);
    let optimized_growth_events = parent_set_growth_events(BATCH_COUNT, FILES_PER_BATCH, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} files_per_batch={FILES_PER_BATCH} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn parent_set_growth_events(batch_count: usize, files_per_batch: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut parents = if reserve {
            HashSet::with_capacity(files_per_batch)
        } else {
            HashSet::new()
        };
        for parent in 0..files_per_batch {
            let previous_capacity = parents.capacity();
            parents.insert(parent);
            growth_events += usize::from(parents.capacity() != previous_capacity);
        }
    }
    growth_events
}

#[test]
fn generated_file_writer_skips_equal_contents_and_updates_changed_contents() {
    let root = temporary_test_root();
    fs::create_dir_all(&root).expect("test root should be created");
    let output = root.join("generated.txt");
    fs::write(&output, "stable").expect("initial generated file should be written");

    let original_permissions = fs::metadata(&output)
        .expect("generated file metadata should be readable")
        .permissions();
    let mut read_only_permissions = original_permissions.clone();
    read_only_permissions.set_readonly(true);
    fs::set_permissions(&output, read_only_permissions)
        .expect("generated file should become read-only");

    assert!(!write_if_changed(&output, "stable")
        .expect("equal generated contents should not rewrite the file"));

    fs::set_permissions(&output, original_permissions)
        .expect("generated file should become writable again");

    assert!(write_if_changed(&output, "changed")
        .expect("changed generated contents should rewrite the file"));
    assert_eq!(
        fs::read_to_string(&output).expect("updated generated file should be readable"),
        "changed"
    );

    fs::remove_dir_all(root).expect("test root should be removable");
}

fn temporary_test_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon-export-generated-incremental-{}-{nonce}",
        std::process::id()
    ))
}
