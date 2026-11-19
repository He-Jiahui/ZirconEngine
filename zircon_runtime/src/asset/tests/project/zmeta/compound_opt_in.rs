//! Private trial controls: exact physical time and explicitly rejected interrupted late writes.

use super::*;

fn future_exact_stamp() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(2_000_000_000) + Duration::from_nanos(123_400)
}

#[test]
fn compound_source_time_opt_in_preserves_exact_future_time_through_targeted_and_cold_full() {
    let fixture = Fixture::new("compound_targeted_exact_clock");
    let (mut manager, before) = Published::first(&fixture);
    let stamp = future_exact_stamp();
    set_mtime(&fixture.descriptor, stamp);
    assert_eq!(modified(&fixture.descriptor), stamp);
    let author_object = descriptor_revision(&fixture);
    let generation = crate::asset::project::lock_project_generation(&fixture.root).unwrap();
    manager
        .import_targeted_source(&fixture.locators[0], fixture.members[0].parent().unwrap())
        .expect("real targeted compound import and publication");
    drop(generation);
    assert_ne!(
        descriptor_revision(&fixture),
        author_object,
        "targeted wrote the descriptor"
    );
    assert_eq!(
        modified(&fixture.descriptor),
        stamp,
        "targeted publication must not quantize raw time"
    );
    let scan = Scan::read(&fixture, &manager);
    let records = scan.ready(&fixture, &manager, before.scan.meta.uuid);
    assert_eq!(scan.meta.source_mtime_unix_ms, millis(stamp));
    let payloads = fixture.payloads(&manager);
    assert_shader_payloads(&payloads, &scan.members);
    let manifests = records
        .each_ref()
        .map(|record| artifact_revision(&manager, record));
    let ready = Published {
        scan,
        records,
        payloads,
        manifests,
    };
    drop(manager);
    let reopened = ProjectManager::open(&fixture.root).unwrap();
    let reopened = ready.cold_scan(&fixture, reopened);
    ready.assert_clock(&reopened, millis(stamp), stamp, "targeted-then-cold");
}

#[test]
fn compound_source_time_opt_in_restores_sub_millisecond_time_in_both_modes() {
    for reopen in [false, true] {
        let fixture = Fixture::new(if reopen {
            "compound_recover_raw"
        } else {
            "compound_rollback_raw"
        });
        let (mut manager, before) = Published::first(&fixture);
        let stamp = future_exact_stamp();
        set_mtime(&fixture.descriptor, stamp);
        assert_eq!(modified(&fixture.descriptor), stamp);
        if reopen {
            let error = manager
                .scan_and_import_with_last_commit_interruption()
                .unwrap_err();
            assert!(error
                .to_string()
                .contains("injected interruption after committed transition"));
            assert_eq!(
                AssetMetaDocument::load(&fixture.descriptor)
                    .unwrap()
                    .source_mtime_unix_ms,
                millis(stamp)
            );
        } else {
            let error = manager
                .scan_and_import_with_commit_failure_before_registry(None)
                .unwrap_err();
            assert!(error
                .to_string()
                .contains("injected transaction commit failure"));
            assert_eq!(
                modified(&fixture.descriptor),
                stamp,
                "live rollback (before reopen)"
            );
        }
        drop(manager);
        let _reopened = ProjectManager::open(&fixture.root).unwrap();
        assert_eq!(
            fs::read(&fixture.descriptor).unwrap(),
            before.scan.descriptor_text.as_bytes()
        );
        assert_eq!(
            modified(&fixture.descriptor),
            stamp,
            "raw source time must survive restore"
        );
    }
}

#[test]
fn compound_source_time_opt_in_refuses_clock_conflict_before_any_reopen_restore() {
    let fixture = Fixture::new("compound_recover_late_touch");
    let (mut manager, before) = Published::first(&fixture);
    let stamp = future_exact_stamp();
    set_mtime(&fixture.descriptor, stamp);
    assert_eq!(modified(&fixture.descriptor), stamp);
    let error = manager
        .scan_and_import_with_last_commit_interruption()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("injected interruption after committed transition"));
    let registry = manager.paths().registry_root().join("asset-registry.json");
    let committed_registry = fs::read(&registry).unwrap();
    let journal = manager.paths().derived_root().join("project-generation");
    assert_eq!(entries(&journal), 1);
    drop(manager);

    let bytes = fs::read(&fixture.descriptor).unwrap();
    assert_ne!(bytes, before.scan.descriptor_text.as_bytes());
    let object = descriptor_revision(&fixture);
    // Same millisecond, same live bytes and ID, but a real distinct raw FILETIME.
    let later = stamp + Duration::from_nanos(100);
    set_mtime(&fixture.descriptor, later);
    let observed_later = modified(&fixture.descriptor);
    assert_eq!(observed_later, later);
    assert_eq!(millis(observed_later), millis(stamp));
    assert_eq!(descriptor_revision(&fixture), object);
    let failure =
        ProjectManager::open(&fixture.root).expect_err("late touch must not be erased by recovery");
    let AssetImportError::Io(error) = &failure else {
        panic!("{failure}")
    };
    let log_error = error
        .get_ref()
        .and_then(|error| error.downcast_ref::<DurableTransactionError>());
    assert!(
        matches!(log_error, Some(DurableTransactionError::InvalidJournal {reason,..}) if reason.contains("cannot restore clock-protected target")),
        "{failure}"
    );
    assert_eq!(entries(&journal), 1);
    assert_eq!(fs::read(&registry).unwrap(), committed_registry);
    assert_eq!(fs::read(&fixture.descriptor).unwrap(), bytes);
    assert_eq!(modified(&fixture.descriptor), later);
    assert_eq!(descriptor_revision(&fixture), object);
}
