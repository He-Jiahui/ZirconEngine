use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use zircon_runtime::core::resource::{
    ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceLocator, ResourceManager,
    ResourceRecord,
};

#[derive(Debug)]
struct VersionedPayload {
    version: u64,
}

#[test]
fn resource_snapshot_never_pairs_a_new_revision_with_an_old_payload() {
    const UPDATE_COUNT: u64 = 2_000;

    let manager = ResourceManager::new();
    let locator = ResourceLocator::parse("res://models/snapshot.obj").unwrap();
    let id = ResourceId::from_locator(&locator);
    manager.register_ready(
        record(id, locator.clone(), 0),
        VersionedPayload { version: 0 },
    );

    let start = Arc::new(Barrier::new(2));
    let done = Arc::new(AtomicBool::new(false));
    let writer = {
        let manager = manager.clone();
        let start = Arc::clone(&start);
        let done = Arc::clone(&done);
        let locator = locator.clone();
        thread::spawn(move || {
            start.wait();
            for version in 1..=UPDATE_COUNT {
                manager.register_ready(
                    record(id, locator.clone(), version),
                    VersionedPayload { version },
                );
            }
            done.store(true, Ordering::Release);
        })
    };

    start.wait();
    while !done.load(Ordering::Acquire) {
        let snapshot = manager
            .snapshot::<ModelMarker, VersionedPayload>(ResourceHandle::new(id))
            .expect("registered payload snapshot");
        assert_eq!(snapshot.revision(), snapshot.version + 1);
    }
    writer.join().unwrap();

    let final_snapshot = manager
        .snapshot::<ModelMarker, VersionedPayload>(ResourceHandle::new(id))
        .unwrap();
    assert_eq!(final_snapshot.revision(), UPDATE_COUNT + 1);
    assert_eq!(final_snapshot.version, UPDATE_COUNT);
}

fn record(id: ResourceId, locator: ResourceLocator, version: u64) -> ResourceRecord {
    ResourceRecord::new(id, ResourceKind::Model, locator)
        .with_source_hash(format!("version-{version}"))
}
