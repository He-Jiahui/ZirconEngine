use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::asset::AssetUri;
use crate::core::framework::render::{
    IblBakeArtifactBlob, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactPayload, IblBakeArtifactRequest, IblBakeKey, SourceCubemapMipChain,
    SourceCubemapPrefilterQuality,
};
use crate::core::math::Real;
use crate::core::resource::io::transaction::TransactionFault;

use super::{
    IblBakeArtifactAssetDerivedError, IblSourceCubemapStagingRead, IblSourceCubemapStagingStore,
};

static TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

fn assert_texels_close(actual: &[[Real; 4]], expected: &[[Real; 4]]) {
    assert_eq!(actual.len(), expected.len(), "texel counts must match");
    for (texel_index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        for component in 0..4 {
            assert!(
                (actual[component] - expected[component]).abs() <= 1.0e-5,
                "texel {texel_index} component {component} differs: actual={}, expected={}",
                actual[component],
                expected[component]
            );
        }
    }
}

#[test]
fn source_path_ignores_pmrem_layout_and_optional_derived_contents() {
    let store = IblSourceCubemapStagingStore::new("E:/cache");
    let source = request(
        7,
        [11; 4],
        256,
        9,
        128,
        8,
        IblBakeArtifactContents::PMREM_SH9,
    );
    let alternate_recipe = request(
        7,
        [11; 4],
        256,
        9,
        64,
        7,
        IblBakeArtifactContents::PMREM_SH9_IEM,
    );

    assert_eq!(
        store.source_cubemap_path(&source),
        store.source_cubemap_path(&alternate_recipe)
    );
    assert_ne!(
        store.asset_derived_store().asset_derived_path(&source),
        store
            .asset_derived_store()
            .asset_derived_path(&alternate_recipe)
    );
}

#[test]
fn source_path_changes_with_source_identity_or_layout() {
    let store = IblSourceCubemapStagingStore::new(Path::new("E:/cache"));
    let source = request(
        7,
        [11; 4],
        256,
        9,
        128,
        8,
        IblBakeArtifactContents::PMREM_SH9,
    );
    let changed_bytes = request(
        8,
        [12; 4],
        256,
        9,
        128,
        8,
        IblBakeArtifactContents::PMREM_SH9,
    );
    let changed_layout = request(
        7,
        [11; 4],
        512,
        10,
        128,
        8,
        IblBakeArtifactContents::PMREM_SH9,
    );

    assert_ne!(
        store.source_cubemap_path(&source),
        store.source_cubemap_path(&changed_bytes)
    );
    assert_ne!(
        store.source_cubemap_path(&source),
        store.source_cubemap_path(&changed_layout)
    );
}

#[test]
fn source_zcube_writer_delegates_to_bundle_publication() {
    let source = include_str!("ibl_source_cubemap_staging.rs");
    let writer = source
        .split("pub fn write_source_cubemap_zcube(")
        .nth(1)
        .and_then(|writer| writer.split("pub fn read_source_cubemap_zcube(").next())
        .expect("source staging must retain the compatibility writer");

    assert!(writer.contains("write_source_cubemap_staged_bundle("));
    assert!(!writer.contains("PreparedFileWrite::new("));
    assert!(!writer.contains("atomic_write("));
    assert!(!writer.contains("fs::write("));
}

#[test]
fn prepared_bundle_leaves_live_cache_targets_unchanged_until_commit() {
    let root = test_directory("prepared-bundle-no-live-write");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let source_path = store.source_cubemap_path(&request);
    let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);
    let manifest_path = store.bundle_manifest_path(&request);

    let (writes, bundle) = store
        .prepare_source_cubemap_staged_bundle(
            &request,
            uri(),
            &cubemap([0.25, 0.5, 0.75, 1.0]),
            None,
        )
        .expect("IBL bundle preparation must encode without publishing");

    assert_eq!(writes.len(), 3);
    assert_eq!(writes[2].path(), manifest_path);
    assert_eq!(bundle.source_zcube().path(), source_path);
    assert_eq!(bundle.asset_derived().path(), asset_derived_path);
    assert!(
        !source_path.exists() && !asset_derived_path.exists() && !manifest_path.exists(),
        "prepared IBL bytes must not appear in the live cache before their owner commits"
    );
    assert!(
        !store.bundle_journal_directory().exists(),
        "preparation must not create an independent IBL durable transaction"
    );
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn fresh_cache_read_is_missing_without_creating_a_bundle_journal() {
    let root = test_directory("fresh-cache-read");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);

    assert!(matches!(
        store
            .read_source_cubemap_zcube(&request, uri())
            .expect("a fresh cache must not require a bundle journal"),
        IblSourceCubemapStagingRead::Missing
    ));
    assert!(
        fs::read_dir(store.bundle_journal_directory())
            .expect("a source read miss must prepare an empty owner journal")
            .next()
            .is_none(),
        "a source read miss must not create a bundle transaction"
    );
}

#[test]
fn environment_read_recovers_a_bundle_staged_after_initial_recovery_before_missing() {
    let root = test_directory("source-miss-after-recovery");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let publisher = store.clone();
    let mut staged = false;

    let result = store.read_source_cubemap_environment_with_snapshot_hooks(
        &request,
        uri(),
        || {
            if staged {
                return Ok(());
            }
            staged = true;
            publisher
                .write_source_cubemap_staged_bundle_for_test(
                    &request,
                    uri(),
                    &cubemap([0.25, 0.5, 0.75, 1.0]),
                    None,
                    TransactionFault::CrashAfterStaging(0),
                )
                .expect_err("fixture must retain the staged bundle journal");
            Ok(())
        },
        || Ok(()),
        || Ok(()),
        || Ok(()),
    );

    assert!(
        staged,
        "fixture must stage after the initial recovery check"
    );
    assert!(matches!(
        result,
        Err(super::IblSourceCubemapStagingError::MissingBundleManifest)
    ));
    assert_eq!(
        fs::read_dir(store.bundle_journal_directory())
            .expect("the next reader iteration must recover the staged journal")
            .count(),
        0,
        "a miss must not bypass recovery for a bundle staged after its initial check"
    );
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn environment_read_miss_holds_the_bundle_owner_barrier() {
    let root = test_directory("environment-miss-owner-barrier");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let publisher = store.clone();
    let mut publisher_blocked = false;

    let result = store.read_source_cubemap_environment_with_snapshot_hooks(
        &request,
        uri(),
        || Ok(()),
        || Ok(()),
        || Ok(()),
        || {
            publisher_blocked = true;
            publisher
                .write_source_cubemap_staged_bundle(
                    &request,
                    uri(),
                    &cubemap([0.25, 0.5, 0.75, 1.0]),
                    None,
                )
                .expect_err("a publisher cannot start after the miss barrier is held");
            Ok(())
        },
    );

    assert!(
        publisher_blocked,
        "fixture must attempt publication under the barrier"
    );
    assert!(matches!(
        result,
        Err(super::IblSourceCubemapStagingError::MissingBundleManifest)
    ));
    assert!(
        !store.source_cubemap_path(&request).exists(),
        "blocked publication must not create a source target"
    );
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn source_read_recovers_a_bundle_staged_after_initial_recovery_before_missing() {
    let root = test_directory("source-only-miss-after-recovery");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let publisher = store.clone();
    let mut staged = false;

    let result = store.read_source_cubemap_zcube_with_snapshot_hooks(
        &request,
        || {
            if staged {
                return Ok(());
            }
            staged = true;
            publisher
                .write_source_cubemap_staged_bundle_for_test(
                    &request,
                    uri(),
                    &cubemap([0.25, 0.5, 0.75, 1.0]),
                    None,
                    TransactionFault::CrashAfterStaging(0),
                )
                .expect_err("fixture must retain the staged bundle journal");
            Ok(())
        },
        || Ok(()),
    );

    assert!(
        staged,
        "fixture must stage after the initial recovery check"
    );
    assert!(matches!(result, Ok(IblSourceCubemapStagingRead::Missing)));
    assert_eq!(
        fs::read_dir(store.bundle_journal_directory())
            .expect("the next reader iteration must recover the staged journal")
            .count(),
        0,
        "a source-only miss must not bypass recovery for a staged bundle"
    );
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn source_read_miss_holds_the_bundle_owner_barrier() {
    let root = test_directory("source-only-miss-owner-barrier");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let publisher = store.clone();
    let mut publisher_blocked = false;

    let result = store.read_source_cubemap_zcube_with_snapshot_hooks(
        &request,
        || Ok(()),
        || {
            publisher_blocked = true;
            publisher
                .write_source_cubemap_staged_bundle(
                    &request,
                    uri(),
                    &cubemap([0.25, 0.5, 0.75, 1.0]),
                    None,
                )
                .expect_err("a publisher cannot start after the miss barrier is held");
            Ok(())
        },
    );

    assert!(
        publisher_blocked,
        "fixture must attempt publication under the barrier"
    );
    assert!(matches!(result, Ok(IblSourceCubemapStagingRead::Missing)));
    assert!(
        !store.source_cubemap_path(&request).exists(),
        "blocked publication must not create a source target"
    );
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn interrupted_bundle_staging_recovers_before_the_next_read() {
    let root = test_directory("interrupted-staging");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);

    store
        .write_source_cubemap_staged_bundle_for_test(
            &request,
            uri(),
            &cubemap([0.25, 0.5, 0.75, 1.0]),
            None,
            TransactionFault::CrashAfterStaging(0),
        )
        .expect_err("the injected interruption must retain staging recovery evidence");
    assert!(
        store.bundle_journal_directory().exists(),
        "the interrupted bundle must retain its journal"
    );

    assert!(matches!(
        store
            .read_source_cubemap_zcube(&request, uri())
            .expect("the next read must recover the pre-commit bundle"),
        IblSourceCubemapStagingRead::Missing
    ));
    assert_eq!(
        fs::read_dir(store.bundle_journal_directory())
            .expect("recovered journal directory must remain inspectable")
            .count(),
        0,
        "recovery must remove the interrupted journal and staged files"
    );
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn interrupted_bundle_publication_recovers_the_previous_source_and_derived_generation() {
    let root = test_directory("interrupted-bundle");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let initial = cubemap([0.25, 0.5, 0.75, 1.0]);
    let replacement = cubemap([0.75, 0.5, 0.25, 1.0]);

    store
        .write_source_cubemap_staged_bundle(&request, uri(), &initial, None)
        .expect("initial source bundle must publish");
    let source_path = store.source_cubemap_path(&request);
    let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);
    let initial_source_bytes = fs::read(&source_path).expect("initial source must exist");
    let initial_derived_bytes =
        fs::read(&asset_derived_path).expect("initial derived artifact must exist");

    store
        .write_source_cubemap_staged_bundle_for_test(
            &request,
            uri(),
            &replacement,
            None,
            TransactionFault::CrashAfterCommit(0),
        )
        .expect_err("the injected interruption must retain recovery evidence");
    assert_ne!(
        fs::read(&source_path).expect("source first target must have changed"),
        initial_source_bytes
    );
    assert_eq!(
        fs::read(&asset_derived_path).expect("derived second target must remain old"),
        initial_derived_bytes
    );

    assert!(matches!(
        store
            .read_source_cubemap_zcube(&request, uri())
            .expect("the next read must recover the interrupted bundle"),
        IblSourceCubemapStagingRead::Hit(_)
    ));
    assert_eq!(
        fs::read(&source_path).expect("recovered source must exist"),
        initial_source_bytes
    );
    assert_eq!(
        fs::read(&asset_derived_path).expect("recovered derived artifact must exist"),
        initial_derived_bytes
    );
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn environment_read_retries_when_bundle_publication_interleaves_the_snapshot() {
    let root = test_directory("interleaved-read");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let initial = cubemap([0.25, 0.5, 0.75, 1.0]);
    let replacement = cubemap([0.75, 0.5, 0.25, 1.0]);
    store
        .write_source_cubemap_staged_bundle(&request, uri(), &initial, None)
        .expect("initial source bundle must publish");

    let publisher = store.clone();
    let mut published = false;
    let environment = store
        .read_source_cubemap_environment_with_snapshot_hooks(
            &request,
            uri(),
            || Ok(()),
            || {
                if published {
                    return Ok(());
                }
                published = true;
                publisher
                    .write_source_cubemap_staged_bundle(&request, uri(), &replacement, None)
                    .map(|_| ())
            },
            || Ok(()),
            || Ok(()),
        )
        .expect("the reader must retry after an interleaved bundle publication");

    assert!(
        published,
        "the test hook must publish the replacement bundle"
    );
    assert_texels_close(
        environment.mip_chain.source_texels(),
        replacement.source_texels(),
    );
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn environment_read_retries_a_missing_recipe_after_reused_source_publication() {
    let root = test_directory("interleaved-reused-source-read");
    let store = IblSourceCubemapStagingStore::new(&root);
    let source_request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let reused_recipe = request(7, [11; 4], 4, 3, 1, 1, IblBakeArtifactContents::PMREM_SH9);
    let source = cubemap([0.25, 0.5, 0.75, 1.0]);
    let reused_source = cubemap_with_pmrem([0.25, 0.5, 0.75, 1.0], 1, 1);
    store
        .write_source_cubemap_staged_bundle(&source_request, uri(), &source, None)
        .expect("initial source bundle must publish");

    let publisher = store.clone();
    let mut published = false;
    let environment = store
        .read_source_cubemap_environment_with_snapshot_hooks(
            &reused_recipe,
            uri(),
            || Ok(()),
            || Ok(()),
            || Ok(()),
            || {
                if published {
                    return Ok(());
                }
                published = true;
                publisher
                    .write_source_cubemap_staged_bundle(&reused_recipe, uri(), &reused_source, None)
                    .map(|_| ())
            },
        )
        .expect("the reader must retry after the missing recipe is published");

    assert!(published, "the test hook must publish the missing recipe");
    assert_eq!(environment.mip_chain.pmrem_face_size(), 1);
    assert_eq!(environment.mip_chain.pmrem_mip_count(), 1);
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn environment_read_retries_a_rejected_derived_artifact_replaced_by_bundle() {
    let root = test_directory("interleaved-rejected-derived-read");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let source = cubemap([0.25, 0.5, 0.75, 1.0]);
    store
        .write_source_cubemap_staged_bundle(&request, uri(), &source, None)
        .expect("initial source bundle must publish");
    let derived_path = store.asset_derived_store().asset_derived_path(&request);
    fs::write(&derived_path, b"rejected derived artifact")
        .expect("fixture must replace the derived artifact with invalid bytes");

    let publisher = store.clone();
    let mut published = false;
    let environment = store
        .read_source_cubemap_environment_with_snapshot_hooks(
            &request,
            uri(),
            || Ok(()),
            || Ok(()),
            || {
                if published {
                    return Ok(());
                }
                published = true;
                publisher
                    .write_source_cubemap_staged_bundle(&request, uri(), &source, None)
                    .map(|_| ())
            },
            || Ok(()),
        )
        .expect("the reader must retry after a rejected artifact is repaired as a bundle");

    assert!(published, "the test hook must publish the repaired bundle");
    assert_texels_close(
        environment.mip_chain.source_texels(),
        source.source_texels(),
    );
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

#[test]
fn standalone_derived_blob_cannot_replace_a_paired_source_bundle() {
    let root = test_directory("reject-standalone-derived");
    let store = IblSourceCubemapStagingStore::new(&root);
    let request = request(7, [11; 4], 4, 3, 2, 2, IblBakeArtifactContents::PMREM_SH9);
    let source = cubemap([0.25, 0.5, 0.75, 1.0]);
    store
        .write_source_cubemap_staged_bundle(&request, uri(), &source, None)
        .expect("initial source bundle must publish");
    let descriptor = IblBakeArtifactDescriptor::current_for_request(&request);
    let payload = IblBakeArtifactPayload::from_source_cubemap(descriptor, &source, None)
        .expect("matching standalone fixture must encode");

    assert!(matches!(
        store
            .asset_derived_store()
            .write_asset_derived_blob(&IblBakeArtifactBlob::from_payload(payload)),
        Err(IblBakeArtifactAssetDerivedError::PairedSourceRequiresBundle { .. })
    ));
    fs::remove_dir_all(root).expect("test cache root must be removable");
}

fn request(
    revision: u64,
    source_hash: [u32; 4],
    source_face_size: u32,
    source_mip_count: u32,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    contents: IblBakeArtifactContents,
) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        IblBakeKey::source_cubemap(revision, source_hash),
        source_face_size,
        source_mip_count,
    )
    .with_pmrem_layout(pmrem_face_size, pmrem_mip_count)
    .with_required_contents(contents)
}

fn cubemap(color: [Real; 4]) -> SourceCubemapMipChain {
    cubemap_with_pmrem(color, 2, 2)
}

fn cubemap_with_pmrem(
    color: [Real; 4],
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
) -> SourceCubemapMipChain {
    SourceCubemapMipChain::from_equirect_with_pmrem_layout(
        4,
        pmrem_face_size,
        pmrem_mip_count,
        SourceCubemapPrefilterQuality::Fast,
        move |_, _| color,
    )
}

fn uri() -> AssetUri {
    AssetUri::parse("res://environment/interrupted-bundle.hdr")
        .expect("test source URI must be valid")
}

fn test_directory(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-ibl-source-staging-{label}-{}-{}",
        std::process::id(),
        TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed)
    ))
}
