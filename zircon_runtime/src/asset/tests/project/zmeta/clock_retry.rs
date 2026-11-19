//! A partially rolled-back generation must not republish an already-restored descriptor.

use std::fs::{self, File, FileTimes};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{
    artifact_revision, millis, modified, write_include_shader_package, Fixture, SEEDED_MS,
};
use crate::asset::project::{
    lock_project_generation, AssetMetaDocument, ProjectManager, ProjectPaths,
};
use crate::asset::{AssetImportError, AssetUri};
use crate::core::resource::io::transaction::{
    DurableTransactionError, TransactionFault, TransactionPhase,
};
use crate::core::resource::io::{replace_staged_file, SourceRevision};
use crate::core::resource::ResourceState;
use zircon_runtime_interface::project::RelPath;

const SECOND: &str = "stamp_probe_retry";

#[test]
fn compound_source_time_retry_keeps_touch_on_already_restored_descriptor() {
    assert_retry_preserves_restored_author_file(false);
}

#[test]
fn compound_source_time_retry_keeps_same_bytes_same_stamp_author_replacement() {
    assert_retry_preserves_restored_author_file(true);
}

fn assert_retry_preserves_restored_author_file(replace_identical: bool) {
    let fixture = Fixture::new(if replace_identical {
        "clock_retry_replace"
    } else {
        "clock_retry_touch"
    });
    let paths = ProjectPaths::from_root(&fixture.root).unwrap();
    let shaders = paths.asset_root(&RelPath::project_assets()).join("shaders");
    let second_uri = write_include_shader_package(&paths, SECOND, "asset_stamp_fixture::retry");
    let descriptors = [
        fixture.descriptor.clone(),
        shaders.join(format!("{SECOND}.zmeta")),
    ];
    assert!(descriptors[0] < descriptors[1], "full writes in path order");
    let members = [
        fixture.members[0].clone(),
        fixture.members[1].clone(),
        shaders.join(SECOND).join(format!("{SECOND}.zshader")),
        shaders.join(SECOND).join(format!("{SECOND}.wgsl")),
    ];
    for path in descriptors.iter().chain(&members) {
        set_sync_time(path, UNIX_EPOCH + Duration::from_millis(SEEDED_MS));
    }
    let mut manager = ProjectManager::open(&fixture.root).unwrap();
    manager
        .scan_and_import()
        .expect("initial durable full scan");
    let active_catalog = manager.catalog_input_generation();
    let active_index = manager.asset_registry_shared();
    let uris = [
        fixture.locators[0].clone(),
        fixture.locators[1].clone(),
        fixture.locators[2].clone(),
        second_uri.clone(),
        AssetUri::parse(&format!("{second_uri}#zshader:{SECOND}.zshader")).unwrap(),
        AssetUri::parse(&format!("{second_uri}#wgsl:{SECOND}.wgsl")).unwrap(),
    ];
    let records = uris.each_ref().map(|uri| {
        let record = manager.registry().get_by_locator(uri).unwrap();
        assert_eq!(record.state, ResourceState::Ready, "{uri}");
        record.clone()
    });
    let manifests = records.each_ref().map(|r| artifact_revision(&manager, r));
    let first_meta = descriptors
        .each_ref()
        .map(|path| AssetMetaDocument::load(path).unwrap());
    let original = descriptors.each_ref().map(|path| fs::read(path).unwrap());
    let registry = paths.registry_root().join("asset-registry.json");
    let original_index = fs::read(&registry).unwrap();
    let journal = paths.derived_root().join("project-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    let member_input = members
        .each_ref()
        .map(|p| (fs::read(p).unwrap(), modified(p)));

    // Only author timestamps change. Both new generated descriptors must differ from the old
    // bytes, but their cached resource and artifact identities and member inputs are stable.
    let highest = descriptors
        .iter()
        .map(|path| millis(modified(path)))
        .chain(first_meta.iter().map(|meta| meta.source_mtime_unix_ms))
        .max()
        .unwrap();
    let times =
        [60_000, 120_000, 180_000].map(|delta| UNIX_EPOCH + Duration::from_millis(highest + delta));
    for (path, time) in descriptors.iter().zip(times) {
        set_sync_time(path, time);
    }
    assert_eq!(
        descriptors.each_ref().map(|p| fs::read(p).unwrap()),
        original
    );

    let error = {
        let _guard = lock_project_generation(manager.paths().root()).unwrap();
        manager
            .scan_and_import_with_full_generation_fault(|commit_index| {
                assert_eq!(
                    commit_index, 2,
                    "exactly two changed descriptors then registry"
                );
                TransactionFault::RestoreFailure {
                    commit_index,
                    restore_index: 0,
                }
            })
            .expect_err("the reverse rollback restores descriptor #1, then fails #0")
    };
    let AssetImportError::Io(error) = &error else {
        panic!("not a durable transaction error: {error}");
    };
    let Some(DurableTransactionError::Operation {
        phase,
        path,
        source,
    }) = error
        .get_ref()
        .and_then(|e| e.downcast_ref::<DurableTransactionError>())
    else {
        panic!("not the injected restore failure: {error}");
    };
    assert_eq!(*phase, TransactionPhase::Rollback);
    assert_eq!(path.extension().and_then(|s| s.to_str()), Some("zrjournal"));
    assert_eq!(source.to_string(), "injected restore failure");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 1);
    assert_eq!(fs::read(&registry).unwrap(), original_index);
    assert!(Arc::ptr_eq(
        &active_catalog,
        &manager.catalog_input_generation()
    ));
    assert!(Arc::ptr_eq(&active_index, &manager.asset_registry_shared()));
    assert_eq!(
        records.each_ref().map(|r| artifact_revision(&manager, r)),
        manifests
    );
    let mut first_new = first_meta[0].clone();
    first_new.source_mtime_unix_ms = millis(times[0]);
    assert_eq!(AssetMetaDocument::load(&descriptors[0]).unwrap(), first_new);
    assert_ne!(fs::read(&descriptors[0]).unwrap(), original[0]);
    assert_eq!(fs::read(&descriptors[1]).unwrap(), original[1]);
    assert_eq!(modified(&descriptors[0]), times[0]);
    assert_eq!(modified(&descriptors[1]), times[1]);
    drop(manager);

    let restored = &descriptors[1];
    let old_object = SourceRevision::observe(restored).unwrap();
    let (author_time, author_object) = if replace_identical {
        // Editors commonly publish through an adjacent replacement. Preserve bytes *and* exact
        // physical stamp this time; the object must still not be replaced again during reopen.
        let stage = restored.with_file_name(".clock-author-save");
        let mut file = File::create_new(&stage).unwrap();
        file.write_all(&original[1]).unwrap();
        file.set_modified(times[1]).unwrap();
        file.sync_all().unwrap();
        drop(file);
        let saved = SourceRevision::observe(&stage).unwrap();
        replace_staged_file(&stage, restored).unwrap();
        assert_ne!(saved, old_object);
        (times[1], saved)
    } else {
        set_sync_time(restored, times[2]);
        (times[2], old_object)
    };
    assert_eq!(SourceRevision::observe(restored).unwrap(), author_object);
    assert_eq!(modified(restored), author_time);
    assert_eq!(fs::read(restored).unwrap(), original[1]);

    let mut reopened = ProjectManager::open(&fixture.root).unwrap_or_else(|error| {
        panic!("COMPOUND_SOURCE_TIME_REENTRY: old bytes no longer need rollback: {error}")
    });
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    assert_eq!(
        descriptors.each_ref().map(|p| fs::read(p).unwrap()),
        original
    );
    assert_eq!(fs::read(&registry).unwrap(), original_index);
    assert_eq!(modified(&descriptors[0]), times[0]);
    assert_eq!(modified(restored), author_time);
    assert_eq!(
        SourceRevision::observe(restored).unwrap(),
        author_object,
        "COMPOUND_SOURCE_TIME_REENTRY: replay republished the already-restored object"
    );
    assert!(reopened
        .catalog_input_generation()
        .records()
        .next()
        .is_none());

    reopened
        .scan_and_import()
        .expect("first post-recovery scan");
    let published = reopened.catalog_input_generation();
    for (meta, time) in first_meta.iter().zip([times[0], author_time]) {
        let row = published
            .record(crate::asset::AssetId::from_asset_uuid(meta.uuid))
            .expect("ready compound source in new catalog");
        assert_eq!(row.meta().source_mtime_unix_ms, millis(time));
        assert_eq!(row.source_mtime_unix_ms(), millis(time));
    }
    for record in &records {
        assert_eq!(
            reopened.registry().get_by_locator(record.primary_locator()),
            Some(record)
        );
    }
    assert_eq!(
        records.each_ref().map(|r| artifact_revision(&reopened, r)),
        manifests
    );
    assert_eq!(
        members
            .each_ref()
            .map(|p| (fs::read(p).unwrap(), modified(p))),
        member_input
    );
}

fn set_sync_time(path: &Path, time: SystemTime) {
    let file = File::options().write(true).open(path).unwrap();
    file.set_times(FileTimes::new().set_modified(time)).unwrap();
    file.sync_all().unwrap();
    drop(file);
    assert_eq!(modified(path), time, "{path:?}");
}
