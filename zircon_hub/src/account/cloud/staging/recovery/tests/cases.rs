use std::{fs, path::PathBuf};

use super::super::tests::{fixture, stage, STAGE};
use super::super::{next_publish_temporary_path, persist_publication_artifact};
use super::*;

fn publication(root: &Path) -> (PublicationArtifact, PathBuf) {
    let parent = root.join("Content");
    fs::create_dir_all(&parent).unwrap();
    let temporary = next_publish_temporary_path(&parent, STAGE, &sha256(b"remote")).unwrap();
    let artifact = PublicationArtifact {
        schema_version: PUBLICATION_SCHEMA_VERSION,
        stage_id: STAGE.into(),
        relative_path: "Content/remote.txt".into(),
        temporary_name: temporary.file_name().unwrap().to_str().unwrap().into(),
        digest: sha256(b"remote"),
        bytes: 6,
    };
    persist_publication_artifact(root, &artifact).unwrap();
    fs::write(&temporary, b"rem").unwrap();
    (artifact, temporary)
}

#[cfg(windows)]
#[test]
fn recovery_retains_private_ancestors_across_an_ordinary_directory_swap_attempt() {
    let (root, guid) = fixture();
    let staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);
    let orphan = next_publish_temporary_path(&staged.directory, STAGE, &sha256(b"orphan")).unwrap();
    fs::write(&orphan, b"partial").unwrap();
    let (external, _) = fixture();
    let external_cloud = external.join("cloud");
    fs::create_dir_all(&external_cloud).unwrap();
    let external_file = external_cloud.join(orphan.file_name().unwrap());
    fs::write(&external_file, b"external must survive").unwrap();
    let lease = ProjectCloudSyncLease::acquire(&root).unwrap();

    let result = recover_with_anchor_hook(&root, &lease, || {
        // These are ordinary directories, so the check exercises handle lifetime without
        // relying on reparse-point rejection or privileged symlink creation.
        assert!(fs::rename(root.join(".zircon"), root.join(".zircon-held")).is_err());
        assert!(fs::rename(&external, root.join(".zircon")).is_err());
    });

    assert_eq!(result, Ok(1));
    assert!(!orphan.exists());
    assert_eq!(fs::read(external_file).unwrap(), b"external must survive");
    assert_eq!(
        fs::read(staged.directory.join("Content/remote.txt")).unwrap(),
        b"remote"
    );
    drop(lease);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(external).unwrap();
}

#[cfg(windows)]
#[test]
fn receipted_cleanup_pins_the_project_parent_until_the_temporary_is_removed() {
    let (root, guid) = fixture();
    let staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);
    let (artifact, temporary) = publication(&root);
    let (external, _) = fixture();
    let external_file = external.join(&artifact.temporary_name);
    fs::write(&external_file, b"external must survive").unwrap();
    let project = AnchoredDirectory::open(&root).unwrap();

    let result = reap_one_with_parent_hook(&project, STAGE, &artifact, || {
        assert!(fs::rename(root.join("Content"), root.join("Content-held")).is_err());
        assert!(fs::rename(&external, root.join("Content")).is_err());
    });

    assert_eq!(result, Ok(()));
    assert!(!temporary.exists());
    assert_eq!(fs::read(external_file).unwrap(), b"external must survive");
    assert_eq!(
        fs::read(staged.directory.join("Content/remote.txt")).unwrap(),
        b"remote"
    );
    drop(project);
    // The existing receipt remains recoverable after its temporary was removed.
    let lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert_eq!(recover_orphaned_publish_temporaries(&root, &lease), Ok(1));
    drop(lease);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(external).unwrap();
}

#[cfg(unix)]
#[test]
fn recovery_rejects_a_moved_private_namespace_before_deleting_any_temporaries() {
    let (root, guid) = fixture();
    let staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);
    let orphan = next_publish_temporary_path(&staged.directory, STAGE, &sha256(b"orphan")).unwrap();
    fs::write(&orphan, b"partial").unwrap();
    let replacement = root.join(".zircon/cloud").join(orphan.file_name().unwrap());
    let held = root.join(".zircon-held");
    let lease = ProjectCloudSyncLease::acquire(&root).unwrap();

    let result = recover_with_anchor_hook(&root, &lease, || {
        fs::rename(root.join(".zircon"), &held).unwrap();
        fs::create_dir_all(replacement.parent().unwrap()).unwrap();
        fs::write(&replacement, b"replacement must survive").unwrap();
    });

    assert_eq!(result, Err("hub_cloud_sync_stage_invalid"));
    assert_eq!(fs::read(replacement).unwrap(), b"replacement must survive");
    let original = held
        .join("cloud/staging")
        .join(STAGE)
        .join(orphan.file_name().unwrap());
    assert_eq!(fs::read(original).unwrap(), b"partial");
    drop(lease);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn receipted_cleanup_rejects_a_replaced_ordinary_project_parent() {
    let (root, guid) = fixture();
    let _staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);
    let (artifact, temporary) = publication(&root);
    let parent = root.join("Content");
    let held = root.join("Content-held");
    let project = AnchoredDirectory::open(&root).unwrap();

    let result = reap_one_with_parent_hook(&project, STAGE, &artifact, || {
        fs::rename(&parent, &held).unwrap();
        fs::create_dir_all(&parent).unwrap();
        fs::write(&temporary, b"replacement must survive").unwrap();
    });

    assert_eq!(result, Err("hub_cloud_sync_stage_invalid"));
    assert_eq!(fs::read(&temporary).unwrap(), b"replacement must survive");
    assert_eq!(
        fs::read(held.join(&artifact.temporary_name)).unwrap(),
        b"rem"
    );
    drop(project);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_entry_limit_preserves_temporaries_when_the_remaining_budget_is_exhausted() {
    let (root, _) = fixture();
    let temporary = next_publish_temporary_path(&root, STAGE, &sha256(b"orphan")).unwrap();
    fs::write(&temporary, b"preserve").unwrap();
    let directory = AnchoredDirectory::open(&root).unwrap();
    let mut budget = ArtifactScanBudget {
        entries: super::super::MAX_ARTIFACT_ENTRIES,
        ..Default::default()
    };
    let mut removed = 0;

    assert_eq!(
        reap_managed_publish_temporaries(&directory, &mut budget, &mut removed, 0),
        Err("hub_cloud_sync_storage_quota_exceeded")
    );
    assert_eq!(removed, 0);
    assert_eq!(fs::read(temporary).unwrap(), b"preserve");
    drop(directory);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn recovery_keeps_one_stage_anchor_between_marker_and_payload_reads() {
    let (root, guid) = fixture();
    let staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);
    let (artifact, temporary) = publication(&root);
    let moved = staged.directory.with_file_name("stage-held-between-reads");
    let project = AnchoredDirectory::open(&root).unwrap();

    let result = reap_one_with_hooks(
        &project,
        STAGE,
        &artifact,
        || {
            // No symlink or reparse point is involved. The stage anchor must keep
            // the same ordinary directory alive across both validated reads.
            assert!(fs::rename(&staged.directory, &moved).is_err());
            assert!(!moved.exists());
        },
        || {},
    );

    assert_eq!(result, Ok(()));
    assert!(!temporary.exists());
    assert_eq!(
        fs::read(staged.directory.join("Content/remote.txt")).unwrap(),
        b"remote"
    );
    drop(project);
    let lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert_eq!(recover_orphaned_publish_temporaries(&root, &lease), Ok(1));
    drop(lease);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn recovery_rejects_a_stage_replaced_between_marker_and_payload_reads() {
    let (root, guid) = fixture();
    let staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);
    let (artifact, temporary) = publication(&root);
    let moved = staged.directory.with_file_name("stage-held-between-reads");
    let project = AnchoredDirectory::open(&root).unwrap();

    let result = reap_one_with_hooks(
        &project,
        STAGE,
        &artifact,
        || {
            fs::rename(&staged.directory, &moved).unwrap();
            fs::create_dir_all(staged.directory.join("Content")).unwrap();
            fs::write(staged.directory.join("Content/remote.txt"), b"remote").unwrap();
            fs::copy(
                moved.join(STAGE_MARKER),
                staged.directory.join(STAGE_MARKER),
            )
            .unwrap();
        },
        || {},
    );

    // Even a byte-identical replacement is a different stage namespace.
    assert_eq!(result, Err("hub_cloud_sync_stage_invalid"));
    assert_eq!(fs::read(&temporary).unwrap(), b"rem");
    assert_eq!(
        fs::read(moved.join("Content/remote.txt")).unwrap(),
        b"remote"
    );
    assert_eq!(
        fs::read(staged.directory.join("Content/remote.txt")).unwrap(),
        b"remote"
    );
    let receipt = PathBuf::from_iter(PUBLICATION_ROOT.iter().copied())
        .join(STAGE)
        .join(publication_record_name(
            &serde_json::to_vec(&artifact).unwrap(),
        ));
    assert!(!fs::read(root.join(receipt)).unwrap().is_empty());
    drop(project);
    fs::remove_dir_all(root).unwrap();
}
