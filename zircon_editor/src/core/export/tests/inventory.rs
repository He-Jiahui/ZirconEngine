use super::ExportGenerationInventory;

#[test]
fn overlapping_root_and_child_digests_read_each_file_once() {
    let fixture = InventoryFixture::new();
    let child = fixture.root.join("child");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(fixture.root.join("root.txt"), b"root").unwrap();
    std::fs::write(child.join("nested.txt"), b"nested").unwrap();

    let mut inventory = ExportGenerationInventory::default();
    inventory.digest_path(&fixture.root).unwrap();
    inventory.digest_path(&child).unwrap();
    inventory.digest_path(&child.join("nested.txt")).unwrap();

    assert_eq!(inventory.file_reads, 2);
}

#[test]
fn file_digest_streams_content_without_an_owned_whole_file_buffer() {
    let source = include_str!("../inventory.rs");
    let whole_file_read = ["let bytes = fs::", "read(path)?"].concat();
    let streaming_chunk = ["let mut chunk = [0_u8; ", "64 * 1024]"].concat();

    assert!(!source.contains(&whole_file_read));
    assert!(source.contains(&streaming_chunk));
}

#[test]
fn invalidating_a_rebuilt_subtree_refreshes_its_digest() {
    let fixture = InventoryFixture::new();
    let rebuilt = fixture.root.join("rebuilt");
    std::fs::create_dir_all(&rebuilt).unwrap();
    let artifact = rebuilt.join("artifact.bin");
    std::fs::write(&artifact, b"before").unwrap();

    let mut inventory = ExportGenerationInventory::default();
    let before = inventory.digest_path(&fixture.root).unwrap();
    std::fs::write(&artifact, b"after").unwrap();
    inventory.invalidate_subtree(&rebuilt);
    let after = inventory.digest_path(&fixture.root).unwrap();

    assert_ne!(before, after);
    assert_eq!(inventory.file_reads, 2);
}

#[test]
fn persistent_cache_reuses_unchanged_file_without_reading_content() {
    let fixture = InventoryFixture::new();
    let artifact = fixture.root.join("artifact.bin");
    let cache = fixture.root.join("cache/inventory.json");
    std::fs::write(&artifact, b"unchanged").unwrap();

    let first_digest = {
        let mut inventory = ExportGenerationInventory::with_persistent_cache(cache.clone());
        let digest = inventory.digest_path(&artifact).unwrap();
        assert_eq!(inventory.file_reads, 1);
        digest
    };
    let mut inventory = ExportGenerationInventory::with_persistent_cache(cache);
    let second_digest = inventory.digest_path(&artifact).unwrap();

    assert_eq!(second_digest, first_digest);
    assert_eq!(inventory.file_reads, 0);
    assert_eq!(inventory.file_bytes_read, 0);
    assert_eq!(inventory.file_hashes, 0);
}

#[test]
#[ignore = "performance evidence; run explicitly with --ignored --nocapture"]
fn unchanged_warm_inventory_reports_zero_content_io_and_p95() {
    const ITERATIONS: usize = 64;

    let fixture = InventoryFixture::new();
    let artifact = fixture.root.join("artifact.bin");
    let cache = fixture.root.join("cache/inventory.json");
    std::fs::write(&artifact, vec![0x5a; 1024 * 1024]).unwrap();

    {
        let mut inventory = ExportGenerationInventory::with_persistent_cache(cache.clone());
        inventory.digest_path(&artifact).unwrap();
        assert_eq!(inventory.file_reads, 1);
        assert_eq!(inventory.file_bytes_read, 1024 * 1024);
        assert_eq!(inventory.file_hashes, 1);
    }

    let mut elapsed_micros = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let started_at = std::time::Instant::now();
        let mut inventory = ExportGenerationInventory::with_persistent_cache(cache.clone());
        inventory.digest_path(&artifact).unwrap();
        elapsed_micros.push(started_at.elapsed().as_micros());
        assert_eq!(inventory.file_reads, 0);
        assert_eq!(inventory.file_bytes_read, 0);
        assert_eq!(inventory.file_hashes, 0);
    }
    elapsed_micros.sort_unstable();
    let p95_index = (ITERATIONS * 95).div_ceil(100).saturating_sub(1);
    let p95_micros = elapsed_micros[p95_index];

    eprintln!(
        "editor15 warm inventory evidence: iterations={ITERATIONS} content_bytes_read=0 content_hash_count=0 p95_micros={p95_micros}"
    );
}

#[test]
fn same_size_rewrite_invalidates_persistent_digest() {
    let fixture = InventoryFixture::new();
    let artifact = fixture.root.join("artifact.bin");
    let cache = fixture.root.join("cache/inventory.json");
    std::fs::write(&artifact, b"before!!").unwrap();

    let before = {
        let mut inventory = ExportGenerationInventory::with_persistent_cache(cache.clone());
        inventory.digest_path(&artifact).unwrap()
    };
    std::thread::sleep(std::time::Duration::from_millis(2));
    std::fs::write(&artifact, b"after!!!").unwrap();
    let mut inventory = ExportGenerationInventory::with_persistent_cache(cache);
    let after = inventory.digest_path(&artifact).unwrap();

    assert_ne!(after, before);
    assert_eq!(inventory.file_reads, 1);
}

#[test]
fn directory_refresh_prunes_deleted_file_from_persistent_cache() {
    let fixture = InventoryFixture::new();
    let source_root = fixture.root.join("source");
    let cache = fixture.root.join("cache/inventory.json");
    let retained = source_root.join("retained.bin");
    let deleted = source_root.join("deleted.bin");
    std::fs::create_dir_all(&source_root).unwrap();
    std::fs::write(&retained, b"retained").unwrap();
    std::fs::write(&deleted, b"deleted").unwrap();
    let canonical_deleted = std::fs::canonicalize(&deleted).unwrap();

    {
        let mut inventory = ExportGenerationInventory::with_persistent_cache(cache.clone());
        inventory.digest_path(&source_root).unwrap();
    }
    std::fs::remove_file(deleted).unwrap();
    {
        let mut inventory = ExportGenerationInventory::with_persistent_cache(cache.clone());
        inventory.digest_path(&source_root).unwrap();
    }
    let inventory = ExportGenerationInventory::with_persistent_cache(cache);

    assert!(!inventory
        .persistent_file_digests
        .contains_key(&canonical_deleted));
    assert_eq!(inventory.persistent_file_digests.len(), 1);
}

#[test]
fn tool_identity_is_probed_once_per_generation() {
    let mut inventory = ExportGenerationInventory::default();
    let first = inventory
        .tool_identity_with_probe(
            "cargo",
            std::ffi::OsStr::new("cargo"),
            &["--version"],
            || Ok(b"cargo test-version".to_vec()),
        )
        .unwrap();
    let second = inventory
        .tool_identity_with_probe(
            "cargo",
            std::ffi::OsStr::new("cargo"),
            &["--version"],
            || panic!("cached generation identity must not probe the tool twice"),
        )
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(inventory.tool_probes, 1);
}

#[test]
fn unchanged_tool_identity_does_not_rewrite_persistent_cache() {
    let fixture = InventoryFixture::new();
    let cache = fixture.root.join("cache/inventory.json");
    {
        let mut inventory = ExportGenerationInventory::with_persistent_cache(cache.clone());
        inventory
            .tool_identity_with_probe(
                "cargo",
                std::ffi::OsStr::new("cargo"),
                &["--version"],
                || Ok(b"cargo test-version".to_vec()),
            )
            .unwrap();
        assert!(inventory.persistent_cache_dirty);
    }

    let mut inventory = ExportGenerationInventory::with_persistent_cache(cache);
    inventory
        .tool_identity_with_probe(
            "cargo",
            std::ffi::OsStr::new("cargo"),
            &["--version"],
            || Ok(b"cargo test-version".to_vec()),
        )
        .unwrap();

    assert_eq!(inventory.tool_probes, 1);
    assert!(!inventory.persistent_cache_dirty);
}

struct InventoryFixture {
    root: std::path::PathBuf,
}

impl InventoryFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-editor-export-inventory-{}-{:x}",
            std::process::id(),
            fixture_nonce()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }
}

impl Drop for InventoryFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn fixture_nonce() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::thread::current().id().hash(&mut hasher);
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}
