use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::project::ProjectPaths;
use crate::core::framework::render::{
    GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
    SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::resource::ResourceId;

use super::{
    atomic_create_or_verify, atomic_create_or_verify_with_hook, atomic_manifest_commit,
    atomic_manifest_commit_with_hook, shader_cache_root_for_project, ShaderVariantCacheDisk,
    ShaderVariantCacheDiskError, ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup,
};
use crate::core::framework::render::ShaderVariantPrewarmSource;

#[cfg(any(unix, windows))]
#[test]
fn shader_cache_roots_keep_the_physical_project_identity_for_relative_layouts() {
    let parent = unique_shader_cache_project_root("physical-identity");
    let physical_project = parent.join("physical-project");
    fs::create_dir_all(&physical_project).unwrap();
    let project_alias = parent.join("project-alias");
    create_directory_link(&physical_project, &project_alias);

    let default_root = shader_cache_root_for_project(&project_alias, None);
    let configured_root =
        shader_cache_root_for_project(&project_alias, Some(Path::new("derived/shaders")));
    let expected_project = ProjectPaths::resolve_existing_path(&physical_project).unwrap();

    fs::remove_dir_all(&parent).unwrap();
    assert_eq!(
        default_root,
        expected_project.join(".zircon/cache/shader_variants")
    );
    assert_eq!(configured_root, expected_project.join("derived/shaders"));
}

fn unique_shader_cache_project_root(case_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "zircon-shader-cache-{case_name}-{}-{timestamp}",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    path
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create shader-cache project alias");
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) {
    let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
    let output = std::process::Command::new("cmd")
        .args(["/D", "/S", "/C"])
        .arg(command)
        .output()
        .expect("start mklink for shader-cache project alias");
    assert!(
        output.status.success(),
        "create shader-cache project junction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn render_shader_variant_cache_hits_disk_after_restart() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_variant_cache_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let source = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-test");
    let key = disk_key(&variant_key(), &source);
    let cache = ShaderVariantCacheDisk::new(&root);

    assert!(matches!(
        cache.write(&key, "fn changed() {}"),
        Err(super::ShaderVariantCacheDiskError::SourceHashMismatch)
    ));
    cache
        .write(&key, "fn main() {}")
        .expect("write variant cache");

    let restarted = ShaderVariantCacheDisk::new(&root);
    let lookup = restarted.lookup(&key);

    match lookup {
        ShaderVariantCacheDiskLookup::Hit(entry) => {
            assert_eq!(entry.wgsl_source, "fn main() {}");
            assert_eq!(entry.meta.canonical_string, key.canonical_string);
            assert_eq!(entry.meta.source_id, source.id);
        }
        other => panic!("expected disk hit, got {other:?}"),
    }
    let changed_source = cache_source("fn changed() {}", "template-r1", "naga-test", "wgpu-test");
    let changed_key = disk_key(&variant_key(), &changed_source);
    let changed_wgpu = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-next");
    let changed_wgpu_key = disk_key(&variant_key(), &changed_wgpu);
    assert_ne!(key.hash, changed_key.hash);
    assert_ne!(key.hash, changed_wgpu_key.hash);
    assert!(matches!(
        restarted.lookup(&changed_key),
        ShaderVariantCacheDiskLookup::Miss
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_shader_variant_cache_treats_corrupt_entry_as_miss_after_cleanup() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_variant_cache_corrupt_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let source = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-test");
    let key = disk_key(&variant_key(), &source);
    let cache = ShaderVariantCacheDisk::new(&root);
    cache
        .write(&key, "fn main() {}")
        .expect("write variant cache");
    let shard = key.hash.get(0..2).unwrap_or("00");
    fs::write(
        root.join("v3")
            .join(shard)
            .join(format!("{}.manifest", key.hash)),
        b"{ invalid json",
    )
    .expect("corrupt manifest");

    assert!(matches!(
        cache.lookup(&key),
        ShaderVariantCacheDiskLookup::Error(_)
    ));
    assert!(matches!(
        cache.lookup(&key),
        ShaderVariantCacheDiskLookup::Miss
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_shader_variant_cache_manifest_is_the_only_publication_point() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_variant_cache_manifest_commit_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let source = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-test");
    let key = disk_key(&variant_key(), &source);
    let cache = ShaderVariantCacheDisk::new(&root);
    let path = cache.entry_path(&key);
    fs::create_dir_all(&path.directory).unwrap();
    let compressed = zstd::stream::encode_all(b"fn main() {}".as_slice(), 3).unwrap();
    fs::write(&path.payload, compressed).unwrap();
    assert!(matches!(
        cache.lookup(&key),
        ShaderVariantCacheDiskLookup::Miss
    ));
    cache.write(&key, "fn main() {}").unwrap();
    assert!(matches!(
        cache.lookup(&key),
        ShaderVariantCacheDiskLookup::Hit(_)
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_shader_variant_cache_manifest_without_payload_is_not_a_hit() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_variant_cache_manifest_only_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let source = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-test");
    let key = disk_key(&variant_key(), &source);
    let cache = ShaderVariantCacheDisk::new(&root);
    cache.write(&key, "fn main() {}").unwrap();
    let path = cache.entry_path(&key);
    fs::remove_file(&path.payload).unwrap();
    assert!(matches!(
        cache.lookup(&key),
        ShaderVariantCacheDiskLookup::Error(_)
    ));
    assert!(matches!(
        cache.lookup(&key),
        ShaderVariantCacheDiskLookup::Miss
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_shader_variant_cache_conflicts_verify_exact_bytes() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_variant_cache_conflict_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let payload = root.join("payload");
    atomic_create_or_verify(&payload, b"same").unwrap();
    atomic_create_or_verify(&payload, b"same").unwrap();
    assert!(matches!(
        atomic_create_or_verify(&payload, b"different"),
        Err(super::ShaderVariantCacheDiskError::CorruptTarget(_))
    ));
    let manifest = root.join("manifest");
    atomic_manifest_commit(&manifest, b"{}").unwrap();
    atomic_manifest_commit(&manifest, b"{}").unwrap();
    assert!(matches!(
        atomic_manifest_commit(&manifest, b"{\"identity\":\"other\"}"),
        Err(super::ShaderVariantCacheDiskError::CorruptTarget(_))
    ));
    let directory_target = root.join("directory-target");
    fs::create_dir_all(&directory_target).unwrap();
    assert!(matches!(
        atomic_create_or_verify(&directory_target, b"payload"),
        Err(super::ShaderVariantCacheDiskError::Io(_))
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_shader_variant_cache_create_only_race_preserves_first_different_writer() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_variant_cache_create_only_race_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let payload = root.join("payload");
    let payload_barrier = Arc::new(Barrier::new(2));
    let payload_left = payload.clone();
    let payload_left_barrier = payload_barrier.clone();
    let payload_left_thread = std::thread::spawn(move || {
        let after_check = move || {
            payload_left_barrier.wait();
        };
        atomic_create_or_verify_with_hook(&payload_left, b"left-payload", Some(&after_check))
    });
    let payload_right_barrier = payload_barrier.clone();
    let payload_right_thread = std::thread::spawn(move || {
        let after_check = move || {
            payload_right_barrier.wait();
        };
        atomic_create_or_verify_with_hook(&payload, b"right-payload", Some(&after_check))
    });
    let payload_results = [
        payload_left_thread.join().unwrap(),
        payload_right_thread.join().unwrap(),
    ];
    assert_eq!(
        payload_results
            .iter()
            .filter(|result| result.is_err())
            .count(),
        1
    );
    assert_eq!(
        payload_results
            .iter()
            .filter(|result| result.is_ok())
            .count(),
        1
    );
    assert!(payload_results
        .iter()
        .any(|result| matches!(result, Err(ShaderVariantCacheDiskError::CorruptTarget(_)))));
    let payload_bytes = fs::read(root.join("payload")).unwrap();
    assert!(
        payload_bytes.as_slice() == b"left-payload" || payload_bytes.as_slice() == b"right-payload"
    );

    let manifest = root.join("manifest");
    let manifest_barrier = Arc::new(Barrier::new(2));
    let manifest_left = manifest.clone();
    let manifest_left_barrier = manifest_barrier.clone();
    let manifest_left_thread = std::thread::spawn(move || {
        let after_check = move || {
            manifest_left_barrier.wait();
        };
        atomic_manifest_commit_with_hook(&manifest_left, b"left-manifest", Some(&after_check))
    });
    let manifest_right_barrier = manifest_barrier.clone();
    let manifest_right_thread = std::thread::spawn(move || {
        let after_check = move || {
            manifest_right_barrier.wait();
        };
        atomic_manifest_commit_with_hook(&manifest, b"right-manifest", Some(&after_check))
    });
    let manifest_results = [
        manifest_left_thread.join().unwrap(),
        manifest_right_thread.join().unwrap(),
    ];
    assert_eq!(
        manifest_results
            .iter()
            .filter(|result| result.is_err())
            .count(),
        1
    );
    assert_eq!(
        manifest_results
            .iter()
            .filter(|result| result.is_ok())
            .count(),
        1
    );
    assert!(manifest_results
        .iter()
        .any(|result| matches!(result, Err(ShaderVariantCacheDiskError::CorruptTarget(_)))));
    let manifest_bytes = fs::read(root.join("manifest")).unwrap();
    assert!(
        manifest_bytes.as_slice() == b"left-manifest"
            || manifest_bytes.as_slice() == b"right-manifest"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_shader_variant_cache_same_key_writers_publish_one_verified_entry() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_variant_cache_concurrent_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let source = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-test");
    let key = disk_key(&variant_key(), &source);
    let left = ShaderVariantCacheDisk::new(&root);
    let right = ShaderVariantCacheDisk::new(&root);
    let left_key = key.clone();
    let right_key = key.clone();
    let left_thread = std::thread::spawn(move || left.write(&left_key, "fn main() {}"));
    let right_thread = std::thread::spawn(move || right.write(&right_key, "fn main() {}"));
    left_thread.join().unwrap().unwrap();
    right_thread.join().unwrap().unwrap();
    assert!(matches!(
        ShaderVariantCacheDisk::new(&root).lookup(&key),
        ShaderVariantCacheDiskLookup::Hit(_)
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_shader_variant_cache_reuses_first_manifest_observation() {
    let root = unique_shader_cache_project_root("repeated-manifest-observation");
    let cache = ShaderVariantCacheDisk::new(&root);
    let source = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-test");
    let key = disk_key(&variant_key(), &source);
    let first = cache.write(&key, &source.wgsl_source).unwrap();
    let path = cache.entry_path(&key);
    let payload = fs::read(&path.payload).unwrap();
    let mut published = first.meta;
    // Fixed historical observation (not the current wall clock); no sleep.
    published.created_unix_seconds = 1;
    let published_bytes = serde_json::to_vec_pretty(&published).unwrap();
    fs::write(&path.manifest, &published_bytes).unwrap();

    let again = cache.write(&key, &source.wgsl_source);
    assert_eq!(fs::read(&path.manifest).unwrap(), published_bytes);
    assert_eq!(fs::read(&path.payload).unwrap(), payload);
    let _ = fs::remove_dir_all(&root);
    let again = again.unwrap();
    assert_eq!(again.meta, published);
    assert_eq!(again.wgsl_source, source.wgsl_source);
}

#[test]
fn render_shader_variant_cache_retry_rejects_malformed_or_different_manifest_identity() {
    let root = unique_shader_cache_project_root("repeated-manifest-conflicts");
    let cache = ShaderVariantCacheDisk::new(&root);
    let source = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-test");
    let key = disk_key(&variant_key(), &source);
    let first = cache.write(&key, &source.wgsl_source).unwrap();
    let path = cache.entry_path(&key);
    let payload = fs::read(&path.payload).unwrap();
    let mut published = serde_json::to_value(first.meta).unwrap();
    published["created_unix_seconds"] = serde_json::json!(1);
    for (field, value) in [
        ("source_hash", serde_json::json!("other-source")),
        ("payload_hash", serde_json::json!("other-payload")),
        ("wgpu_version", serde_json::json!("other-toolchain")),
        ("future_identity", serde_json::json!({"producer": "other"})),
        ("created_unix_seconds", serde_json::json!("1")),
        ("created_unix_seconds", serde_json::json!(true)),
        ("created_unix_seconds", serde_json::json!(-1)),
        ("created_unix_seconds", serde_json::json!(1.0)),
    ] {
        let mut changed = published.clone();
        changed[field] = value;
        let changed_bytes = serde_json::to_vec(&changed).unwrap();
        assert_manifest_retry_conflict(&cache, &key, &source.wgsl_source, &changed_bytes, &payload);
    }

    let mut missing = published.clone();
    missing.as_object_mut().unwrap().remove("created_unix_seconds");
    let missing_bytes = serde_json::to_vec(&missing).unwrap();
    assert_manifest_retry_conflict(&cache, &key, &source.wgsl_source, &missing_bytes, &payload);
    let duplicate = published.to_string().replacen('{', r#"{"created_unix_seconds":2,"#, 1);
    assert_manifest_retry_conflict(&cache, &key, &source.wgsl_source, duplicate.as_bytes(), &payload);
    assert_manifest_retry_conflict(&cache, &key, &source.wgsl_source, b"{ invalid json", &payload);
    let _ = fs::remove_dir_all(&root);
}

fn assert_manifest_retry_conflict(
    cache: &ShaderVariantCacheDisk,
    key: &ShaderVariantCacheDiskKey,
    wgsl_source: &str,
    manifest: &[u8],
    payload: &[u8],
) {
    let path = cache.entry_path(key);
    fs::write(&path.manifest, manifest).unwrap();
    let retry = cache.write(key, wgsl_source);
    assert_eq!(fs::read(&path.manifest).unwrap().as_slice(), manifest);
    assert_eq!(fs::read(&path.payload).unwrap().as_slice(), payload);
    assert!(matches!(retry, Err(ShaderVariantCacheDiskError::CorruptTarget(_))), "{retry:?}");
}

#[test]
fn render_shader_variant_cache_retry_rejects_changed_immutable_payload() {
    let root = unique_shader_cache_project_root("repeated-manifest-payload-conflict");
    let cache = ShaderVariantCacheDisk::new(&root);
    let source = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-test");
    let key = disk_key(&variant_key(), &source);
    cache.write(&key, &source.wgsl_source).unwrap();
    let path = cache.entry_path(&key);
    let manifest = fs::read(&path.manifest).unwrap();
    let mut payload = fs::read(&path.payload).unwrap();
    payload[0] ^= 1;
    fs::write(&path.payload, &payload).unwrap();

    let retry = cache.write(&key, &source.wgsl_source);
    assert_eq!(fs::read(&path.manifest).unwrap(), manifest);
    assert_eq!(fs::read(&path.payload).unwrap(), payload);
    let _ = fs::remove_dir_all(&root);
    assert!(matches!(retry, Err(ShaderVariantCacheDiskError::CorruptTarget(message))
        if message.contains("immutable payload")));
}

#[test]
fn render_shader_variant_cache_manifest_commit_conflicts_keep_raw_first_writer() {
    let root = unique_shader_cache_project_root("repeated-manifest-raw-install-conflict");
    let cache = ShaderVariantCacheDisk::new(&root);
    let source = cache_source("fn main() {}", "template-r1", "naga-test", "wgpu-test");
    let key = disk_key(&variant_key(), &source);
    let entry = cache.write(&key, &source.wgsl_source).unwrap();
    let path = cache.entry_path(&key);
    let payload = fs::read(&path.payload).unwrap();
    let mut first = entry.meta;
    first.created_unix_seconds = 1;
    let first_bytes = serde_json::to_vec_pretty(&first).unwrap();
    let mut later = first.clone();
    later.created_unix_seconds = 2;
    let later_bytes = serde_json::to_vec_pretty(&later).unwrap();
    fs::remove_file(&path.manifest).unwrap();
    // Install the winner just after the losing caller checked for a destination.
    let publish_first = || atomic_manifest_commit(&path.manifest, &first_bytes).unwrap();
    let late = atomic_manifest_commit_with_hook(&path.manifest, &later_bytes, Some(&publish_first));
    let preexisting = atomic_manifest_commit(&path.manifest, &later_bytes);
    assert_eq!(late, preexisting);
    assert!(matches!(late, Err(ShaderVariantCacheDiskError::CorruptTarget(_))));
    assert_eq!(fs::read(&path.manifest).unwrap(), first_bytes);
    assert_eq!(fs::read(&path.payload).unwrap(), payload);

    // The typed caller may reuse, but must keep the original published bytes.
    let retry = cache.write(&key, &source.wgsl_source);
    assert_eq!(fs::read(&path.manifest).unwrap(), first_bytes);
    assert_eq!(fs::read(&path.payload).unwrap(), payload);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(retry.unwrap().meta, first);
}

#[test]
fn shader_variant_disk_cache_profiles_lookup_and_write_independently() {
    let source = include_str!("../disk.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("shader disk cache test boundary");

    assert!(source.contains("\"shader_pipeline\", \"disk_cache_lookup\""));
    assert!(source.contains("\"shader_pipeline\", \"disk_cache_write\""));
}

#[test]
fn shader_variant_disk_cache_profiles_each_runtime_io_and_integrity_stage() {
    let source = include_str!("../disk.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("shader disk cache test boundary");
    let expected_scopes = [
        "disk_cache_key",
        "disk_cache_write_source_hash",
        "disk_cache_compress",
        "disk_cache_metadata_encode",
        "disk_cache_payload_commit",
        "disk_cache_metadata_commit",
        "disk_cache_metadata_read",
        "disk_cache_metadata_decode",
        "disk_cache_payload_read",
        "disk_cache_decompress",
        "disk_cache_payload_rehash",
    ];

    for scope in expected_scopes {
        assert!(
            source.contains(&format!("\"shader_pipeline\", \"{scope}\"")),
            "missing shader disk-cache profile scope {scope}"
        );
    }
    assert!(
        source.find("disk_cache_payload_read") < source.find("disk_cache_payload_rehash"),
        "payload integrity timing must follow the measured payload read"
    );
}

fn variant_key() -> ShaderVariantKey {
    ShaderVariantKey {
        material_shader: ResourceId::from_stable_label("res://materials/cache-test.wgsl"),
        material_revision: 3,
        material_layout_hash: 0,
        material_option_bits: 0,
        geometry_source: GeometrySourceId::new(0),
        shading_model: SHADING_MODEL_ID_STANDARD_PBR,
        pass_type: ShaderPassType::Forward,
        features: ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
        quality: ShaderQualityTier::Medium,
        platform_token: "wgpu-test".to_string(),
    }
}

fn cache_source(
    wgsl_source: &str,
    template_revision: &str,
    naga_version: &str,
    wgpu_version: &str,
) -> ShaderVariantPrewarmSource {
    ShaderVariantPrewarmSource::new(
        "res://materials/cache-test.wgsl",
        wgsl_source,
        vec!["include-a".to_string()],
        template_revision,
        naga_version,
        wgpu_version,
    )
}

fn disk_key(
    variant_key: &ShaderVariantKey,
    source: &ShaderVariantPrewarmSource,
) -> ShaderVariantCacheDiskKey {
    ShaderVariantCacheDiskKey::from_variant_key(
        variant_key,
        &source.source_hash,
        &source.include_content_hashes,
        &source.template_revision,
        &source.naga_version,
        &source.wgpu_version,
    )
}
